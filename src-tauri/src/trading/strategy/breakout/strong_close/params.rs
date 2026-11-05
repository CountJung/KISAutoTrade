use super::super::super::observation::bounded_optional_integer;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrongCloseParams {
    pub threshold_pct: f64,
    pub stop_loss_pct: f64,
    #[serde(default)]
    pub long_trend_period: usize,
    #[serde(default)]
    pub profit_activation_pct: f64,
    #[serde(default)]
    pub profit_trailing_pct: f64,
    #[serde(default)]
    pub max_hold_observations: u64,
}
impl Default for StrongCloseParams {
    fn default() -> Self {
        Self {
            threshold_pct: 3.0,
            stop_loss_pct: 3.0,
            long_trend_period: 0,
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
pub fn validate_strong_close_params(id: &str, v: &serde_json::Value) -> Result<(), String> {
    if !id.starts_with("strong_close") {
        return Ok(());
    }
    for (key, max) in [
        ("long_trend_period", 512),
        ("max_hold_observations", 1_000_000),
    ] {
        if bounded_optional_integer(v.get(key), max).is_none() {
            return Err(format!("{key}는 0..{max} 정수여야 합니다."));
        }
    }
    for (key, max) in [
        ("profit_activation_pct", 1000.0),
        ("profit_trailing_pct", 99.9),
    ] {
        if number(v, key, max).is_none() {
            return Err(format!("{key}는 0..{max} 유한수여야 합니다."));
        }
    }
    if (number(v, "profit_activation_pct", 1000.0).unwrap_or(0.0) > 0.0)
        != (number(v, "profit_trailing_pct", 99.9).unwrap_or(0.0) > 0.0)
    {
        return Err("수익 무장과 되밀림 %는 모두 0 또는 모두 양수여야 합니다.".into());
    }
    Ok(())
}
pub(super) fn parse(v: &serde_json::Value) -> StrongCloseParams {
    let mut legacy = v.clone();
    if let Some(o) = legacy.as_object_mut() {
        for key in [
            "long_trend_period",
            "profit_activation_pct",
            "profit_trailing_pct",
            "max_hold_observations",
        ] {
            o.remove(key);
        }
    }
    let mut p: StrongCloseParams = serde_json::from_value(legacy).unwrap_or_default();
    p.long_trend_period =
        bounded_optional_integer(v.get("long_trend_period"), 512).unwrap_or(0) as usize;
    p.profit_activation_pct = number(v, "profit_activation_pct", 1000.0).unwrap_or(0.0);
    p.profit_trailing_pct = number(v, "profit_trailing_pct", 99.9).unwrap_or(0.0);
    p.max_hold_observations =
        bounded_optional_integer(v.get("max_hold_observations"), 1_000_000).unwrap_or(0);
    p
}
