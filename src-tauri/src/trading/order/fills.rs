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

mod collector;
mod ownership;
mod provider_mode;
mod reconciliation;
#[cfg(test)]
use collector::{
    domestic_order_is_terminal, domestic_order_terminal_status, provider_terminal_status,
};

struct PendingFill {
    odno: String,
    filled_qty: u64,
    avg_price: u64,
    cumulative_notional: Option<u128>,
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
        self.on_fill_confirmed(odno, filled_qty, avg_price, None, execution_date)
            .await
    }

    /// Exact provider consideration in native KRW / USD cents. Average price is
    /// display-only when this consideration is present.
    pub(crate) async fn on_fill_confirmed(
        &mut self,
        odno: &str,
        filled_qty: u64,
        avg_price: u64,
        cumulative_notional: Option<u128>,
        execution_date: chrono::NaiveDate,
    ) -> Result<()> {
        let Some(pending) = self.pending.get(odno).cloned() else {
            tracing::debug!("on_fill: odno {} 는 미체결 풀에 없음 (이미 처리됨)", odno);
            return Ok(());
        };
        if pending.broker_scope != self.execution_scope {
            tracing::debug!("다른 실행 계좌의 체결을 보류합니다: odno={}", odno);
            return Ok(());
        }
        if let Err(error) = self.validate_pending_fill_mode(&pending).await {
            self.mark_reconciliation(Some(error.to_string()));
            return Err(error);
        }

        let execution_date = if pending.application_started {
            match pending.application_execution_date.as_deref() {
                Some(date) => match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                    Ok(parsed) if parsed.to_string() == date => parsed,
                    _ => {
                        self.block_for_persistence_failure(
                            "체결 intent의 실행일을 확인할 수 없습니다.".into(),
                        );
                        anyhow::bail!("체결 intent의 실행일을 확인할 수 없습니다.");
                    }
                },
                None => {
                    self.block_for_persistence_failure(
                        "진행 중인 체결 intent의 실행일이 없습니다.".into(),
                    );
                    anyhow::bail!("진행 중인 체결 intent의 실행일이 없습니다.");
                }
            }
        } else {
            execution_date
        };
        let symbol = pending.record.symbol.clone();
        let symbol_name = pending.record.symbol_name.clone();
        let order_quantity = pending.record.quantity;
        let confirmed_filled = if pending.application_started {
            pending.confirmed_filled_quantity
        } else {
            filled_qty.max(pending.confirmed_filled_quantity)
        }
        .min(order_quantity);
        let confirmed_avg_price = if pending.application_started {
            pending.confirmed_avg_price
        } else if filled_qty >= pending.confirmed_filled_quantity {
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
        let cumulative_notional = if pending.application_started {
            match pending.confirmed_filled_notional {
                Some(notional) => notional,
                None => {
                    self.block_for_persistence_failure(
                        "진행 중인 체결 intent의 누적 금액이 없습니다.".into(),
                    );
                    anyhow::bail!("진행 중인 체결 intent의 누적 금액이 없습니다.");
                }
            }
        } else if filled_qty >= pending.confirmed_filled_quantity {
            if cumulative_notional.is_some() && filled_qty > order_quantity {
                self.block_for_persistence_failure(
                    "확정 체결수량이 주문수량을 초과했습니다.".into(),
                );
                anyhow::bail!("확정 체결수량이 주문수량을 초과했습니다.");
            }
            cumulative_notional
                .unwrap_or(u128::from(confirmed_avg_price) * u128::from(cumulative_filled))
        } else {
            pending
                .confirmed_filled_notional
                .unwrap_or(u128::from(confirmed_avg_price) * u128::from(cumulative_filled))
        };
        let Some(delta_notional) = cumulative_notional
            .checked_sub(pending.filled_notional)
            .filter(|amount| *amount > 0)
        else {
            self.block_for_persistence_failure(
                "증가분 체결 금액이 없거나 누적 금액이 감소했습니다.".into(),
            );
            anyhow::bail!("증가분 체결 금액이 없거나 누적 금액이 감소했습니다.");
        };
        let delta_notional_units = match u64::try_from(delta_notional) {
            Ok(amount) => amount,
            Err(error) => {
                self.block_for_persistence_failure(
                    "증가분 체결 금액이 저장 범위를 초과했습니다.".into(),
                );
                return Err(error.into());
            }
        };
        let confirmed_avg_price =
            u64::try_from(cumulative_notional / u128::from(cumulative_filled))?;
        let fill_price = delta_notional_units / delta_qty;
        if fill_price == 0 || delta_notional_units > i64::MAX as u64 {
            self.block_for_persistence_failure(
                "증가분 체결가 또는 금액 범위를 확인할 수 없습니다.".into(),
            );
            anyhow::bail!("증가분 체결가 또는 금액 범위를 확인할 수 없습니다.");
        }

        let is_sell = matches!(pending.record.side, OrderSide::Sell);
        let is_overseas = pending.exchange.is_some() || !is_domestic_symbol(&symbol);
        let first_application = !pending.application_started;
        let exchange_rate = if is_overseas {
            if pending.application_started {
                match pending.application_exchange_rate {
                    Some(rate) => rate,
                    None => {
                        self.block_for_persistence_failure(
                            "진행 중인 해외 체결 intent의 환율이 없습니다.".into(),
                        );
                        anyhow::bail!("진행 중인 해외 체결 intent의 환율이 없습니다.");
                    }
                }
            } else {
                *self.exchange_rate_krw.read().await
            }
        } else {
            1.0
        };
        if !exchange_rate.is_finite() || exchange_rate <= 0.0 {
            self.block_for_persistence_failure("체결 intent의 환율을 확인할 수 없습니다.".into());
            anyhow::bail!("체결 intent의 환율을 확인할 수 없습니다.");
        }

        self.holdings_refresh_required |= if is_overseas { 2 } else { 1 };
        // A whole broker snapshot may already include this fill. Invalidate older
        // queries, but never add/subtract the same execution from account totals.
        if is_overseas {
            self.overseas_position_tracker
                .lock()
                .await
                .invalidate_snapshot();
        } else {
            self.position_tracker.lock().await.invalidate_snapshot();
        }
        let prepared = self
            .prepare_confirmed_ownership(
                &pending,
                cumulative_filled,
                cumulative_notional,
                delta_qty,
                is_overseas,
            )
            .await?;
        let pnl = prepared.pnl;
        if let Some(current) = self.pending.get_mut(odno) {
            current.confirmed_filled_quantity = cumulative_filled;
            current.confirmed_avg_price = confirmed_avg_price;
            current.confirmed_filled_notional = Some(cumulative_notional);
            current.application_started = true;
            current.application_execution_date = Some(execution_date.to_string());
            current.application_exchange_rate = Some(exchange_rate);
            current.application_pnl = Some(pnl);
            current.provider_status = Some("applying_fill".to_string());
        }
        if let Err(error) = self.persist_pending_orders().await {
            self.block_for_persistence_failure(format!("체결 intent 영속화 실패: {error}"));
            return Err(error);
        }

        if let Some(candidate) = prepared.candidate {
            self.save_ownership_ledger(&candidate).await?;
        }

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
            let converted = ((pnl as f64 / 100.0) * exchange_rate).round();
            if !converted.is_finite()
                || !(-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&converted)
            {
                self.block_for_persistence_failure(
                    "체결 손익의 원화 환산 범위를 초과했습니다.".into(),
                );
                anyhow::bail!("체결 손익의 원화 환산 범위를 초과했습니다.");
            }
            converted as i64
        } else {
            pnl
        };
        let mut fill_event_id = format!(
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
        // Provider IDs may be reused across paper/live orders on the same account.
        // New owned orders have a durable unique record ID. Keep legacy IDs intact
        // so a pre-migration application intent retries the original event.
        if pending.ownership_key.is_some() {
            fill_event_id.push(':');
            fill_event_id.push_str(&pending.record.id);
        }

        if let Err(error) = self
            .apply_budget_fill(&pending, cumulative_filled, cumulative_notional)
            .await
        {
            self.block_for_persistence_failure(format!("자동매매 예산 체결 반영 실패: {error}"));
            return Err(error);
        }

        if is_sell {
            if first_application && self.buy_suspended && !self.persistence_blocked {
                self.buy_suspended = false;
                self.buy_suspended_reason = None;
                tracing::info!(
                    "매도 체결로 자본 확보 — 잔고 부족 매수 정지 해제: {}",
                    symbol
                );
            }
            let risk_snapshot = {
                let mut risk = self.risk_manager.lock().await;
                if let Err(error) = risk.record_fill_pnl_once(
                    execution_date,
                    &fill_event_id,
                    &pending.broker_scope,
                    pending.strategy_id.as_deref().unwrap_or("unknown"),
                    &symbol,
                    pnl_krw,
                ) {
                    drop(risk);
                    self.block_for_persistence_failure(format!("체결 리스크 반영 실패: {error}"));
                    return Err(error);
                }
                risk.runtime_state()
            };
            if let Err(error) = self.risk_store.save_runtime(&risk_snapshot).await {
                self.block_for_persistence_failure(format!("체결 리스크 영속화 실패: {error}"));
                return Err(error);
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
        trade_record.total_amount = delta_notional_units;
        if is_overseas {
            let usd = delta_notional_units as f64 / 100.0;
            trade_record.total_amount_usd = Some(usd);
            let converted = (usd * exchange_rate).round();
            anyhow::ensure!(
                converted.is_finite() && converted < 9_223_372_036_854_775_808.0,
                "체결 원화 환산 금액 범위 오류"
            );
            trade_record.total_amount_krw = Some(converted as i64);
        } else {
            trade_record.total_amount_krw = Some(i64::try_from(delta_notional_units)?);
        }
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
        if is_complete {
            self.release_budget(&pending).await?;
        }
        let applying = self.pending.get(odno).cloned();
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
            current.application_execution_date = None;
            current.application_exchange_rate = None;
            current.application_pnl = None;
        }
        if let Err(error) = self.persist_pending_orders().await {
            if let Some(applying) = applying {
                self.pending.insert(odno.to_string(), applying);
                self.symbol_to_odno.insert(symbol.clone(), odno.to_string());
            }
            self.block_for_persistence_failure(format!("체결 watermark 영속화 실패: {error}"));
            return Err(error);
        }

        if is_complete {
            // All effects and pending removal are durable before relinquishing
            // this exact receipt. A crash before this step only retains evidence.
            self.prune_completed_ownership_receipt(
                &pending,
                cumulative_filled,
                cumulative_notional,
            )
            .await?;
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

#[cfg(test)]
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
