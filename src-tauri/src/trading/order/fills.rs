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
}

impl OrderManager {
    /// ④ 체결 이벤트 처리 (WebSocket H0STCNI0 또는 폴링에서 호출)
    ///
    /// - `odno`: KIS 주문번호
    /// - `filled_qty`: 주문번호 기준 누적 체결 수량
    /// - `avg_price`: 누적 체결 평균가(국내 원, 해외 USD × 100 cents)
    pub async fn on_fill(&mut self, odno: &str, filled_qty: u64, avg_price: u64) -> Result<()> {
        let Some(pending) = self.pending.get(odno).cloned() else {
            tracing::debug!("on_fill: odno {} 는 미체결 풀에 없음 (이미 처리됨)", odno);
            return Ok(());
        };

        let symbol = pending.record.symbol.clone();
        let symbol_name = pending.record.symbol_name.clone();
        let order_quantity = pending.record.quantity;
        let cumulative_filled = filled_qty.min(order_quantity);
        if cumulative_filled <= pending.filled_quantity {
            tracing::debug!(
                "on_fill: odno {} 누적 체결량 변화 없음 ({} / {})",
                odno,
                pending.filled_quantity,
                order_quantity
            );
            return Ok(());
        }

        let delta_qty = cumulative_filled - pending.filled_quantity;
        let is_complete = cumulative_filled >= order_quantity;

        if is_complete {
            self.pending.remove(odno);
            self.symbol_to_odno.remove(&symbol);
        } else if let Some(current) = self.pending.get_mut(odno) {
            current.filled_quantity = cumulative_filled;
            current.record.status = OrderStatus::PartiallyFilled;
            current.record.price = avg_price;
        }

        let is_sell = matches!(pending.record.side, OrderSide::Sell);
        let is_overseas = pending.exchange.is_some() || !is_domestic_symbol(&symbol);
        let pnl = if is_overseas {
            let exchange = pending
                .exchange
                .clone()
                .unwrap_or_else(|| "UNKNOWN".to_string());
            let mut tracker = self.overseas_position_tracker.lock().await;
            match &pending.record.side {
                OrderSide::Buy => {
                    tracker.on_buy(
                        symbol.clone(),
                        symbol_name.clone(),
                        exchange.clone(),
                        delta_qty,
                        avg_price,
                    );
                    tracing::info!(
                        "해외 매수 체결: {} {} @ ${:.2} ({})",
                        symbol,
                        delta_qty,
                        avg_price as f64 / 100.0,
                        exchange
                    );
                    0i64
                }
                OrderSide::Sell => {
                    let buy_avg = tracker
                        .get(&symbol)
                        .map(|p| p.avg_price_cents)
                        .unwrap_or(0.0);
                    let realized = (avg_price as f64 - buy_avg) * delta_qty as f64;
                    tracker.on_sell(&symbol, delta_qty);
                    tracing::info!(
                        "해외 매도 체결: {} {} @ ${:.2} ({}) (PnL: ${:.2})",
                        symbol,
                        delta_qty,
                        avg_price as f64 / 100.0,
                        exchange,
                        realized / 100.0
                    );
                    realized as i64
                }
            }
        } else {
            let mut tracker = self.position_tracker.lock().await;
            match &pending.record.side {
                OrderSide::Buy => {
                    tracker.on_buy(symbol.clone(), symbol_name.clone(), delta_qty, avg_price);
                    tracing::info!("매수 체결: {} {} @ {}원", symbol, delta_qty, avg_price);
                    0i64
                }
                OrderSide::Sell => {
                    let buy_avg = tracker.get(&symbol).map(|p| p.avg_price).unwrap_or(0.0);
                    let realized = (avg_price as f64 - buy_avg) * delta_qty as f64;
                    tracker.on_sell(&symbol, delta_qty);
                    tracing::info!(
                        "매도 체결: {} {} @ {}원 (PnL: {}원)",
                        symbol,
                        delta_qty,
                        avg_price,
                        realized as i64
                    );
                    realized as i64
                }
            }
        };

        let exchange_rate = if is_overseas {
            *self.exchange_rate_krw.read().await
        } else {
            1.0
        };

        let fee = if is_overseas {
            calculate_overseas_fee_cents(avg_price, delta_qty)
        } else {
            calculate_domestic_fee(avg_price, delta_qty, is_sell)
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

        if is_sell {
            if self.buy_suspended {
                self.buy_suspended = false;
                self.buy_suspended_reason = None;
                tracing::info!(
                    "매도 체결로 자본 확보 — 잔고 부족 매수 정지 해제: {}",
                    symbol
                );
            }
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

            let today = chrono::Local::now().date_naive();
            if let Ok(mut stats) = self.stats_store.get_by_date(today).await {
                stats.total_trades += 1;
                if pnl_krw > 0 {
                    stats.winning_trades += 1;
                    stats.gross_profit += pnl_krw;
                } else if pnl_krw < 0 {
                    stats.losing_trades += 1;
                    stats.gross_loss += pnl_krw;
                }
                stats.fees_paid += fee_krw;
                stats.recalculate();
                if let Err(e) = self.stats_store.upsert(stats).await {
                    tracing::error!("통계 저장 실패: {}", e);
                }
            }
        } else {
            let today = chrono::Local::now().date_naive();
            if let Ok(mut stats) = self.stats_store.get_by_date(today).await {
                stats.fees_paid += fee_krw;
                stats.recalculate();
                if let Err(e) = self.stats_store.upsert(stats).await {
                    tracing::error!("통계 저장 실패 (매수 수수료): {}", e);
                }
            }
        }

        let mut filled_record = pending.record.clone();
        filled_record.status = if is_complete {
            OrderStatus::Filled
        } else {
            OrderStatus::PartiallyFilled
        };
        filled_record.price = avg_price;
        filled_record.quantity = delta_qty;
        if let Err(e) = self.order_store.append(filled_record).await {
            tracing::error!("주문 기록 저장 실패 (체결): {}", e);
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
        let trade_record = if is_overseas {
            TradeRecord::new_overseas(
                symbol.clone(),
                symbol_name.clone(),
                trade_side,
                delta_qty,
                avg_price,
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
                avg_price,
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
        tracing::info!(
            "체결 trace 저장: provider={} tr_id={} order_id={} request_id={}",
            trade_record.provider.as_deref().unwrap_or("unknown"),
            trade_record.provider_tr_id.as_deref().unwrap_or("-"),
            trade_record.provider_order_id.as_deref().unwrap_or("-"),
            trade_record.provider_request_id.as_deref().unwrap_or("-")
        );
        if let Err(e) = self.trade_store.append(trade_record).await {
            tracing::error!("TradeStore 저장 실패: {}", e);
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
                format!("${:.2}", avg_price as f64 / 100.0)
            } else {
                format!("{}원", avg_price)
            };
            let content = format!(
                "{} {} {}주 @ {}{}",
                symbol_name, side_str, delta_qty, price_text, pnl_str
            );
            let _ = discord.send(NotificationEvent::trade(content)).await;
        }

        Ok(())
    }

    /// 종목명으로 미체결 주문을 체결 처리 (시장가 주문 자동 확인용)
    ///
    /// 폴링 루프에서 주문 접수 후 다음 틱에 호출 — 시장가 주문은 즉시 체결 가정
    pub async fn confirm_fill_by_symbol(&mut self, symbol: &str, fill_price: u64) -> Result<()> {
        let ondo = match self.symbol_to_odno.get(symbol).cloned() {
            Some(o) => o,
            None => return Ok(()),
        };
        let qty = self
            .pending
            .get(&ondo)
            .map(|p| p.record.quantity)
            .unwrap_or(1);
        self.on_fill(&ondo, qty, fill_price).await
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

    fn pending_fill_candidates(&self) -> Vec<(BrokerId, String, String)> {
        self.pending
            .iter()
            .map(|(odno, order)| {
                (
                    pending_order_provider(order),
                    odno.clone(),
                    order.record.symbol.clone(),
                )
            })
            .collect()
    }

    async fn apply_pending_fills(&mut self, fills: Vec<PendingFill>) -> Result<()> {
        for fill in fills {
            self.on_fill(&fill.odno, fill.filled_qty, fill.avg_price)
                .await?;
        }
        Ok(())
    }
}

fn kis_pending_from(pending: &[(BrokerId, String, String)]) -> Vec<(String, String)> {
    pending
        .iter()
        .filter(|(broker_id, _, _)| *broker_id == BrokerId::Kis)
        .map(|(_, odno, symbol)| (odno.clone(), symbol.clone()))
        .collect()
}

fn toss_pending_from(pending: &[(BrokerId, String, String)]) -> Vec<(String, String)> {
    pending
        .iter()
        .filter(|(broker_id, _, _)| *broker_id == BrokerId::Toss)
        .map(|(_, order_id, symbol)| (order_id.clone(), symbol.clone()))
        .collect()
}

async fn collect_kis_pending_fills(
    client: Arc<KisRestClient>,
    pending: &[(String, String)],
) -> Result<Vec<PendingFill>> {
    let mut fills = Vec::new();

    if pending.iter().any(|(_, symbol)| is_domestic_symbol(symbol)) {
        let executed = client.get_today_executed_orders().await?;
        for (odno, symbol) in pending
            .iter()
            .filter(|(_, symbol)| is_domestic_symbol(symbol))
        {
            let Some(order) = executed.iter().find(|o| o.odno == *odno) else {
                continue;
            };
            let qty = order.tot_ccld_qty.parse::<u64>().unwrap_or(0);
            if qty == 0 {
                continue;
            }
            let amount = order.tot_ccld_amt.parse::<u64>().unwrap_or(0);
            let avg_price = if amount > 0 {
                amount / qty
            } else {
                order.ord_unpr.parse::<u64>().unwrap_or(0)
            };
            if avg_price == 0 {
                continue;
            }
            tracing::info!(
                "국내 주문번호 기반 체결 확인: odno={} symbol={} qty={} avg={}",
                odno,
                symbol,
                qty,
                avg_price
            );
            fills.push(PendingFill {
                odno: odno.clone(),
                filled_qty: qty,
                avg_price,
            });
        }
    }

    if pending
        .iter()
        .any(|(_, symbol)| !is_domestic_symbol(symbol))
    {
        let executed = client.get_today_overseas_executed_orders().await?;
        for (odno, symbol) in pending
            .iter()
            .filter(|(_, symbol)| !is_domestic_symbol(symbol))
        {
            let Some(order) = executed.iter().find(|o| o.odno == *odno) else {
                continue;
            };
            let qty = order.filled_qty();
            if qty == 0 {
                continue;
            }
            let avg_price_cents = order.avg_price_cents();
            if avg_price_cents == 0 {
                continue;
            }
            tracing::info!(
                "해외 주문번호 기반 체결 확인: odno={} symbol={} qty={} avg_cents={}",
                odno,
                symbol,
                qty,
                avg_price_cents
            );
            fills.push(PendingFill {
                odno: odno.clone(),
                filled_qty: qty,
                avg_price: avg_price_cents,
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
        tracing::warn!(
            "Toss pending 체결 확인 스킵 — 활성 Toss 프로파일이 없습니다: {}건",
            pending.len()
        );
        return Ok(Vec::new());
    };
    let account_seq = profile.broker_account_id();
    if account_seq.trim().is_empty() {
        tracing::warn!(
            "Toss pending 체결 확인 스킵 — accountSeq가 비어 있습니다: {}건",
            pending.len()
        );
        return Ok(Vec::new());
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(account_seq.clone()),
    );
    let mut fills = Vec::new();
    for (order_id, symbol) in pending {
        let order = match adapter.get_order(Some(&account_seq), order_id).await {
            Ok(order) => order,
            Err(e) => {
                tracing::warn!(
                    "Toss 주문번호 기반 체결 확인 실패: order_id={} symbol={} error={}",
                    order_id,
                    symbol,
                    e
                );
                continue;
            }
        };
        let qty = storage_quantity_units(&order.execution.filled_quantity);
        if qty == 0 {
            continue;
        }
        let Some(avg_price) = order.execution.average_filled_price.as_deref() else {
            continue;
        };
        let avg_units = storage_money_units(avg_price, &order.currency);
        if avg_units == 0 {
            continue;
        }
        tracing::info!(
            "Toss 주문번호 기반 체결 확인: order_id={} symbol={} status={} qty={} avg={}",
            order_id,
            symbol,
            order.status,
            qty,
            avg_units
        );
        fills.push(PendingFill {
            odno: order_id.clone(),
            filled_qty: qty,
            avg_price: avg_units,
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
