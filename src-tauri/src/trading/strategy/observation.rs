//! Shared confirmed-ownership observation mechanics; strategy decisions stay local.
use serde::Deserialize;
use serde_json::Value;
use std::collections::VecDeque;

pub(super) fn bounded_optional_integer(value: Option<&serde_json::Value>, max: u64) -> Option<u64> {
    match value {
        None => Some(0),
        Some(v) => v
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 0.0 && *n <= max as f64 && n.fract() == 0.0)
            .map(|n| n as u64),
    }
}

/// Unknown restored ages remain unknown through further owned observations.
pub(super) fn advance_owned_age(quantity: u64, age: &mut Option<u64>) {
    if quantity > 0 {
        *age = age.map(|n| n.saturating_add(1));
    }
}

/// Apply confirmed feedback without changing ownership or average price itself.
pub(super) fn sync_owned_age(
    previous_qty: u64,
    qty: u64,
    age: &mut Option<u64>,
    candidate: &mut bool,
) {
    if qty > 0 {
        if previous_qty == 0 {
            *age = candidate.then_some(0);
        }
        *candidate = false;
    } else if previous_qty > 0 {
        *age = None;
        *candidate = false;
    }
}

/// Preserve the version-1 shape and the caller's strategy kind discriminator.
pub(super) fn encode_owned_observation(
    kind: &str,
    qty: u64,
    age: Option<u64>,
    candidate: bool,
) -> Option<Value> {
    if qty == 0 && !candidate {
        return None;
    }
    Some(
        serde_json::json!({"version":1,"kind":kind,"ageKnown":age.is_some(),"heldObservations":age.unwrap_or(0),"candidateEntry":candidate}),
    )
}

/// Fully validate before returning values; callers alone publish restored state.
pub(super) fn decode_owned_observation(
    expected_kind: &str,
    qty: u64,
    metadata: &Value,
) -> Result<(Option<u64>, bool), String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Protection {
        version: u64,
        kind: String,
        age_known: bool,
        held_observations: u64,
        candidate_entry: bool,
    }
    let label = expected_kind.replace('_', " ");
    let p: Protection = serde_json::from_value(metadata.clone())
        .map_err(|e| format!("Invalid {label} protection checkpoint: {e}"))?;
    if p.version != 1
        || p.kind != expected_kind
        || (!p.age_known && p.held_observations != 0)
        || (p.age_known && p.candidate_entry)
    {
        return Err(format!(
            "Invalid {label} protection checkpoint identity or age"
        ));
    }
    if qty > 0 {
        Ok((
            if p.candidate_entry {
                Some(0)
            } else {
                p.age_known.then_some(p.held_observations)
            },
            false,
        ))
    } else {
        if p.age_known || !p.candidate_entry {
            return Err(format!("Confirmed {label} age requires owned quantity"));
        }
        Ok((None, true))
    }
}

/// Evaluate previous observations before appending the current price.
pub(super) fn trend_allows(prices: &VecDeque<u64>, period: usize, price: u64) -> bool {
    if period == 0 {
        return true;
    }
    if prices.len() < period {
        return false;
    }
    let mean = prices
        .iter()
        .rev()
        .take(period)
        .map(|v| *v as f64)
        .sum::<f64>()
        / period as f64;
    mean > 0.0 && price as f64 > mean
}

/// Keep the same bounded cadence context even while its filter is disabled.
pub(super) fn observe_trend(prices: &mut VecDeque<u64>, observations: &mut u64, price: u64) {
    prices.push_back(price);
    while prices.len() > 512 {
        prices.pop_front();
    }
    *observations = observations.saturating_add(1);
}

/// Discard daily/minute seeds only on the first actual live observation.
pub(super) fn begin_live_trend(
    live_started: &mut bool,
    prices: &mut VecDeque<u64>,
    observations: &mut u64,
) {
    if !*live_started {
        *live_started = true;
        prices.clear();
        *observations = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn optional_counts_keep_omission_numeric_integer_and_range_contract() {
        assert_eq!(bounded_optional_integer(None, 512), Some(0));
        for value in [json!(0), json!(512), json!(512.0)] {
            assert!(bounded_optional_integer(Some(&value), 512).is_some());
        }
        for value in [
            json!(null),
            json!("1"),
            json!(false),
            json!(-1),
            json!(0.5),
            json!(513),
        ] {
            assert_eq!(bounded_optional_integer(Some(&value), 512), None);
        }
    }

    #[test]
    fn age_preserves_pending_partial_same_quantity_and_unknown_origins() {
        let (mut age, mut candidate) = (None, true);
        sync_owned_age(0, 0, &mut age, &mut candidate);
        assert_eq!((age, candidate), (None, true));
        advance_owned_age(0, &mut age);
        sync_owned_age(0, 5, &mut age, &mut candidate);
        assert_eq!((age, candidate), (Some(0), false));
        advance_owned_age(5, &mut age);
        sync_owned_age(5, 5, &mut age, &mut candidate);
        sync_owned_age(5, 2, &mut age, &mut candidate);
        assert_eq!(age, Some(1));
        age = Some(u64::MAX);
        advance_owned_age(2, &mut age);
        assert_eq!(age, Some(u64::MAX));
        sync_owned_age(2, 0, &mut age, &mut candidate);
        sync_owned_age(0, 5, &mut age, &mut candidate);
        advance_owned_age(5, &mut age);
        assert_eq!((age, candidate), (None, false));
    }

    #[test]
    fn both_kinds_preserve_exact_checkpoint_shape_and_pending_fill_origin() {
        for kind in ["deviation", "mean_reversion"] {
            assert_eq!(encode_owned_observation(kind, 0, None, false), None);
            let pending = encode_owned_observation(kind, 0, None, true).unwrap();
            assert_eq!(
                pending,
                json!({"version":1,"kind":kind,"ageKnown":false,"heldObservations":0,"candidateEntry":true})
            );
            assert_eq!(
                decode_owned_observation(kind, 0, &pending),
                Ok((None, true))
            );
            assert_eq!(
                decode_owned_observation(kind, 3, &pending),
                Ok((Some(0), false))
            );
            let held = encode_owned_observation(kind, 3, Some(42), false).unwrap();
            assert_eq!(
                decode_owned_observation(kind, 3, &held),
                Ok((Some(42), false))
            );
            assert!(decode_owned_observation(kind, 0, &held).is_err());
            let unknown = encode_owned_observation(kind, 3, None, false).unwrap();
            assert_eq!(
                decode_owned_observation(kind, 3, &unknown),
                Ok((None, false))
            );
        }
    }

    #[test]
    fn checkpoints_reject_cross_kind_unknown_fields_and_contradictions() {
        let valid = encode_owned_observation("deviation", 3, Some(42), false).unwrap();
        assert!(decode_owned_observation("mean_reversion", 3, &valid).is_err());
        for patch in [
            json!({"version":2}),
            json!({"extra":0}),
            json!({"ageKnown":false}),
            json!({"candidateEntry":true}),
            json!({"heldObservations":0.5}),
            json!({"ageKnown":"true"}),
            json!({"heldObservations":-1}),
        ] {
            let mut malformed = valid.clone();
            malformed
                .as_object_mut()
                .unwrap()
                .extend(patch.as_object().unwrap().clone());
            assert!(decode_owned_observation("deviation", 3, &malformed).is_err());
        }
    }

    #[test]
    fn prior_trend_and_first_live_clear_remain_bounded_and_saturating() {
        let mut prices = VecDeque::from([1, 100, 100]);
        assert!(trend_allows(&prices, 0, 0));
        assert!(!trend_allows(&prices, 4, 101));
        assert!(!trend_allows(&prices, 2, 100));
        assert!(trend_allows(&prices, 2, 101));
        let (mut observations, mut live_started) = (3, false);
        begin_live_trend(&mut live_started, &mut prices, &mut observations);
        assert!(prices.is_empty());
        assert_eq!(observations, 0);
        observe_trend(&mut prices, &mut observations, 7);
        begin_live_trend(&mut live_started, &mut prices, &mut observations);
        assert_eq!((prices.len(), observations), (1, 1));
        observations = u64::MAX;
        for price in 0..600 {
            observe_trend(&mut prices, &mut observations, price);
        }
        assert_eq!((prices.len(), observations), (512, u64::MAX));
        assert_eq!(prices.front(), Some(&88));
    }
}
