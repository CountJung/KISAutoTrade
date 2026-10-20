use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviationEntryMode {
    #[default]
    Level,
    Crossing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviationParams {
    pub ma_period: usize,
    pub buy_threshold_pct: f64,
    pub sell_threshold_pct: f64,
    #[serde(default)]
    pub entry_mode: DeviationEntryMode,
    /// Prior observations at the evaluation cadence, not verified daily bars.
    #[serde(default)]
    pub long_trend_period: usize,
    #[serde(default)]
    pub max_hold_observations: u64,
    #[serde(default)]
    pub stop_loss_pct: f64,
}
impl Default for DeviationParams {
    fn default() -> Self {
        Self {
            ma_period: 20,
            buy_threshold_pct: -5.0,
            sell_threshold_pct: 5.0,
            entry_mode: DeviationEntryMode::Level,
            long_trend_period: 0,
            max_hold_observations: 0,
            stop_loss_pct: 0.0,
        }
    }
}
fn integer(value: Option<&serde_json::Value>, max: u64) -> Option<u64> {
    match value {
        None => Some(0),
        Some(v) => v
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 0.0 && *n <= max as f64 && n.fract() == 0.0)
            .map(|n| n as u64),
    }
}
pub fn validate_deviation_entry_params(id: &str, value: &serde_json::Value) -> Result<(), String> {
    if !id.starts_with("deviation") {
        return Ok(());
    }
    if value
        .get("entry_mode")
        .is_some_and(|v| !matches!(v.as_str(), Some("level" | "crossing")))
    {
        return Err("entry_mode는 level 또는 crossing이어야 합니다.".into());
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
            .is_some_and(|n| n.is_finite() && (0.0..=100.0).contains(&n))
    }) {
        return Err("stop_loss_pct는 0..100의 유한한 % 값이어야 합니다.".into());
    }
    Ok(())
}
pub(super) fn parse(value: &serde_json::Value) -> DeviationParams {
    // Malformed optional entry data cannot erase valid custom legacy exits.
    let mut legacy = value.clone();
    if let Some(o) = legacy.as_object_mut() {
        for key in [
            "entry_mode",
            "long_trend_period",
            "max_hold_observations",
            "stop_loss_pct",
        ] {
            o.remove(key);
        }
    }
    let mut p: DeviationParams = serde_json::from_value(legacy).unwrap_or_default();
    if value.get("entry_mode").and_then(serde_json::Value::as_str) == Some("crossing") {
        p.entry_mode = DeviationEntryMode::Crossing;
    }
    p.long_trend_period = integer(value.get("long_trend_period"), 512).unwrap_or(0) as usize;
    p.max_hold_observations = integer(value.get("max_hold_observations"), 1_000_000).unwrap_or(0);
    p.stop_loss_pct = value
        .get("stop_loss_pct")
        .and_then(serde_json::Value::as_f64)
        .filter(|n| n.is_finite() && (0.0..=100.0).contains(n))
        .unwrap_or(0.0);
    p
}
