use super::super::super::observation::bounded_optional_integer as integer;
use serde::{Deserialize, Serialize};

/// Simple-average RSI plus independently disabled entry and owned-exit policies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RsiParams {
    pub period: usize,
    pub oversold: f64,
    pub overbought: f64,
    #[serde(default)]
    pub long_trend_period: usize,
    #[serde(default)]
    pub stop_loss_pct: f64,
    #[serde(default)]
    pub max_hold_observations: u64,
}
impl Default for RsiParams {
    fn default() -> Self {
        Self {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            long_trend_period: 0,
            stop_loss_pct: 0.0,
            max_hold_observations: 0,
        }
    }
}

pub fn validate_rsi_params(id: &str, value: &serde_json::Value) -> Result<(), String> {
    if !id.starts_with("rsi") {
        return Ok(());
    }
    for (key, max) in [
        ("long_trend_period", 512),
        ("max_hold_observations", 1_000_000),
    ] {
        if integer(value.get(key), max).is_none() {
            return Err(format!("{key}는 0..{max} 정수여야 합니다."));
        }
    }
    if value.get("stop_loss_pct").is_some_and(|v| {
        !v.as_f64()
            .is_some_and(|n| n.is_finite() && (0.0..=99.9).contains(&n))
    }) {
        return Err("stop_loss_pct는 0..99.9의 유한한 % 값이어야 합니다.".into());
    }
    Ok(())
}

pub(super) fn parse(value: &serde_json::Value) -> RsiParams {
    // Broken optional data must not erase custom period/30/70 settings or valid protection.
    let mut legacy = value.clone();
    if let Some(object) = legacy.as_object_mut() {
        for key in [
            "long_trend_period",
            "stop_loss_pct",
            "max_hold_observations",
        ] {
            object.remove(key);
        }
    }
    let mut p: RsiParams = serde_json::from_value(legacy).unwrap_or_default();
    p.long_trend_period = integer(value.get("long_trend_period"), 512).unwrap_or(0) as usize;
    p.max_hold_observations = integer(value.get("max_hold_observations"), 1_000_000).unwrap_or(0);
    p.stop_loss_pct = value
        .get("stop_loss_pct")
        .and_then(serde_json::Value::as_f64)
        .filter(|n| n.is_finite() && (0.0..=99.9).contains(n))
        .unwrap_or(0.0);
    p
}
