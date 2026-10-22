//! Decode atomically; v1 anchors remain readable without inventing age/peak evidence.
use super::*;
use serde::Deserialize;
use serde_json::{json, Value};
pub(super) fn encode(s: &FailedBreakoutState) -> Option<Value> {
    if s.owned_quantity == 0 && s.candidate_prev_high.is_none() {
        return None;
    }
    Some(
        json!({"version":2,"kind":"failedBreakout","confirmedAnchor":s.breakout_prev_high,"candidateAnchor":s.candidate_prev_high,"ageKnown":s.held_observations.is_some(),"heldObservations":s.held_observations.unwrap_or(0),"observedPeak":s.observed_peak,"profitArmed":s.profit_armed}),
    )
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct V1 {
    version: u64,
    kind: String,
    confirmed_anchor: Option<u64>,
    candidate_anchor: Option<u64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct V2 {
    version: u64,
    kind: String,
    confirmed_anchor: Option<u64>,
    candidate_anchor: Option<u64>,
    age_known: bool,
    held_observations: u64,
    observed_peak: Option<u64>,
    profit_armed: bool,
}
pub(super) fn restore(s: &mut FailedBreakoutState, v: &Value) -> Result<(), String> {
    let error = || "Invalid breakout protection checkpoint identity or state".to_string();
    let (confirmed, candidate, age, peak, armed) =
        if v.get("version").and_then(Value::as_u64) == Some(1) {
            let p: V1 = serde_json::from_value(v.clone())
                .map_err(|e| format!("Invalid breakout protection checkpoint: {e}"))?;
            if p.version != 1
                || p.kind != "failedBreakout"
                || p.confirmed_anchor == Some(0)
                || p.candidate_anchor == Some(0)
                || (p.confirmed_anchor.is_none() && p.candidate_anchor.is_none())
            {
                return Err(error());
            }
            (p.confirmed_anchor, p.candidate_anchor, None, None, false)
        } else {
            for key in [
                "version",
                "kind",
                "confirmedAnchor",
                "candidateAnchor",
                "ageKnown",
                "heldObservations",
                "observedPeak",
                "profitArmed",
            ] {
                if v.get(key).is_none() {
                    return Err(error());
                }
            }
            let p: V2 = serde_json::from_value(v.clone())
                .map_err(|e| format!("Invalid breakout protection checkpoint: {e}"))?;
            if p.version != 2
                || p.kind != "failedBreakout"
                || p.confirmed_anchor == Some(0)
                || p.candidate_anchor == Some(0)
                || p.observed_peak == Some(0)
                || (!p.age_known && p.held_observations != 0)
                || (p.confirmed_anchor.is_some() && p.candidate_anchor.is_some())
                || (p.candidate_anchor.is_some()
                    && (p.age_known || p.observed_peak.is_some() || p.profit_armed))
                || (p.profit_armed && p.observed_peak.is_none())
            {
                return Err(error());
            }
            let age = if s.owned_quantity > 0 && p.candidate_anchor.is_some() {
                Some(0)
            } else {
                p.age_known.then_some(p.held_observations)
            };
            (
                p.confirmed_anchor,
                p.candidate_anchor,
                age,
                p.observed_peak,
                p.profit_armed,
            )
        };
    if s.owned_quantity > 0 {
        s.breakout_prev_high = confirmed.or(candidate);
        s.candidate_prev_high = None;
        s.held_observations = age;
        s.entry_candidate = false;
        s.observed_peak = peak;
        s.profit_armed = armed;
    } else {
        if confirmed.is_some() || candidate.is_none() || age.is_some() || peak.is_some() || armed {
            return Err(error());
        }
        s.candidate_prev_high = candidate;
        s.entry_candidate = true;
    }
    Ok(())
}
