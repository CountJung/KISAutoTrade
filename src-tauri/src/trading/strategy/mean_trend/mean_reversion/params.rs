use super::super::super::observation::bounded_optional_integer as integer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MeanReversionExitMode {
    #[default]
    UpperBand,
    MiddleBand,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeanReversionParams {
    pub period: u32,
    pub std_dev: f64,
    /// Native entry-price loss percentage; zero retains the legacy >=0 exit.
    pub stop_loss_pct: f64,
    #[serde(default)]
    pub exit_mode: MeanReversionExitMode,
    /// Previous observations, excluding the current price; zero disables it.
    #[serde(default)]
    pub long_trend_period: usize,
    /// Owned observations since confirmed entry; zero disables only this limit.
    #[serde(default)]
    pub max_hold_observations: u64,
}

impl Default for MeanReversionParams {
    fn default() -> Self {
        Self {
            period: 20,
            std_dev: 2.0,
            stop_loss_pct: 5.0,
            exit_mode: MeanReversionExitMode::UpperBand,
            long_trend_period: 0,
            max_hold_observations: 0,
        }
    }
}

pub fn validate_mean_reversion_params(id: &str, value: &serde_json::Value) -> Result<(), String> {
    if !id.starts_with("mean_reversion") {
        return Ok(());
    }
    if value
        .get("exit_mode")
        .is_some_and(|v| !matches!(v.as_str(), Some("upperBand" | "middleBand")))
    {
        return Err("exit_mode는 upperBand 또는 middleBand이어야 합니다.".into());
    }
    for (key, max) in [
        ("long_trend_period", 512),
        ("max_hold_observations", 1_000_000),
    ] {
        if integer(value.get(key), max).is_none() {
            return Err(format!("{key}는 0..{max} 정수여야 합니다."));
        }
    }
    Ok(())
}

pub(super) fn parse(value: &serde_json::Value) -> MeanReversionParams {
    // A damaged optional field must not replace custom legacy bands or stop.
    let mut legacy = value.clone();
    if let Some(object) = legacy.as_object_mut() {
        for key in ["exit_mode", "long_trend_period", "max_hold_observations"] {
            object.remove(key);
        }
    }
    let mut p: MeanReversionParams = serde_json::from_value(legacy).unwrap_or_default();
    if value.get("exit_mode").and_then(serde_json::Value::as_str) == Some("middleBand") {
        p.exit_mode = MeanReversionExitMode::MiddleBand;
    }
    p.long_trend_period = integer(value.get("long_trend_period"), 512).unwrap_or(0) as usize;
    p.max_hold_observations = integer(value.get("max_hold_observations"), 1_000_000).unwrap_or(0);
    p
}
