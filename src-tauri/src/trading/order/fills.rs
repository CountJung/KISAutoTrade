use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::{
    api::rest::KisRestClient,
    broker::{BrokerId, TossBrokerAdapter},
    config::AccountProfile,
    market_hours::is_domestic_symbol,
    notifications::types::NotificationEvent,
    storage::{
        order_store::{OrderSide, OrderStatus},
        trade_store::{TradeRecord, TradeSide},
    },
};

use super::{conflicts::pending_order_provider, OrderManager};

struct PendingFill {
    odno: String,
    filled_qty: u64,
    avg_price: u64,
    terminal_status: Option<OrderStatus>,
    provider_status: String,
    execution_date: chrono::NaiveDate,
}

impl OrderManager {
    /// ④ 체결 이벤트 처리 (WebSocket H0STCNI0 또는 폴링에서 호출)
    ///
    /// - `odno`: KIS 주문번호
    /// - `filled_qty`: 주문번호 기준 누적 체결 수량
    /// - `avg_price`: 누적 체결 평균가(국내 원, 해외 USD × 100 cents)
    pub async fn on_fill(&mut self, odno: &str, filled_qty: u64, avg_price: u64) -> Result<()> {
        self.on_fill_on(
            odno,
            filled_qty,
            avg_price,
            chrono::Local::now().date_naive(),
        )
        .await
    }

    async fn on_fill_on(
        &mut self,
        odno: &str,
        filled_qty: u64,
        avg_price: u64,
        execution_date: chrono::NaiveDate,
    ) -> Result<()> {
        let Some(pending) = self.pending.get(odno).cloned() else {
            tracing::debug!("on_fill: odno {} 는 미체결 풀에 없음 (이미 처리됨)", odno);
            return Ok(());
        };

        let symbol = pending.record.symbol.clone();
        let symbol_name = pending.record.symbol_name.clone();
        let order_quantity = pending.record.quantity;
        let confirmed_filled = filled_qty
            .max(pending.confirmed_filled_quantity)
            .min(order_quantity);
        let confirmed_avg_price = if filled_qty >= pending.confirmed_filled_quantity {
            avg_price
        } else {
            pending.confirmed_avg_price
        };
        let Some((delta_qty, cumulative_filled, is_complete)) =
            cumulative_fill_delta(pending.filled_quantity, confirmed_filled, order_quantity)
        else {
            tracing::debug!(
                "on_fill: odno {} 누적 체결량 변화 없음 ({} / {})",
                odno,
                pending.filled_quantity,
                order_quantity
            );
            return Ok(());
        };
        let (fill_price, cumulative_notional) = cumulative_fill_delta_price(
            pending.filled_notional,
            cumulative_filled,
            confirmed_avg_price,
            delta_qty,
        );
        if fill_price == 0 {
            anyhow::bail!("on_fill: odno {odno} 증가분 체결가를 계산할 수 없습니다.");
        }

        let is_sell = matches!(pending.record.side, OrderSide::Sell);
        let is_overseas = pending.exchange.is_some() || !is_domestic_symbol(&symbol);
        let first_application = !pending.application_started;
        let pnl = if let Some(pnl) = pending.application_pnl {
            pnl
        } else if !is_sell {
            0
        } else if is_overseas {
            let tracker = self.overseas_position_tracker.lock().await;
            let buy_avg = tracker
                .get(&symbol)
                .map(|position| position.avg_price_cents)
                .unwrap_or(0.0);
            ((fill_price as f64 - buy_avg) * delta_qty as f64) as i64
        } else {
            let tracker = self.position_tracker.lock().await;
            let buy_avg = tracker
                .get(&symbol)
                .map(|position| position.avg_price)
                .unwrap_or(0.0);
            ((fill_price as f64 - buy_avg) * delta_qty as f64) as i64
        };
        if let Some(current) = self.pending.get_mut(odno) {
            current.confirmed_filled_quantity = cumulative_filled;
            current.confirmed_avg_price = confirmed_avg_price;
            current.application_started = true;
            current.application_pnl = Some(pnl);
            current.provider_status = Some("applying_fill".to_string());
        }
        if let Err(error) = self.persist_pending_orders().await {
            self.block_for_persistence_failure(format!("체결 intent 영속화 실패: {error}"));
            return Err(error);
        }

        let exchange_rate = if is_overseas {
            *self.exchange_rate_krw.read().await
        } else {
            1.0
        };

        let fee = if is_overseas {
            calculate_overseas_fee_cents(fill_price, delta_qty)
        } else {
            calculate_domestic_fee(fill_price, delta_qty, is_sell)
        };
        let fee_krw = if is_overseas {
            ((fee as f64 / 100.0) * exchange_rate).round().max(0.0) as u64
        } else {
            fee
        };
        let pnl_krw = if is_overseas {
            ((pnl as f64 / 100.0) * exchange_rate).round() as i64
        } else {
            pnl
        };
        let fill_event_id = format!(
            "fill:{:?}:{}:{}:{}:{}:{}",
            pending.broker_scope.broker_id,
            pending
                .broker_scope
                .account_id
                .as_ref()
                .map(|account| account.0.as_str())
                .unwrap_or("default"),
            if is_overseas { "overseas" } else { "domestic" },
            pending
                .record
                .provider_order_id
                .as_deref()
                .or(pending.record.kis_order_id.as_deref())
                .unwrap_or(odno),
            execution_date,
            cumulative_filled,
        );

        if is_sell {
            if first_application && self.buy_suspended && !self.persistence_blocked {
                self.buy_suspended = false;
                self.buy_suspended_reason = None;
                tracing::info!(
                    "매도 체결로 자본 확보 — 잔고 부족 매수 정지 해제: {}",
                    symbol
                );
            }
            if first_application {
                {
                    let mut risk = self.risk_manager.lock().await;
                    risk.record_pnl(pnl_krw);
                    risk.record_strategy_symbol_pnl_for_scope(
                        &pending.broker_scope,
                        pending.strategy_id.as_deref().unwrap_or("unknown"),
                        &symbol,
                        pnl_krw,
                    );
                }
                self.persist_risk_runtime().await;
            }

            let mut stats = match self.stats_store.get_by_date(execution_date).await {
                Ok(stats) => stats,
                Err(error) => {
                    self.block_for_persistence_failure(format!("체결 통계 로드 실패: {error}"));
                    return Err(error);
                }
            };
            if stats.processed_fill_ids.contains(&fill_event_id) {
                // retry: 이미 통계에 반영됨
            } else {
                stats.total_trades += 1;
                if pnl_krw > 0 {
                    stats.winning_trades += 1;
                    stats.gross_profit += pnl_krw;
                } else if pnl_krw < 0 {
                    stats.losing_trades += 1;
                    stats.gross_loss += pnl_krw;
                }
                stats.fees_paid += fee_krw;
                stats.processed_fill_ids.push(fill_event_id.clone());
                stats.recalculate();
                if let Err(e) = self.stats_store.upsert(stats).await {
                    self.block_for_persistence_failure(format!("체결 통계 영속화 실패: {e}"));
                    return Err(e);
                }
            }
        } else {
            let mut stats = match self.stats_store.get_by_date(execution_date).await {
                Ok(stats) => stats,
                Err(error) => {
                    self.block_for_persistence_failure(format!("체결 통계 로드 실패: {error}"));
                    return Err(error);
                }
            };
            if !stats.processed_fill_ids.contains(&fill_event_id) {
                stats.fees_paid += fee_krw;
                stats.processed_fill_ids.push(fill_event_id.clone());
                stats.recalculate();
                if let Err(e) = self.stats_store.upsert(stats).await {
                    self.block_for_persistence_failure(format!("체결 통계 영속화 실패: {e}"));
                    return Err(e);
                }
            }
        }

        let mut filled_record = pending.record.clone();
        filled_record.status = if is_complete {
            OrderStatus::Filled
        } else {
            OrderStatus::PartiallyFilled
        };
        filled_record.price = fill_price;
        filled_record.quantity = delta_qty;
        filled_record.id = format!("order-{fill_event_id}");
        filled_record.execution_date = Some(execution_date.to_string());
        if let Err(e) = self
            .order_store
            .append_on(execution_date, filled_record)
            .await
        {
            self.block_for_persistence_failure(format!("체결 주문 영속화 실패: {e}"));
            return Err(e);
        }

        let trade_side = match &pending.record.side {
            OrderSide::Buy => TradeSide::Buy,
            OrderSide::Sell => TradeSide::Sell,
        };
        let order_id = pending.record.kis_order_id.clone().unwrap_or_default();
        let provider_order_id = pending
            .record
            .provider_order_id
            .clone()
            .or_else(|| Some(order_id.clone()).filter(|id| !id.is_empty()));
        let mut trade_record = if is_overseas {
            TradeRecord::new_overseas(
                symbol.clone(),
                symbol_name.clone(),
                trade_side,
                delta_qty,
                fill_price,
                fee,
                order_id,
                pending.strategy_id.clone(),
                pending.signal_reason.clone(),
                pending
                    .exchange
                    .clone()
                    .unwrap_or_else(|| "UNKNOWN".to_string()),
                exchange_rate,
                is_sell.then_some(pnl),
            )
            .with_execution_prices(pending.signal_price, pending.order_price)
            .with_provider_trace(
                pending.record.provider.clone(),
                provider_order_id.clone(),
                pending.record.provider_request_id.clone(),
                pending.record.provider_tr_id.clone(),
            )
        } else {
            TradeRecord::new(
                symbol.clone(),
                symbol_name.clone(),
                trade_side,
                delta_qty,
                fill_price,
                fee,
                order_id,
                pending.strategy_id.clone(),
                pending.signal_reason.clone(),
            )
            .with_execution_prices(pending.signal_price, pending.order_price)
            .with_provider_trace(
                pending.record.provider.clone(),
                provider_order_id.clone(),
                pending.record.provider_request_id.clone(),
                pending.record.provider_tr_id.clone(),
            )
        };
        trade_record = trade_record.with_broker_scope(&pending.broker_scope);
        trade_record.id = format!("trade-{fill_event_id}");
        trade_record.execution_date = Some(execution_date.to_string());
        if !is_overseas {
            trade_record.realized_pnl_krw = is_sell.then_some(pnl_krw);
        }
        tracing::info!(
            "체결 trace 저장: provider={} tr_id={} order_id={} request_id={}",
            trade_record.provider.as_deref().unwrap_or("unknown"),
            trade_record.provider_tr_id.as_deref().unwrap_or("-"),
            trade_record.provider_order_id.as_deref().unwrap_or("-"),
            trade_record.provider_request_id.as_deref().unwrap_or("-")
        );
        if let Err(e) = self
            .trade_store
            .append_on(execution_date, trade_record)
            .await
        {
            self.block_for_persistence_failure(format!("체결 기록 영속화 실패: {e}"));
            return Err(e);
        }

        if is_complete {
            self.pending.remove(odno);
            self.symbol_to_odno.remove(&symbol);
        } else if let Some(current) = self.pending.get_mut(odno) {
            current.filled_quantity = cumulative_filled;
            current.filled_notional = cumulative_notional;
            current.record.status = OrderStatus::PartiallyFilled;
            current.record.price = confirmed_avg_price;
            current.provider_status = Some("partially_filled".to_string());
            current.application_started = false;
            current.application_pnl = None;
        }
        let mut lifecycle_record = pending.record.clone();
        lifecycle_record.status = if is_complete {
            OrderStatus::Filled
        } else {
            OrderStatus::PartiallyFilled
        };
        lifecycle_record.price = confirmed_avg_price;
        let lifecycle_date = chrono::DateTime::parse_from_rfc3339(&lifecycle_record.timestamp)
            .map(|timestamp| timestamp.date_naive())
            .unwrap_or(execution_date);
        if let Err(error) = self
            .order_store
            .upsert_on(lifecycle_date, lifecycle_record)
            .await
        {
            self.block_for_persistence_failure(format!("주문 상태 전이 영속화 실패: {error}"));
            return Err(error);
        }
        if let Err(error) = self.persist_pending_orders().await {
            self.block_for_persistence_failure(format!("체결 watermark 영속화 실패: {error}"));
            return Err(error);
        }

        if first_application {
            if is_overseas {
                let exchange = pending
                    .exchange
                    .clone()
                    .unwrap_or_else(|| "UNKNOWN".to_string());
                let mut tracker = self.overseas_position_tracker.lock().await;
                match pending.record.side {
                    OrderSide::Buy => tracker.on_buy(
                        symbol.clone(),
                        symbol_name.clone(),
                        exchange,
                        delta_qty,
                        fill_price,
                    ),
                    OrderSide::Sell => tracker.on_sell(&symbol, delta_qty),
                }
            } else {
                let mut tracker = self.position_tracker.lock().await;
                match pending.record.side {
                    OrderSide::Buy => {
                        tracker.on_buy(symbol.clone(), symbol_name.clone(), delta_qty, fill_price)
                    }
                    OrderSide::Sell => tracker.on_sell(&symbol, delta_qty),
                }
            }
        }

        if let Some(discord) = &self.discord {
            let side_str = if !is_sell { "매수" } else { "매도" };
            let pnl_str = if is_sell {
                if is_overseas {
                    format!(
                        " (PnL: {}${:.2}, {}{}원, 수수료 약 {}원)",
                        if pnl >= 0 { "+" } else { "" },
                        pnl as f64 / 100.0,
                        if pnl_krw >= 0 { "+" } else { "" },
                        pnl_krw,
                        fee_krw
                    )
                } else {
                    format!(
                        " (PnL: {}{}원, 수수료: {}원)",
                        if pnl >= 0 { "+" } else { "" },
                        pnl,
                        fee
                    )
                }
            } else if is_overseas {
                format!(" (수수료 약 {}원)", fee_krw)
            } else {
                format!(" (수수료: {}원)", fee)
            };
            let price_text = if is_overseas {
                format!("${:.2}", fill_price as f64 / 100.0)
            } else {
                format!("{}원", fill_price)
            };
            let content = format!(
                "{} {} {}주 @ {}{}",
                symbol_name, side_str, delta_qty, price_text, pnl_str
            );
            let _ = discord.send(NotificationEvent::trade(content)).await;
        }

        Ok(())
    }

    /// 주문번호 기반 체결 확인.
    ///
    /// KIS 당일 체결 내역에서 pending 주문번호를 찾아 실제 체결수량/체결금액으로 반영한다.
    /// 국내는 원화 정수, 해외는 USD × 100(cents) 단위로 `on_fill()`에 전달한다.
    pub async fn confirm_pending_fills_from_broker(&mut self) -> Result<()> {
        let pending = self.pending_fill_candidates();
        if pending.is_empty() {
            return Ok(());
        }

        let client = self.rest_client.read().await.clone();
        let fills = collect_kis_pending_fills(client, &kis_pending_from(&pending)).await?;
        let toss_fills = collect_toss_pending_fills(
            self.active_toss_profile().await,
            &toss_pending_from(&pending),
        )
        .await?;
        let mut fills = fills;
        fills.extend(toss_fills);
        self.apply_pending_fills(fills).await?;

        Ok(())
    }

    /// 주문번호 기반 체결 확인을 shared `OrderManager` mutex 밖 네트워크 조회로 수행한다.
    pub async fn confirm_pending_fills_from_broker_shared(
        order_manager: &Arc<Mutex<OrderManager>>,
    ) -> Result<()> {
        let result = Self::confirm_pending_fills_from_broker_shared_inner(order_manager).await;
        let error = result.as_ref().err().map(ToString::to_string);
        order_manager.lock().await.mark_reconciliation(error);
        result
    }

    async fn confirm_pending_fills_from_broker_shared_inner(
        order_manager: &Arc<Mutex<OrderManager>>,
    ) -> Result<()> {
        let (pending, rest_client, active_toss_profile) = {
            let manager = order_manager.lock().await;
            if manager.pending.is_empty() {
                return Ok(());
            }
            (
                manager.pending_fill_candidates(),
                Arc::clone(&manager.rest_client),
                manager.active_toss_profile().await,
            )
        };

        let client = rest_client.read().await.clone();
        let mut fills = collect_kis_pending_fills(client, &kis_pending_from(&pending)).await?;
        fills.extend(
            collect_toss_pending_fills(active_toss_profile, &toss_pending_from(&pending)).await?,
        );

        if !fills.is_empty() {
            let mut manager = order_manager.lock().await;
            manager.apply_pending_fills(fills).await?;
        }

        Ok(())
    }

    async fn active_toss_profile(&self) -> Option<AccountProfile> {
        let profiles = self.profiles.read().await;
        let account_id = self
            .execution_scope
            .account_id
            .as_ref()
            .map(|id| id.0.as_str());
        profiles
            .profiles
            .iter()
            .find(|profile| {
                profile.broker_id == BrokerId::Toss
                    && account_id
                        .map(|id| profile.broker_account_id() == id)
                        .unwrap_or_else(|| {
                            profiles.get_active().map(|p| p.id.as_str())
                                == Some(profile.id.as_str())
                        })
            })
            .cloned()
    }

    fn pending_fill_candidates(&self) -> Vec<(BrokerId, String, String, String)> {
        self.pending
            .iter()
            .filter(|(odno, order)| {
                order.broker_scope == self.execution_scope && !self.modifying.contains(*odno)
            })
            .map(|(odno, order)| {
                let is_overseas =
                    order.exchange.is_some() || !is_domestic_symbol(&order.record.symbol);
                (
                    pending_order_provider(order),
                    odno.clone(),
                    order.record.symbol.clone(),
                    chrono::DateTime::parse_from_rfc3339(&order.record.timestamp)
                        .map(|timestamp| {
                            if is_overseas {
                                timestamp
                                    .with_timezone(&chrono_tz::America::New_York)
                                    .format("%Y%m%d")
                                    .to_string()
                            } else {
                                timestamp.format("%Y%m%d").to_string()
                            }
                        })
                        .unwrap_or_else(|_| chrono::Local::now().format("%Y%m%d").to_string()),
                )
            })
            .collect()
    }

    async fn apply_pending_fills(&mut self, fills: Vec<PendingFill>) -> Result<()> {
        for fill in fills {
            if fill.filled_qty > 0 {
                self.on_fill_on(
                    &fill.odno,
                    fill.filled_qty,
                    fill.avg_price,
                    fill.execution_date,
                )
                .await?;
            }
            if let Some(status) = fill.terminal_status {
                self.finalize_pending_order(&fill.odno, status, &fill.provider_status, None)
                    .await?;
            }
        }
        Ok(())
    }
}

fn cumulative_fill_delta(
    previous_filled: u64,
    reported_filled: u64,
    order_quantity: u64,
) -> Option<(u64, u64, bool)> {
    let cumulative = reported_filled.min(order_quantity);
    (cumulative > previous_filled).then(|| {
        (
            cumulative - previous_filled,
            cumulative,
            cumulative >= order_quantity,
        )
    })
}

fn cumulative_fill_delta_price(
    previous_notional: u128,
    cumulative_filled: u64,
    cumulative_avg_price: u64,
    delta_quantity: u64,
) -> (u64, u128) {
    let cumulative_notional = cumulative_avg_price as u128 * cumulative_filled as u128;
    let delta_notional = cumulative_notional.saturating_sub(previous_notional);
    (
        (delta_notional / delta_quantity.max(1) as u128) as u64,
        cumulative_notional,
    )
}

fn kis_pending_from(
    pending: &[(BrokerId, String, String, String)],
) -> Vec<(String, String, String)> {
    pending
        .iter()
        .filter(|(broker_id, _, _, _)| *broker_id == BrokerId::Kis)
        .map(|(_, odno, symbol, date)| (odno.clone(), symbol.clone(), date.clone()))
        .collect()
}

fn toss_pending_from(pending: &[(BrokerId, String, String, String)]) -> Vec<(String, String)> {
    pending
        .iter()
        .filter(|(broker_id, _, _, _)| *broker_id == BrokerId::Toss)
        .map(|(_, order_id, symbol, _)| (order_id.clone(), symbol.clone()))
        .collect()
}

async fn collect_kis_pending_fills(
    client: Arc<KisRestClient>,
    pending: &[(String, String, String)],
) -> Result<Vec<PendingFill>> {
    let mut fills = Vec::new();
    let from = pending
        .iter()
        .map(|(_, _, date)| date.as_str())
        .min()
        .unwrap_or_else(|| "");
    let today = chrono::Local::now().format("%Y%m%d").to_string();

    if pending
        .iter()
        .any(|(_, symbol, _)| is_domestic_symbol(symbol))
    {
        let executed = client.get_executed_orders_range(from, &today).await?;
        for (odno, symbol, pending_date) in pending
            .iter()
            .filter(|(_, symbol, _)| is_domestic_symbol(symbol))
        {
            let Some(order) = executed.iter().find(|order| {
                order.odno == *odno
                    && order.pdno.eq_ignore_ascii_case(symbol)
                    && order.ord_dt == *pending_date
            }) else {
                continue;
            };
            let qty = normalized_execution_u64(&order.tot_ccld_qty);
            let terminal_status = domestic_order_terminal_status(order);
            if qty == 0 && terminal_status.is_none() {
                continue;
            }
            let amount = normalized_execution_u64(&order.tot_ccld_amt);
            let avg_price = if qty > 0 && amount > 0 {
                amount / qty
            } else if qty > 0 {
                normalized_execution_u64(&order.ord_unpr)
            } else {
                0
            };
            if qty > 0 && avg_price == 0 {
                continue;
            }
            tracing::info!(
                "국내 주문번호 기반 체결 확인: odno={} symbol={} qty={} avg={}",
                odno,
                symbol,
                qty,
                avg_price
            );
            let execution_date = parse_provider_execution_date(&order.ord_dt).ok_or_else(|| {
                anyhow::anyhow!("KIS 국내 체결일을 해석할 수 없습니다: {}", order.ord_dt)
            })?;
            fills.push(PendingFill {
                odno: odno.clone(),
                filled_qty: qty,
                avg_price,
                terminal_status,
                provider_status: domestic_provider_status(order),
                execution_date,
            });
        }
    }

    if pending
        .iter()
        .any(|(_, symbol, _)| !is_domestic_symbol(symbol))
    {
        let executed = client
            .get_overseas_executed_orders_range(from, &today)
            .await?;
        for (odno, symbol, pending_date) in pending
            .iter()
            .filter(|(_, symbol, _)| !is_domestic_symbol(symbol))
        {
            let Some(order) = executed.iter().find(|order| {
                order.odno == *odno
                    && order.pdno.eq_ignore_ascii_case(symbol)
                    && order.ord_dt == *pending_date
            }) else {
                continue;
            };
            let qty = order.filled_qty();
            let terminal_status = overseas_order_terminal_status(order);
            if qty == 0 && terminal_status.is_none() {
                continue;
            }
            let avg_price_cents = order.avg_price_cents();
            if qty > 0 && avg_price_cents == 0 {
                continue;
            }
            tracing::info!(
                "해외 주문번호 기반 체결 확인: odno={} symbol={} qty={} avg_cents={}",
                odno,
                symbol,
                qty,
                avg_price_cents
            );
            let execution_date = parse_provider_execution_date(&order.ord_dt).ok_or_else(|| {
                anyhow::anyhow!("KIS 해외 체결일을 해석할 수 없습니다: {}", order.ord_dt)
            })?;
            fills.push(PendingFill {
                odno: odno.clone(),
                filled_qty: qty,
                avg_price: avg_price_cents,
                terminal_status,
                provider_status: order.prcs_stat_name.clone(),
                execution_date,
            });
        }
    }

    Ok(fills)
}

async fn collect_toss_pending_fills(
    profile: Option<AccountProfile>,
    pending: &[(String, String)],
) -> Result<Vec<PendingFill>> {
    if pending.is_empty() {
        return Ok(Vec::new());
    }
    let Some(profile) = profile else {
        anyhow::bail!(
            "Toss pending {}건을 확인할 활성 Toss 프로파일이 없습니다.",
            pending.len()
        );
    };
    let account_seq = profile.broker_account_id();
    if account_seq.trim().is_empty() {
        anyhow::bail!(
            "Toss pending {}건을 확인할 accountSeq가 비어 있습니다.",
            pending.len()
        );
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(account_seq.clone()),
    );
    let mut fills = Vec::new();
    for (order_id, symbol) in pending {
        let order = adapter
            .get_order(Some(&account_seq), order_id)
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Toss 주문번호 기반 체결 확인 실패: order_id={order_id} symbol={symbol} error={e}"
                )
            })?;
        let qty = storage_quantity_units(&order.execution.filled_quantity);
        let terminal_status = provider_terminal_status(order.status.as_str());
        let avg_units = order
            .execution
            .average_filled_price
            .as_deref()
            .map(|price| storage_money_units(price, &order.currency))
            .unwrap_or(0);
        if qty == 0 && terminal_status.is_none() {
            continue;
        }
        if qty > 0 && avg_units == 0 {
            anyhow::bail!("Toss {order_id} 체결수량은 있으나 평균체결가가 없습니다.");
        }
        tracing::info!(
            "Toss 주문번호 기반 체결 확인: order_id={} symbol={} status={} qty={} avg={}",
            order_id,
            symbol,
            order.status,
            qty,
            avg_units
        );
        let execution_date = order
            .execution
            .filled_at
            .as_deref()
            .and_then(parse_provider_execution_date)
            .or_else(|| parse_provider_execution_date(&order.ordered_at))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Toss 주문의 체결일/주문일을 해석할 수 없습니다: order_id={order_id}"
                )
            })?;
        fills.push(PendingFill {
            odno: order_id.clone(),
            filled_qty: qty,
            avg_price: avg_units,
            terminal_status,
            provider_status: order.status.clone(),
            execution_date,
        });
    }
    Ok(fills)
}

fn storage_quantity_units(value: &str) -> u64 {
    value
        .trim()
        .replace(',', "")
        .parse::<f64>()
        .map(|v| v.max(0.0).floor() as u64)
        .unwrap_or(0)
}

fn normalized_execution_u64(value: &str) -> u64 {
    value.trim().replace(',', "").parse::<u64>().unwrap_or(0)
}

fn provider_terminal_status(status: &str) -> Option<OrderStatus> {
    match status.trim().to_ascii_uppercase().as_str() {
        "FILLED" => Some(OrderStatus::Filled),
        "CANCELED" | "CANCELLED" => Some(OrderStatus::Cancelled),
        "REJECTED" => Some(OrderStatus::Rejected),
        "EXPIRED" => Some(OrderStatus::Expired),
        "FAILED" => Some(OrderStatus::Failed),
        _ => None,
    }
}

fn parse_provider_execution_date(value: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(value.trim(), "%Y%m%d")
        .ok()
        .or_else(|| {
            chrono::DateTime::parse_from_rfc3339(value.trim())
                .ok()
                .map(|timestamp| timestamp.date_naive())
        })
        .or_else(|| chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok())
}

#[cfg(test)]
fn domestic_order_is_terminal(order: &crate::api::rest::ExecutedOrder) -> bool {
    domestic_order_terminal_status(order).is_some()
}

fn domestic_order_terminal_status(order: &crate::api::rest::ExecutedOrder) -> Option<OrderStatus> {
    let ordered = normalized_execution_u64(&order.ord_qty);
    let filled = normalized_execution_u64(&order.tot_ccld_qty);
    let canceled = normalized_execution_u64(&order.cnc_cfrm_qty);
    let rejected = normalized_execution_u64(&order.rjct_qty);
    let remaining = if order.rmn_qty.trim().is_empty() {
        ordered.saturating_sub(filled.saturating_add(canceled).saturating_add(rejected))
    } else {
        normalized_execution_u64(&order.rmn_qty)
    };
    if ordered > 0 && filled >= ordered {
        Some(OrderStatus::Filled)
    } else if ordered > 0 && remaining == 0 && rejected > 0 {
        Some(OrderStatus::Rejected)
    } else if ordered > 0
        && remaining == 0
        && (order.cncl_yn.trim().eq_ignore_ascii_case("Y") || canceled > 0)
    {
        Some(OrderStatus::Cancelled)
    } else if ordered > 0
        && remaining == 0
        && filled.saturating_add(canceled).saturating_add(rejected) >= ordered
    {
        Some(OrderStatus::Failed)
    } else {
        None
    }
}

fn domestic_provider_status(order: &crate::api::rest::ExecutedOrder) -> String {
    match domestic_order_terminal_status(order) {
        Some(OrderStatus::Filled) => "FILLED",
        Some(OrderStatus::Cancelled) => "CANCELLED",
        Some(OrderStatus::Rejected) => "REJECTED",
        Some(OrderStatus::Failed) => "FAILED",
        _ => "PENDING",
    }
    .to_string()
}

fn overseas_order_terminal_status(
    order: &crate::api::rest::OverseasExecutedOrder,
) -> Option<OrderStatus> {
    if !order.is_terminal() {
        return None;
    }
    let status = order.prcs_stat_name.trim().to_ascii_lowercase();
    if !order.rjct_rson.trim().is_empty()
        || matches!(status.as_str(), "주문거부" | "거부" | "rejected")
    {
        Some(OrderStatus::Rejected)
    } else if status == "expired" {
        Some(OrderStatus::Expired)
    } else if matches!(
        status.as_str(),
        "취소완료" | "주문취소" | "canceled" | "cancelled"
    ) {
        Some(OrderStatus::Cancelled)
    } else if order.filled_qty() > 0 {
        Some(OrderStatus::Filled)
    } else {
        Some(OrderStatus::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cumulative_fill_delta, cumulative_fill_delta_price, domestic_order_is_terminal,
        domestic_order_terminal_status, provider_terminal_status,
    };
    use crate::storage::order_store::OrderStatus;

    #[test]
    fn repeated_cumulative_fill_is_idempotent() {
        assert_eq!(cumulative_fill_delta(3, 3, 10), None);
        assert_eq!(cumulative_fill_delta(3, 2, 10), None);
        assert_eq!(cumulative_fill_delta(3, 7, 10), Some((4, 7, false)));
        assert_eq!(cumulative_fill_delta(7, 10, 10), Some((3, 10, true)));
        assert_eq!(cumulative_fill_delta(10, 10, 10), None);
    }

    #[test]
    fn cumulative_average_is_converted_to_delta_fill_price() {
        // 2주 @ 100 이후 누적 5주 평균 112면 추가 3주는 @ 120이다.
        assert_eq!(cumulative_fill_delta_price(200, 5, 112, 3), (120, 560));
    }

    #[test]
    fn domestic_zero_fill_cancel_is_terminal() {
        let order = crate::api::rest::ExecutedOrder {
            ord_qty: "10".into(),
            tot_ccld_qty: "0".into(),
            rmn_qty: "0".into(),
            cncl_yn: "Y".into(),
            ..Default::default()
        };
        assert!(domestic_order_is_terminal(&order));
        assert_eq!(
            domestic_order_terminal_status(&order),
            Some(OrderStatus::Cancelled)
        );
    }

    #[test]
    fn domestic_partial_cancel_with_remaining_quantity_is_not_terminal() {
        let order = crate::api::rest::ExecutedOrder {
            ord_qty: "10".into(),
            tot_ccld_qty: "2".into(),
            cnc_cfrm_qty: "3".into(),
            rmn_qty: "5".into(),
            cncl_yn: "Y".into(),
            ..Default::default()
        };
        assert!(!domestic_order_is_terminal(&order));
    }

    #[test]
    fn overseas_order_received_complete_label_is_not_terminal_with_remaining() {
        let order = crate::api::rest::OverseasExecutedOrder {
            ft_ord_qty: "10".into(),
            ft_ccld_qty: "2".into(),
            nccs_qty: "8".into(),
            prcs_stat_name: "주문접수완료".into(),
            ..Default::default()
        };
        assert!(!order.is_terminal());
    }

    #[test]
    fn provider_terminal_status_preserves_rejection_and_expiration() {
        assert_eq!(
            provider_terminal_status("REJECTED"),
            Some(OrderStatus::Rejected)
        );
        assert_eq!(
            provider_terminal_status("expired"),
            Some(OrderStatus::Expired)
        );
        assert_eq!(
            provider_terminal_status("CANCELLED"),
            Some(OrderStatus::Cancelled)
        );
        assert_eq!(provider_terminal_status("PENDING"), None);
    }
}

fn storage_money_units(value: &str, currency: &str) -> u64 {
    let parsed = value
        .trim()
        .replace(',', "")
        .parse::<f64>()
        .unwrap_or(0.0)
        .max(0.0);
    if currency.eq_ignore_ascii_case("USD") {
        (parsed * 100.0).round() as u64
    } else {
        parsed.round() as u64
    }
}

/// 국내주식 매매 수수료 추정
///
/// # 구성 (2024~2025년 기준)
/// - 위탁수수료: 0.015% (매수·매도 모두)
/// - 증권거래세: 0.20% (매도 시에만, 코스피/코스닥 모두 동일 적용)
///
/// KIS API(`TTTC8001R`)는 체결 건별(output1) 수수료를 제공하지 않으며
/// output2 합산(`prsm_tlex_smtl`) 에만 전체 기간 추정제비용이 있다.
/// 따라서 체결 시 표준 수수료율로 추정한 값을 로컬에 기록한다.
fn calculate_domestic_fee(price: u64, quantity: u64, is_sell: bool) -> u64 {
    let total = price * quantity;
    let commission = (total as f64 * 0.00015) as u64;
    let transaction_tax = if is_sell {
        (total as f64 * 0.002) as u64
    } else {
        0
    };
    commission + transaction_tax
}

/// 해외주식 매매 수수료 추정.
///
/// KIS 해외 잔고/체결 API는 건별 수수료를 제공하지 않으므로 자동매매 guard의
/// 기본 해외 비용 추정치와 같은 10bps(0.10%)를 사용한다. 금액 단위는 USD cents.
fn calculate_overseas_fee_cents(price_cents: u64, quantity: u64) -> u64 {
    let total_cents = price_cents.saturating_mul(quantity);
    ((total_cents as f64) * 0.001).ceil() as u64
}
