use super::super::super::observation::bounded_optional_integer;
use serde::{Deserialize, Serialize};

/// Daily range expansion and optional confirmed-owner protection parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VolatilityExpansionParams {
    pub lookback_days: usize,
    pub expansion_factor: f64,
    pub stop_loss_pct: f64,
    pub profit_activation_pct: f64,
    pub profit_trailing_pct: f64,
    pub max_hold_observations: u64,
}
impl Default for VolatilityExpansionParams {
    fn default() -> Self {
        Self {
            lookback_days: 10,
            expansion_factor: 2.0,
            stop_loss_pct: 3.0,
            profit_activation_pct: 0.0,
            profit_trailing_pct: 0.0,
            max_hold_observations: 0,
        }
    }
}
fn number(v: &serde_json::Value, key: &str, default: f64, max: f64) -> Option<f64> {
    match v.get(key) {
        None => Some(default),
        Some(n) => n
            .as_f64()
            .filter(|n| n.is_finite() && (0.0..=max).contains(n)),
    }
}
fn lookback(v: &serde_json::Value) -> Option<usize> {
    if v.get("lookback_days").is_none() {
        return Some(10);
    }
    bounded_optional_integer(v.get("lookback_days"), 512)
        .filter(|n| *n > 0)
        .map(|n| n as usize)
}
pub fn validate_volatility_expansion_params(id: &str, v: &serde_json::Value) -> Result<(), String> {
    if !id.starts_with("volatility_expansion") {
        return Ok(());
    }
    if !v.is_object() {
        return Err("전략 설정은 객체여야 합니다.".into());
    }
    if lookback(v).is_none() {
        return Err("lookback_days는 1..512 정수여야 합니다.".into());
    }
    for (key, default, max) in [
        ("expansion_factor", 2.0, 1000.0),
        ("stop_loss_pct", 3.0, 99.9),
        ("profit_activation_pct", 0.0, 1000.0),
        ("profit_trailing_pct", 0.0, 99.9),
    ] {
        if number(v, key, default, max).is_none() {
            return Err(format!("{key}는 0..{max} 유한수여야 합니다."));
        }
    }
    if bounded_optional_integer(v.get("max_hold_observations"), 1_000_000).is_none() {
        return Err("max_hold_observations는 0..1000000 정수여야 합니다.".into());
    }
    if (number(v, "profit_activation_pct", 0.0, 1000.0).unwrap_or(0.0) > 0.0)
        != (number(v, "profit_trailing_pct", 0.0, 99.9).unwrap_or(0.0) > 0.0)
    {
        return Err("수익 무장과 되밀림 %는 모두 0 또는 모두 양수여야 합니다.".into());
    }
    Ok(())
}
pub(super) fn parse(v: &serde_json::Value) -> VolatilityExpansionParams {
    VolatilityExpansionParams {
        lookback_days: lookback(v).unwrap_or(10),
        expansion_factor: number(v, "expansion_factor", 2.0, 1000.0).unwrap_or(2.0),
        stop_loss_pct: number(v, "stop_loss_pct", 3.0, 99.9).unwrap_or(3.0),
        profit_activation_pct: number(v, "profit_activation_pct", 0.0, 1000.0).unwrap_or(0.0),
        profit_trailing_pct: number(v, "profit_trailing_pct", 0.0, 99.9).unwrap_or(0.0),
        max_hold_observations: bounded_optional_integer(v.get("max_hold_observations"), 1_000_000)
            .unwrap_or(0),
    }
}
