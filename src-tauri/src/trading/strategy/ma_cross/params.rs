use super::super::observation::bounded_optional_integer;
use super::*;

/// Existing cross periods use native price observations; entry filters use DAILY context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaCrossParams {
    pub short_period: usize,
    pub long_period: usize,
    #[serde(default)]
    pub long_trend_period: usize,
    #[serde(default)]
    pub min_long_ma_slope_pct: f64,
    #[serde(default)]
    pub min_atr_pct: f64,
    #[serde(default)]
    pub min_bandwidth_pct: f64,
}
impl Default for MaCrossParams {
    fn default() -> Self {
        Self {
            short_period: 5,
            long_period: 20,
            long_trend_period: 0,
            min_long_ma_slope_pct: 0.0,
            min_atr_pct: 0.0,
            min_bandwidth_pct: 0.0,
        }
    }
}
const OPTIONAL: [&str; 4] = [
    "long_trend_period",
    "min_long_ma_slope_pct",
    "min_atr_pct",
    "min_bandwidth_pct",
];
fn finite(params: &serde_json::Value, key: &str, max: f64) -> Option<f64> {
    match params.get(key) {
        None => Some(0.0),
        Some(raw) => raw
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.0 && *v <= max),
    }
}
fn legacy_params(raw: &serde_json::Value) -> MaCrossParams {
    let mut legacy = raw.clone();
    if let Some(object) = legacy.as_object_mut() {
        for key in OPTIONAL {
            object.remove(key);
        }
    }
    serde_json::from_value(legacy).unwrap_or_default()
}
impl MaCrossParams {
    pub(super) fn parse(raw: &serde_json::Value) -> (Self, bool) {
        let valid = validate(raw).is_ok();
        let mut p = legacy_params(raw);
        p.long_trend_period =
            bounded_optional_integer(raw.get("long_trend_period"), 512).unwrap_or(0) as usize;
        p.min_long_ma_slope_pct = finite(raw, "min_long_ma_slope_pct", 100.0).unwrap_or(0.0);
        p.min_atr_pct = finite(raw, "min_atr_pct", 100.0).unwrap_or(0.0);
        p.min_bandwidth_pct = finite(raw, "min_bandwidth_pct", 1000.0).unwrap_or(0.0);
        (p, valid)
    }
    pub(super) fn enabled(&self) -> bool {
        self.long_trend_period > 0
            || self.min_long_ma_slope_pct > 0.0
            || self.min_atr_pct > 0.0
            || self.min_bandwidth_pct > 0.0
    }
    pub(super) fn required_completed(&self) -> usize {
        self.long_trend_period
            .max(if self.min_long_ma_slope_pct > 0.0 {
                self.long_period.saturating_add(1).min(512)
            } else {
                0
            })
            .max(if self.min_atr_pct > 0.0 { 15 } else { 0 })
            .max(if self.min_bandwidth_pct > 0.0 { 20 } else { 0 })
    }
}
fn validate(raw: &serde_json::Value) -> Result<(), String> {
    if bounded_optional_integer(raw.get("long_trend_period"), 512).is_none() {
        return Err("long_trend_period: 0~512 정수만 지원합니다.".into());
    }
    for (key, max) in [
        ("min_long_ma_slope_pct", 100.0),
        ("min_atr_pct", 100.0),
        ("min_bandwidth_pct", 1000.0),
    ] {
        if finite(raw, key, max).is_none() {
            return Err(format!("{key}: 0~{max} 유한수만 지원합니다."));
        }
    }
    if finite(raw, "min_long_ma_slope_pct", 100.0).is_some_and(|v| v > 0.0) {
        // Preserve the legacy all-or-default parser, but never validate a slope
        // against a silently substituted period. Missing legacy keys retain its default.
        let parsed = legacy_params(raw);
        for (key, actual) in [
            ("short_period", parsed.short_period),
            ("long_period", parsed.long_period),
        ] {
            if let Some(value) = raw.get(key) {
                if value.as_u64().and_then(|v| usize::try_from(v).ok()) != Some(actual) {
                    return Err(format!("min_long_ma_slope_pct 활성 시 {key}는 실제 MA 관측 기간과 일치하는 정수여야 합니다."));
                }
            }
        }
        if !(1..=511).contains(&parsed.long_period) {
            return Err("min_long_ma_slope_pct 활성 시 long_period는 1~511이어야 합니다.".into());
        }
    }
    Ok(())
}
pub fn validate_ma_cross_entry_params(id: &str, params: &serde_json::Value) -> Result<(), String> {
    if id.starts_with("ma_cross") {
        validate(params)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn strict_optional_boundaries_accept_integer_float_and_reject_bad_types() {
        for (key, max) in [
            ("long_trend_period", 512.0),
            ("min_long_ma_slope_pct", 100.0),
            ("min_atr_pct", 100.0),
            ("min_bandwidth_pct", 1000.0),
        ] {
            for value in [
                json!(null),
                json!("1"),
                json!(true),
                json!(-0.1),
                json!(max + 1.0),
            ] {
                let mut raw = json!({"short_period":1,"long_period":20});
                raw[key] = value;
                assert!(
                    validate_ma_cross_entry_params("ma_cross_test", &raw).is_err(),
                    "{raw}"
                );
                assert!(!MaCrossParams::parse(&raw).1);
            }
            let mut raw = json!({"short_period":1,"long_period":20});
            raw[key] = json!(max);
            assert!(validate_ma_cross_entry_params("ma_cross_test", &raw).is_ok());
            assert!(MaCrossParams::parse(&raw).1);
        }
        assert!(
            validate_ma_cross_entry_params("ma_cross", &json!({"long_trend_period":0.5})).is_err()
        );
        assert!(validate_ma_cross_entry_params("rsi", &json!({"long_trend_period":"bad"})).is_ok());
    }
    #[test]
    fn active_slope_bounds_legacy_long_period_and_required_context() {
        for period in [0, 512, usize::MAX] {
            let raw = json!({"short_period":2,"long_period":period,"min_long_ma_slope_pct":0.1});
            assert!(validate_ma_cross_entry_params("ma_cross", &raw).is_err());
            assert_eq!(MaCrossParams::parse(&raw).0.long_period, period);
        }
        for bad in [json!("512"), json!(512.5), json!(null), json!(20.0)] {
            assert!(validate_ma_cross_entry_params(
                "ma_cross",
                &json!({"short_period":2,"long_period":bad,"min_long_ma_slope_pct":0.1})
            )
            .is_err());
        }
        assert!(validate_ma_cross_entry_params(
            "ma_cross",
            &json!({"short_period":"bad","long_period":8,"min_long_ma_slope_pct":0.1})
        )
        .is_err());
        assert!(
            validate_ma_cross_entry_params("ma_cross", &json!({"min_long_ma_slope_pct":0.1}))
                .is_ok()
        );
        for period in [1, 511] {
            let (p, valid) = MaCrossParams::parse(
                &json!({"short_period":2,"long_period":period,"min_long_ma_slope_pct":0.1}),
            );
            assert!(valid);
            assert_eq!(p.required_completed(), period + 1);
        }
        let (p, valid) = MaCrossParams::parse(
            &json!({"short_period":2,"long_period":20,"long_trend_period":512.0,"min_long_ma_slope_pct":0.1,"min_atr_pct":0.5,"min_bandwidth_pct":2}),
        );
        assert!(valid);
        assert_eq!(p.required_completed(), 512);
        assert_eq!(p.long_trend_period, 512);
    }
}
