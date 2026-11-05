//! Provider 체결 응답을 공통 누적 체결값으로 정규화한다.

use super::*;

mod notional;
use crate::broker::BrokerCurrency;
use notional::{execution_quantity, scaled_total, toss_amount};

pub(super) fn kis_pending_from(
    pending: &[(BrokerId, String, String, String)],
) -> Vec<(String, String, String)> {
    pending
        .iter()
        .filter(|(broker_id, _, _, _)| *broker_id == BrokerId::Kis)
        .map(|(_, odno, symbol, date)| (odno.clone(), symbol.clone(), date.clone()))
        .collect()
}

pub(super) fn toss_pending_from(
    pending: &[(BrokerId, String, String, String)],
) -> Vec<(String, String)> {
    pending
        .iter()
        .filter(|(broker_id, _, _, _)| *broker_id == BrokerId::Toss)
        .map(|(_, order_id, symbol, _)| (order_id.clone(), symbol.clone()))
        .collect()
}

pub(super) async fn collect_kis_pending_fills(
    client: Arc<KisRestClient>,
    pending: &[(String, String, String)],
) -> Result<Vec<PendingFill>> {
    let mut fills = Vec::new();
    let from = pending
        .iter()
        .map(|(_, _, date)| date.as_str())
        .min()
        .unwrap_or("");
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
            let qty = execution_quantity(&order.tot_ccld_qty)?;
            let terminal_status = domestic_order_terminal_status(order);
            if qty == 0 && terminal_status.is_none() {
                continue;
            }
            let amount = if qty > 0 {
                let amount =
                    OrderManager::parse_fill_notional(&order.tot_ccld_amt, BrokerCurrency::Krw)?;
                anyhow::ensure!(amount > 0, "KIS 국내 양수 체결 수량의 누적 금액이 0입니다");
                amount
            } else {
                0
            };
            let avg_price = if qty > 0 {
                u64::try_from(amount / u128::from(qty))?
            } else {
                0
            };
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
                cumulative_notional: Some(amount),
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
            let qty = execution_quantity(&order.ft_ccld_qty)?;
            let terminal_status = overseas_order_terminal_status(order);
            if qty == 0 && terminal_status.is_none() {
                continue;
            }
            anyhow::ensure!(
                qty == 0 || order.tr_crcy_cd.trim().eq_ignore_ascii_case("USD"),
                "KIS 해외 체결 응답의 거래 통화를 USD로 확인할 수 없습니다"
            );
            let amount = if qty == 0 {
                0
            } else if order.ft_ccld_amt3.trim().is_empty() {
                scaled_total(&order.ft_ccld_unpr3, BrokerCurrency::Usd, qty)?
            } else {
                OrderManager::parse_fill_notional(&order.ft_ccld_amt3, BrokerCurrency::Usd)?
            };
            anyhow::ensure!(
                qty == 0 || amount > 0,
                "KIS 해외 양수 체결 수량의 누적 금액이 0입니다"
            );
            let avg_price_cents = if qty > 0 {
                u64::try_from(amount / u128::from(qty))?
            } else {
                0
            };
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
                cumulative_notional: Some(amount),
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

pub(super) async fn collect_toss_pending_fills(
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
        anyhow::ensure!(
            order.order_id == *order_id && order.symbol.eq_ignore_ascii_case(symbol),
            "Toss 체결 응답의 주문번호 또는 종목이 조회 대상과 다릅니다"
        );
        let qty = execution_quantity(&order.execution.filled_quantity)?;
        let terminal_status = provider_terminal_status(order.status.as_str());
        if qty == 0 && terminal_status.is_none() {
            continue;
        }
        let amount = if qty > 0 {
            toss_amount(&order, symbol, qty)?
        } else {
            0
        };
        let avg_units = if qty > 0 {
            u64::try_from(amount / u128::from(qty))?
        } else {
            0
        };
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
            cumulative_notional: Some(amount),
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

pub(super) fn normalized_execution_u64(value: &str) -> u64 {
    value.trim().replace(',', "").parse::<u64>().unwrap_or(0)
}

pub(super) fn provider_terminal_status(status: &str) -> Option<OrderStatus> {
    match status.trim().to_ascii_uppercase().as_str() {
        "FILLED" => Some(OrderStatus::Filled),
        "CANCELED" | "CANCELLED" => Some(OrderStatus::Cancelled),
        "REJECTED" => Some(OrderStatus::Rejected),
        "EXPIRED" => Some(OrderStatus::Expired),
        "FAILED" => Some(OrderStatus::Failed),
        _ => None,
    }
}

pub(super) fn parse_provider_execution_date(value: &str) -> Option<chrono::NaiveDate> {
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
pub(super) fn domestic_order_is_terminal(order: &crate::api::rest::ExecutedOrder) -> bool {
    domestic_order_terminal_status(order).is_some()
}

pub(super) fn domestic_order_terminal_status(
    order: &crate::api::rest::ExecutedOrder,
) -> Option<OrderStatus> {
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

pub(super) fn domestic_provider_status(order: &crate::api::rest::ExecutedOrder) -> String {
    match domestic_order_terminal_status(order) {
        Some(OrderStatus::Filled) => "FILLED",
        Some(OrderStatus::Cancelled) => "CANCELLED",
        Some(OrderStatus::Rejected) => "REJECTED",
        Some(OrderStatus::Failed) => "FAILED",
        _ => "PENDING",
    }
    .to_string()
}

pub(super) fn overseas_order_terminal_status(
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
