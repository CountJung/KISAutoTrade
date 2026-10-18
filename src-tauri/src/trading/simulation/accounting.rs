//! Research-only cash flows and attribution. Idle cash is KRW, never a USD book.
use super::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CostSource {
    #[default]
    Estimated,
    /// A user's manual attestation, not verification performed by this software.
    BrokerVerifiedManual,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DividendPriceBasis {
    #[default]
    Unverified,
    Unadjusted,
    DividendAdjusted,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FxScenarioPoint {
    pub bar_time: String,
    pub rate_krw: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DividendScenarioPoint {
    pub bar_time: String,
    pub amount_per_share: f64,
    pub withholding_tax_bps: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayAccountingView {
    pub realized_pnl_krw: f64,
    pub residual_unrealized_pnl_krw: f64,
    pub remaining_quantity: u64,
    pub remaining_cost_basis_krw: f64,
    pub remaining_buy_fee_krw: f64,
    pub realized_price_pnl_krw: f64,
    pub realized_fx_pnl_krw: f64,
    pub residual_price_pnl_krw: f64,
    pub residual_fx_pnl_krw: f64,
    pub total_fee_krw: f64,
    pub total_tax_krw: f64,
    pub total_slippage_krw: f64,
    pub dividend_gross_krw: f64,
    pub dividend_tax_krw: f64,
    pub dividend_net_krw: f64,
    pub terminal_liquidation_equity_krw: f64,
    pub terminal_liquidation_return_pct: f64,
    pub terminal_exit_fee_krw: f64,
    pub terminal_exit_tax_krw: f64,
    pub terminal_exit_slippage_krw: f64,
    pub average_invested_notional_krw: f64,
    pub average_invested_capital_pct: f64,
    pub average_invested_equity_pct: f64,
    pub peak_invested_capital_pct: f64,
    pub peak_invested_equity_pct: f64,
    pub realized_equity_sign_mismatch: bool,
    pub warnings: Vec<String>,
}

impl SimulationAssumptions {
    /// Validate against the caller's normalized evaluation labels, excluding history.
    /// Points apply at their own bar start and carry forward, never backwards.
    pub fn validate_scenarios(
        &self,
        is_overseas: bool,
        allowed_bar_times: &[String],
    ) -> Result<(), String> {
        if self.cost_note.chars().count() > 120 || self.cost_note.chars().any(char::is_control) {
            return Err("비용 출처 메모는 제어문자 없이 120자 이하여야 합니다.".into());
        }
        if self.cost_source == CostSource::BrokerVerifiedManual && self.cost_note.trim().is_empty()
        {
            return Err("브로커 비용 수동 확인에는 출처·확인 메모가 필요합니다.".into());
        }
        for bps in [self.buy_fee_bps, self.sell_fee_bps].into_iter().flatten() {
            if !bps.is_finite() || !(0.0..=1_000.0).contains(&bps) {
                return Err("방향별 수수료는 0~1000bps의 유한수여야 합니다.".into());
            }
        }
        if !self.minimum_fee_native.is_finite()
            || !(0.0..=100_000_000.0).contains(&self.minimum_fee_native)
        {
            return Err(
                "최소 수수료는 원화/달러 가격 단위의 0~100000000 유한수여야 합니다.".into(),
            );
        }
        if self.dividend_price_basis == DividendPriceBasis::DividendAdjusted
            && !self.dividend_scenario.is_empty()
        {
            return Err(
                "배당 조정 가격과 현금 배당 시나리오를 함께 사용하면 배당을 중복 계상합니다."
                    .into(),
            );
        }
        if !is_overseas && !self.fx_scenario.is_empty() {
            return Err("환율 시나리오는 해외 종목에서만 사용할 수 있습니다.".into());
        }
        let allowed: HashSet<&str> = allowed_bar_times.iter().map(String::as_str).collect();
        validate_labels(
            self.fx_scenario.iter().map(|point| point.bar_time.as_str()),
            &allowed,
        )?;
        validate_labels(
            self.dividend_scenario
                .iter()
                .map(|point| point.bar_time.as_str()),
            &allowed,
        )?;
        if self.fx_scenario.iter().any(|point| {
            !point.rate_krw.is_finite() || !(100.0..=10_000.0).contains(&point.rate_krw)
        }) {
            return Err("시나리오 환율은 100~10000원의 유한수여야 합니다.".into());
        }
        if self.dividend_scenario.iter().any(|point| {
            !point.amount_per_share.is_finite()
                || !(0.0..=100_000_000.0).contains(&point.amount_per_share)
                || !point.withholding_tax_bps.is_finite()
                || !(0.0..=10_000.0).contains(&point.withholding_tax_bps)
        }) {
            return Err(
                "배당 시나리오의 주당 금액 또는 원천징수 bps가 범위를 벗어났습니다.".into(),
            );
        }
        Ok(())
    }

    pub(super) fn fee(&self, amount_krw: f64, is_buy: bool, fx: f64) -> f64 {
        if amount_krw <= 0.0 {
            return 0.0;
        }
        let bps = if is_buy {
            self.buy_fee_bps
        } else {
            self.sell_fee_bps
        }
        .unwrap_or(self.fee_bps);
        bps_cost(amount_krw, bps).max(self.minimum_fee_native * fx)
    }
}

fn validate_labels<'a>(
    labels: impl Iterator<Item = &'a str>,
    allowed: &HashSet<&str>,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    for label in labels {
        if seen.len() >= 64
            || label.len() > 64
            || !valid_time(label)
            || !allowed.contains(label)
            || !seen.insert(label)
        {
            return Err("시나리오는 평가 봉 시각과 정확히 일치하는 유효·고유 시각을 최대 64개 사용해야 합니다.".into());
        }
    }
    Ok(())
}

fn valid_time(label: &str) -> bool {
    DateTime::parse_from_rfc3339(label).is_ok()
        || ["%Y-%m-%d", "%Y%m%d"]
            .iter()
            .any(|format| NaiveDate::parse_from_str(label, format).is_ok())
        || ["%Y%m%d%H%M", "%Y%m%d%H%M%S"]
            .iter()
            .any(|format| NaiveDateTime::parse_from_str(label, format).is_ok())
}
