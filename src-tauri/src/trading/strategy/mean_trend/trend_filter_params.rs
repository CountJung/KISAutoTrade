//! 독립 raw 검증: 비정렬 MA 기간을 허용하며 수익 보호 쌍만 함께 활성화한다.
use super::super::observation::bounded_optional_integer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrendFilterParams {
    pub long_period: u32,
    pub short_period: u32,
    pub mid_period: u32,
    pub exit_confirm_observations: u64,
    pub profit_activation_pct: f64,
    pub profit_trailing_pct: f64,
    pub max_hold_observations: u64,
}
impl Default for TrendFilterParams {
    fn default() -> Self {
        Self {
            long_period: 200,
            short_period: 5,
            mid_period: 20,
            exit_confirm_observations: 1,
            profit_activation_pct: 0.0,
            profit_trailing_pct: 0.0,
            max_hold_observations: 0,
        }
    }
}
fn integer(v: &Value, key: &str, default: u64, min: u64, max: u64) -> Option<u64> {
    if v.get(key).is_none() {
        return Some(default);
    }
    bounded_optional_integer(v.get(key), max).filter(|n| *n >= min)
}
fn percent(v: &Value, key: &str, max: f64) -> Option<f64> {
    match v.get(key) {
        None => Some(0.0),
        Some(n) => n
            .as_f64()
            .filter(|n| n.is_finite() && (0.0..=max).contains(n)),
    }
}
pub fn validate_trend_filter_params(id: &str, v: &Value) -> Result<(), String> {
    if !id.starts_with("trend_filter") {
        return Ok(());
    }
    if !v.is_object() {
        return Err("추세 필터 설정은 객체여야 합니다.".into());
    }
    for (key, default, min, max) in [
        ("long_period", 200, 1, 512),
        ("short_period", 5, 1, 512),
        ("mid_period", 20, 1, 512),
        ("exit_confirm_observations", 1, 1, 1_000_000),
        ("max_hold_observations", 0, 0, 1_000_000),
    ] {
        if integer(v, key, default, min, max).is_none() {
            return Err(format!("{key}는 {min}..{max} 정수여야 합니다."));
        }
    }
    for (key, max) in [
        ("profit_activation_pct", 1000.0),
        ("profit_trailing_pct", 99.9),
    ] {
        if percent(v, key, max).is_none() {
            return Err(format!("{key}는 0..{max} 유한수여야 합니다."));
        }
    }
    if (percent(v, "profit_activation_pct", 1000.0).unwrap_or(0.0) > 0.0)
        != (percent(v, "profit_trailing_pct", 99.9).unwrap_or(0.0) > 0.0)
    {
        return Err("수익 보호 시작·되밀림은 모두 0 또는 모두 양수여야 합니다.".into());
    }
    Ok(())
}
pub(super) fn parse(v: &Value) -> TrendFilterParams {
    // 필드별 해석으로 손상된 새 값이 유효한 기존 기간·보호를 전체 기본값으로 덮지 않는다.
    TrendFilterParams {
        long_period: integer(v, "long_period", 200, 1, 512).unwrap_or(200) as u32,
        short_period: integer(v, "short_period", 5, 1, 512).unwrap_or(5) as u32,
        mid_period: integer(v, "mid_period", 20, 1, 512).unwrap_or(20) as u32,
        exit_confirm_observations: integer(v, "exit_confirm_observations", 1, 1, 1_000_000)
            .unwrap_or(1),
        profit_activation_pct: percent(v, "profit_activation_pct", 1000.0).unwrap_or(0.0),
        profit_trailing_pct: percent(v, "profit_trailing_pct", 99.9).unwrap_or(0.0),
        max_hold_observations: integer(v, "max_hold_observations", 0, 0, 1_000_000).unwrap_or(0),
    }
}
