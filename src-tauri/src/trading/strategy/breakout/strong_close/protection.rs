//! Strict v1 StrongClose checkpoint; never reconstruct missing age or peak.
use super::*;
use serde::Deserialize;
use serde_json::{json, Value};
pub(super) fn encode(s: &StrongCloseState) -> Option<Value> {
    if s.owned_quantity == 0 && !s.entry_candidate {
        return None;
    }
    Some(
        json!({"version":1,"kind":"strongClose","ageKnown":s.held_observations.is_some(),"heldObservations":s.held_observations.unwrap_or(0),"candidateEntry":s.entry_candidate,"observedPeak":s.observed_peak,"profitArmed":s.profit_armed}),
    )
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Protection {
    version: u64,
    kind: String,
    age_known: bool,
    held_observations: u64,
    candidate_entry: bool,
    observed_peak: Option<u64>,
    profit_armed: bool,
}
pub(super) fn restore(s: &mut StrongCloseState, v: &Value) -> Result<(), String> {
    let error = || "Invalid strong close protection checkpoint identity or state".to_string();
    for key in [
        "version",
        "kind",
        "ageKnown",
        "heldObservations",
        "candidateEntry",
        "observedPeak",
        "profitArmed",
    ] {
        if v.get(key).is_none() {
            return Err(error());
        }
    }
    let p: Protection = serde_json::from_value(v.clone())
        .map_err(|e| format!("Invalid strong close protection checkpoint: {e}"))?;
    if p.version != 1
        || p.kind != "strongClose"
        || (!p.age_known && p.held_observations != 0)
        || p.observed_peak == Some(0)
        || (p.profit_armed && p.observed_peak.is_none())
        || (p.candidate_entry && (p.age_known || p.observed_peak.is_some() || p.profit_armed))
    {
        return Err(error());
    }
    if s.owned_quantity == 0
        && (p.age_known || !p.candidate_entry || p.observed_peak.is_some() || p.profit_armed)
    {
        return Err(error());
    }
    s.held_observations = if s.owned_quantity > 0 && p.candidate_entry {
        Some(0)
    } else {
        p.age_known.then_some(p.held_observations)
    };
    s.entry_candidate = s.owned_quantity == 0 && p.candidate_entry;
    s.observed_peak = p.observed_peak;
    s.profit_armed = p.profit_armed;
    Ok(())
}
