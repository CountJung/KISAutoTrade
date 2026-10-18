//! Strict holdings conversions shared by publication callers.
use super::*;
use crate::api::rest::{BalanceResponse, OverseasBalanceResponse};
use crate::broker::{BrokerCurrency, BrokerHolding, BrokerMarket};
use std::collections::HashSet;

// u64::MAX as f64 rounds up to 2^64 and must never be an inclusive bound.
const U64_EXCLUSIVE: f64 = 18_446_744_073_709_551_616.0;

fn quantity(value: &str, symbol: &str) -> Result<u64> {
    value
        .trim()
        .replace(',', "")
        .parse()
        .map_err(|_| anyhow::anyhow!("{symbol} 보유수량 형식 오류"))
}
fn money(value: &str, multiplier: f64, symbol: &str) -> Result<u64> {
    let parsed: f64 = value
        .trim()
        .replace(',', "")
        .parse()
        .map_err(|_| anyhow::anyhow!("{symbol} 잔고 가격 형식 오류"))?;
    let scaled = parsed * multiplier;
    anyhow::ensure!(
        scaled.is_finite()
            && (0.0..U64_EXCLUSIVE).contains(&scaled)
            && scaled.round() < U64_EXCLUSIVE,
        "{symbol} 잔고 가격 범위 오류"
    );
    Ok(scaled.round() as u64)
}

impl HoldingsUpdate {
    pub(crate) fn from_toss(scope: &BrokerScope, holdings: &[BrokerHolding]) -> Result<Self> {
        anyhow::ensure!(
            scope.broker_id == BrokerId::Toss
                && scope
                    .account_id
                    .as_ref()
                    .is_some_and(|id| !id.0.trim().is_empty()),
            "Toss 보유 응답에 확정된 조회 계좌가 필요합니다"
        );
        let mut domestic = Vec::new();
        let mut overseas = Vec::new();
        let mut seen = HashSet::new();
        for holding in holdings {
            anyhow::ensure!(
                holding.broker == scope.broker_id
                    && holding.account_id.as_ref() == scope.account_id.as_ref(),
                "Toss 보유 응답의 broker 또는 계좌가 조회 범위와 다릅니다"
            );
            anyhow::ensure!(
                !holding.symbol.0.trim().is_empty()
                    && seen.insert((holding.market == BrokerMarket::Us, &holding.symbol.0)),
                "Toss 보유 응답 종목이 비어 있거나 같은 시장에 중복되었습니다"
            );
            let currency = if holding.market == BrokerMarket::Us {
                BrokerCurrency::Usd
            } else {
                BrokerCurrency::Krw
            };
            anyhow::ensure!(
                holding.average_price.currency == currency
                    && holding.current_price.currency == currency,
                "Toss 보유 응답의 시장과 가격 통화가 일치하지 않습니다"
            );
            let parsed: f64 = holding
                .quantity
                .0
                .trim()
                .replace(',', "")
                .parse()
                .map_err(|_| anyhow::anyhow!("{} Toss 보유수량 형식 오류", holding.symbol.0))?;
            anyhow::ensure!(
                parsed.is_finite() && (0.0..U64_EXCLUSIVE).contains(&parsed),
                "{} Toss 보유수량 범위 오류",
                holding.symbol.0
            );
            // Trackers and provider order APIs support whole shares. A fractional
            // residual cannot create a whole share or become sellable inventory.
            let quantity = parsed.floor() as u64;
            let multiplier = if currency == BrokerCurrency::Usd {
                100.0
            } else {
                1.0
            };
            let avg = money(&holding.average_price.amount, multiplier, &holding.symbol.0)?;
            let current = money(&holding.current_price.amount, multiplier, &holding.symbol.0)?;
            match holding.market {
                BrokerMarket::Kr => domestic.push((
                    holding.symbol.0.clone(),
                    holding.symbol_name.clone(),
                    quantity,
                    avg,
                    current,
                )),
                BrokerMarket::Us => overseas.push((
                    holding.symbol.0.clone(),
                    holding.symbol_name.clone(),
                    "TOSS_US".into(),
                    quantity,
                    avg,
                    current,
                )),
            }
        }
        Ok(Self::Both { domestic, overseas })
    }

    pub(crate) fn from_kis_domestic(response: &BalanceResponse) -> Result<Self> {
        let items = response
            .items
            .iter()
            .map(|item| {
                Ok((
                    item.pdno.clone(),
                    item.prdt_name.clone(),
                    quantity(&item.hldg_qty, &item.pdno)?,
                    money(&item.pchs_avg_pric, 1.0, &item.pdno)?,
                    money(&item.prpr, 1.0, &item.pdno)?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self::Domestic(items))
    }
    pub(crate) fn from_kis_overseas(response: &OverseasBalanceResponse) -> Result<Self> {
        let items = response
            .items
            .iter()
            .map(|item| {
                Ok((
                    item.ovrs_pdno.clone(),
                    item.ovrs_item_name.clone(),
                    normalize_overseas_order_exchange(&item.ovrs_excg_cd),
                    quantity(&item.ovrs_cblc_qty, &item.ovrs_pdno)?,
                    money(&item.pchs_avg_pric, 100.0, &item.ovrs_pdno)?,
                    money(&item.now_pric2, 100.0, &item.ovrs_pdno)?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self::Overseas(items))
    }
}

#[cfg(test)]
mod tests;
