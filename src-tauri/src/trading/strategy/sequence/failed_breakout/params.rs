use super::super::super::observation::bounded_optional_integer;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FailedBreakoutBasis {
    #[default]
    PriceObservations,
    CompletedHigh,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailedBreakoutParams {
    pub lookback_days: usize,
    pub buffer_pct: f64,
    #[serde(default)]
    pub breakout_basis: FailedBreakoutBasis,
    #[serde(default)]
    pub atr_buffer_multiplier: f64,
    #[serde(default)]
    pub min_volume_ratio: f64,
    #[serde(default)]
    pub profit_activation_pct: f64,
    #[serde(default)]
    pub profit_trailing_pct: f64,
    #[serde(default)]
    pub max_hold_observations: u64,
}
impl Default for FailedBreakoutParams {
    fn default() -> Self {
        Self {
            lookback_days: 20,
            buffer_pct: 0.5,
            breakout_basis: FailedBreakoutBasis::PriceObservations,
            atr_buffer_multiplier: 0.0,
            min_volume_ratio: 0.0,
            profit_activation_pct: 0.0,
            profit_trailing_pct: 0.0,
            max_hold_observations: 0,
        }
    }
}
fn number(v: &serde_json::Value, key: &str, max: f64) -> Option<f64> {
    match v.get(key) {
        None => Some(0.0),
        Some(n) => n
            .as_f64()
            .filter(|n| n.is_finite() && (0.0..=max).contains(n)),
    }
}
pub fn validate_failed_breakout_params(id: &str, v: &serde_json::Value) -> Result<(), String> {
    if !id.starts_with("failed_breakout") {
        return Ok(());
    }
    if v.get("breakout_basis")
        .is_some_and(|v| !matches!(v.as_str(), Some("priceObservations" | "completedHigh")))
    {
        return Err("breakout_basis는 priceObservations 또는 completedHigh여야 합니다.".into());
    }
    for (key, max) in [
        ("atr_buffer_multiplier", 20.0),
        ("min_volume_ratio", 100.0),
        ("profit_activation_pct", 1000.0),
        ("profit_trailing_pct", 99.9),
    ] {
        if number(v, key, max).is_none() {
            return Err(format!("{key}는 0..{max} 유한수여야 합니다."));
        }
    }
    if bounded_optional_integer(v.get("max_hold_observations"), 1_000_000).is_none() {
        return Err("max_hold_observations는 0..1000000 정수여야 합니다.".into());
    }
    if (number(v, "profit_activation_pct", 1000.0).unwrap_or(0.0) > 0.0)
        != (number(v, "profit_trailing_pct", 99.9).unwrap_or(0.0) > 0.0)
    {
        return Err("수익 무장과 되밀림 %는 모두 0 또는 모두 양수여야 합니다.".into());
    }
    Ok(())
}
pub(super) fn parse(v: &serde_json::Value) -> FailedBreakoutParams {
    let mut legacy = v.clone();
    if let Some(o) = legacy.as_object_mut() {
        for key in [
            "breakout_basis",
            "atr_buffer_multiplier",
            "min_volume_ratio",
            "profit_activation_pct",
            "profit_trailing_pct",
            "max_hold_observations",
        ] {
            o.remove(key);
        }
    }
    let mut p: FailedBreakoutParams = serde_json::from_value(legacy).unwrap_or_default();
    p.breakout_basis = v
        .get("breakout_basis")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    p.atr_buffer_multiplier = number(v, "atr_buffer_multiplier", 20.0).unwrap_or(0.0);
    p.min_volume_ratio = number(v, "min_volume_ratio", 100.0).unwrap_or(0.0);
    p.profit_activation_pct = number(v, "profit_activation_pct", 1000.0).unwrap_or(0.0);
    p.profit_trailing_pct = number(v, "profit_trailing_pct", 99.9).unwrap_or(0.0);
    p.max_hold_observations =
        bounded_optional_integer(v.get("max_hold_observations"), 1_000_000).unwrap_or(0);
    p
}
impl FailedBreakoutParams {
    pub(super) fn required_completed(&self) -> usize {
        (if self.breakout_basis == FailedBreakoutBasis::CompletedHigh {
            self.lookback_days.clamp(1, 512)
        } else {
            0
        })
        .max(if self.atr_buffer_multiplier > 0.0 {
            15
        } else {
            0
        })
        .max(if self.min_volume_ratio > 0.0 { 21 } else { 0 })
    }
}
