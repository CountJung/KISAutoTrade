use super::*;
use crate::broker::{BrokerAccountId, BrokerId};

fn key() -> OwnershipKey {
    OwnershipKey {
        identity: OwnershipIdentity {
            scope: BrokerScope::new(
                BrokerId::Kis,
                Some(BrokerAccountId("synthetic-account".into())),
            ),
            profile_id: Some("synthetic-profile".into()),
            is_paper: true,
        },
        owner: OwnershipOwner::Strategy("strategy-a".into()),
        market: BrokerMarket::Kr,
        symbol: "005930".into(),
    }
}

fn fill(id: &str, side: OrderSide, quantity: u64, notional: u128) -> OwnershipFill {
    OwnershipFill {
        order_record_id: id.into(),
        key: key(),
        side,
        ordered_quantity: quantity,
        cumulative_quantity: Some(quantity),
        cumulative_notional: Some(notional),
    }
}

fn bytes(ledger: &OwnershipLedger) -> String {
    serde_json::to_string(ledger).unwrap()
}

fn rejected_without_mutation(ledger: &mut OwnershipLedger, fill: &OwnershipFill) {
    let before = bytes(ledger);
    assert!(ledger.apply_cumulative(fill).is_err());
    assert_eq!(bytes(ledger), before);
}

#[test]
fn cumulative_buys_and_sells_track_native_weighted_cost_and_proportional_profit() {
    let mut ledger = OwnershipLedger::default();
    let mut buy = fill("buy", OrderSide::Buy, 2, 200);
    buy.ordered_quantity = 4;
    assert_eq!(
        ledger.apply_cumulative(&buy).unwrap().position,
        OwnershipPosition {
            quantity: 2,
            cost_notional: 200
        }
    );
    buy.cumulative_quantity = Some(4);
    buy.cumulative_notional = Some(440);
    let applied = ledger.apply_cumulative(&buy).unwrap();
    assert_eq!((applied.delta_quantity, applied.delta_notional), (2, 240));
    ledger
        .apply_cumulative(&fill("second-buy", OrderSide::Buy, 2, 260))
        .unwrap();
    assert_eq!(
        ledger.position(&key()),
        OwnershipPosition {
            quantity: 6,
            cost_notional: 700
        }
    );
    let mut sell = fill("sell", OrderSide::Sell, 2, 300);
    sell.ordered_quantity = 4;
    let first = ledger.apply_cumulative(&sell).unwrap();
    assert_eq!((first.released_cost, first.realized_pnl), (233, 67));
    sell.cumulative_quantity = Some(4);
    sell.cumulative_notional = Some(650);
    let second = ledger.apply_cumulative(&sell).unwrap();
    assert_eq!(
        (
            second.delta_quantity,
            second.delta_notional,
            second.released_cost,
            second.realized_pnl
        ),
        (2, 350, 233, 117)
    );
    assert_eq!(
        second.position,
        OwnershipPosition {
            quantity: 2,
            cost_notional: 234
        }
    );
    let final_sale = ledger
        .apply_cumulative(&fill("final-sale", OrderSide::Sell, 2, 300))
        .unwrap();
    assert_eq!(
        (final_sale.released_cost, final_sale.realized_pnl),
        (234, 66)
    );
    assert_eq!(final_sale.position, OwnershipPosition::default());
    assert_eq!(ledger.positions().count(), 0);
    assert_eq!(ledger.watermarks.len(), 4);
}

#[test]
fn duplicate_and_regressing_cumulative_reports_are_noops_but_contradictions_fail() {
    let mut ledger = OwnershipLedger::default();
    let mut buy = fill("buy", OrderSide::Buy, 2, 200);
    buy.ordered_quantity = 4;
    ledger.apply_cumulative(&buy).unwrap();
    let before = bytes(&ledger);
    for (quantity, notional) in [(2, 200), (1, 100), (0, 0), (2, 150)] {
        buy.cumulative_quantity = Some(quantity);
        buy.cumulative_notional = Some(notional);
        assert_eq!(ledger.apply_cumulative(&buy).unwrap().delta_quantity, 0);
        assert_eq!(bytes(&ledger), before);
    }
    for (quantity, notional) in [(1, 250), (3, 200), (2, 201), (5, 500)] {
        buy.cumulative_quantity = Some(quantity);
        buy.cumulative_notional = Some(notional);
        rejected_without_mutation(&mut ledger, &buy);
    }
}

#[test]
fn exact_account_profile_mode_owner_market_and_symbol_are_separate_and_orders_cannot_rebind() {
    let mut ledger = OwnershipLedger::default();
    let original = fill("original", OrderSide::Buy, 1, 100);
    ledger.apply_cumulative(&original).unwrap();
    let mut alternatives = Vec::new();
    let mut changed = key();
    changed.identity.scope.broker_id = BrokerId::Toss;
    alternatives.push(changed);
    let mut changed = key();
    changed.identity.scope.account_id = Some(BrokerAccountId("other-account".into()));
    alternatives.push(changed);
    let mut changed = key();
    changed.identity.profile_id = Some("other-profile".into());
    alternatives.push(changed);
    let mut changed = key();
    changed.identity.profile_id = None;
    alternatives.push(changed);
    let mut changed = key();
    changed.identity.is_paper = false;
    alternatives.push(changed);
    let mut changed = key();
    changed.owner = OwnershipOwner::Strategy("strategy-b".into());
    alternatives.push(changed);
    let mut changed = key();
    changed.owner = OwnershipOwner::Manual;
    alternatives.push(changed);
    let mut changed = key();
    changed.market = BrokerMarket::Us;
    alternatives.push(changed);
    let mut changed = key();
    changed.symbol = "AAPL".into();
    alternatives.push(changed);
    for (index, alternate) in alternatives.into_iter().enumerate() {
        assert_eq!(ledger.position(&alternate), OwnershipPosition::default());
        let mut changed = original.clone();
        changed.key = alternate;
        rejected_without_mutation(&mut ledger, &changed);
        changed.order_record_id = format!("different-{index}");
        ledger.apply_cumulative(&changed).unwrap();
        assert_eq!(ledger.position(&changed.key).quantity, 1);
    }
    assert_eq!(ledger.position(&key()).quantity, 1);
    let mut changed = original.clone();
    changed.side = OrderSide::Sell;
    rejected_without_mutation(&mut ledger, &changed);
    let mut changed = original;
    changed.ordered_quantity = 2;
    rejected_without_mutation(&mut ledger, &changed);
}

#[test]
fn unknown_quantities_and_costs_and_invalid_identities_never_assume_zero_basis() {
    let mut ledger = OwnershipLedger::default();
    let valid = fill("buy", OrderSide::Buy, 1, 100);
    let mut invalid = Vec::new();
    let mut changed = valid.clone();
    changed.cumulative_quantity = None;
    invalid.push(changed);
    let mut changed = valid.clone();
    changed.cumulative_notional = None;
    invalid.push(changed);
    let mut changed = valid.clone();
    changed.cumulative_notional = Some(0);
    invalid.push(changed);
    let mut changed = valid.clone();
    changed.cumulative_quantity = Some(0);
    invalid.push(changed);
    let mut changed = valid.clone();
    changed.key.identity.scope.account_id = None;
    invalid.push(changed);
    let mut changed = valid.clone();
    changed.key.identity.profile_id = Some(" ".into());
    invalid.push(changed);
    let mut changed = valid.clone();
    changed.key.owner = OwnershipOwner::Strategy("".into());
    invalid.push(changed);
    let mut changed = valid.clone();
    changed.key.symbol.clear();
    invalid.push(changed);
    let mut changed = valid.clone();
    changed.order_record_id.clear();
    invalid.push(changed);
    for changed in invalid {
        rejected_without_mutation(&mut ledger, &changed);
    }
    let mut unknown_sale = fill("external-sale", OrderSide::Sell, 1, 120);
    unknown_sale.key.owner = OwnershipOwner::Manual;
    rejected_without_mutation(&mut ledger, &unknown_sale);
    ledger.apply_cumulative(&valid).unwrap();
    rejected_without_mutation(
        &mut ledger,
        &fill("too-large-sale", OrderSide::Sell, 2, 240),
    );
    // Another owner's real confirmed BUY still does not authorize a manual SELL.
    rejected_without_mutation(&mut ledger, &unknown_sale);
}

#[test]
fn checked_quantity_cost_and_signed_profit_overflows_are_atomic() {
    let mut quantities = OwnershipLedger::default();
    quantities
        .apply_cumulative(&fill(
            "max-quantity",
            OrderSide::Buy,
            u64::MAX,
            u128::from(u64::MAX),
        ))
        .unwrap();
    rejected_without_mutation(&mut quantities, &fill("one-too-many", OrderSide::Buy, 1, 1));
    let mut costs = OwnershipLedger::default();
    costs
        .apply_cumulative(&fill("max-cost", OrderSide::Buy, 1, u128::MAX))
        .unwrap();
    rejected_without_mutation(&mut costs, &fill("cost-overflow", OrderSide::Buy, 1, 1));
    rejected_without_mutation(&mut costs, &fill("loss-overflow", OrderSide::Sell, 1, 1));
    let mut profit = OwnershipLedger::default();
    profit
        .apply_cumulative(&fill("buy", OrderSide::Buy, 1, 1))
        .unwrap();
    rejected_without_mutation(
        &mut profit,
        &fill("profit-overflow", OrderSide::Sell, 1, u128::MAX),
    );
    let mut boundary = OwnershipLedger::default();
    boundary
        .apply_cumulative(&fill("buy", OrderSide::Buy, 1, (i128::MAX as u128) + 2))
        .unwrap();
    assert_eq!(
        boundary
            .apply_cumulative(&fill("sell", OrderSide::Sell, 1, 1))
            .unwrap()
            .realized_pnl,
        i128::MIN
    );
}

#[test]
fn serde_restores_large_u128_exactly_and_keeps_fill_replay_idempotent() {
    let mut ledger = OwnershipLedger::default();
    let buy = fill("large-buy", OrderSide::Buy, 1, u128::MAX);
    ledger.apply_cumulative(&buy).unwrap();
    let json = bytes(&ledger);
    assert!(json.contains(&u128::MAX.to_string()));
    let mut restored: OwnershipLedger = serde_json::from_str(&json).unwrap();
    assert!(restored == ledger);
    assert_eq!(restored.apply_cumulative(&buy).unwrap().delta_quantity, 0);
    assert_eq!(restored.position(&key()).cost_notional, u128::MAX);
    // A full sale releases all known cost without overflowing cost * quantity.
    assert_eq!(
        restored
            .apply_cumulative(&fill("large-sale", OrderSide::Sell, 1, u128::MAX))
            .unwrap()
            .realized_pnl,
        0
    );
}

#[test]
fn serde_rejects_missing_invalid_duplicate_and_unsupported_state_without_guessing() {
    assert!(serde_json::from_str::<OwnershipLedger>("{}").is_err());
    let mut ledger = OwnershipLedger::default();
    ledger
        .apply_cumulative(&fill("buy", OrderSide::Buy, 1, 100))
        .unwrap();
    let original: serde_json::Value = serde_json::from_str(&bytes(&ledger)).unwrap();
    let mut variants = Vec::new();
    let mut changed = original.clone();
    changed["version"] = 2.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["positions"][0]["position"]["quantity"] = 0.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["positions"][0]["position"]["costNotional"] = 0.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["positions"][0]["key"]["identity"]["scope"]["accountId"] = serde_json::Value::Null;
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["buy"]["cumulativeQuantity"] = 2.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["positions"][0]["key"]["identity"]
        .as_object_mut()
        .unwrap()
        .remove("isPaper");
    variants.push(changed);
    let mut changed = original.clone();
    changed["positions"][0]["key"]["identity"]
        .as_object_mut()
        .unwrap()
        .remove("profileId");
    variants.push(changed);
    let mut changed = original.clone();
    changed["positions"]
        .as_array_mut()
        .unwrap()
        .push(original["positions"][0].clone());
    variants.push(changed);
    for changed in variants {
        assert!(serde_json::from_str::<OwnershipLedger>(&changed.to_string()).is_err());
    }
    let positions = original["positions"].to_string();
    let watermark = original["watermarks"]["buy"].to_string();
    let generation = original["generation"].as_str().unwrap();
    let duplicate = format!(
        r#"{{"version":1,"generation":"{generation}","positions":{positions},"watermarks":{{"buy":{watermark},"buy":{watermark}}}}}"#
    );
    assert!(serde_json::from_str::<OwnershipLedger>(&duplicate).is_err());
}

#[test]
fn terminal_pruning_requires_exact_proof_and_retains_the_owned_position() {
    let mut ledger = OwnershipLedger::default();
    let buy = fill("buy", OrderSide::Buy, 2, 250);
    ledger.apply_cumulative(&buy).unwrap();
    let before = bytes(&ledger);
    let mut mismatch = buy.clone();
    mismatch.cumulative_notional = Some(249);
    assert!(ledger.prune_terminal_order(&mismatch).is_err());
    assert_eq!(bytes(&ledger), before);
    assert!(ledger.prune_terminal_order(&buy).unwrap());
    assert!(ledger.order_watermark("buy").is_none());
    assert_eq!(
        ledger.position(&key()),
        OwnershipPosition {
            quantity: 2,
            cost_notional: 250
        }
    );
    assert!(!ledger.prune_terminal_order(&buy).unwrap());
    let restored: OwnershipLedger = serde_json::from_str(&bytes(&ledger)).unwrap();
    assert_eq!(restored.position(&key()).quantity, 2);
}

#[test]
fn debug_never_exposes_raw_account_profile_or_strategy_identity() {
    let printed = format!("{:?}", key());
    for value in ["synthetic-account", "synthetic-profile", "strategy-a"] {
        assert!(!printed.contains(value));
    }
    assert!(printed.contains("account_configured: true"));
}

#[test]
fn generation_is_required_valid_and_preserved_in_exact_json_roundtrips() {
    let ledger = OwnershipLedger::default();
    assert!(!ledger.generation().is_nil());
    assert_ne!(ledger.generation(), OwnershipLedger::default().generation());
    let restored: OwnershipLedger = serde_json::from_str(&bytes(&ledger)).unwrap();
    assert_eq!(restored.generation(), ledger.generation());
    let original: serde_json::Value = serde_json::from_str(&bytes(&ledger)).unwrap();
    let mut missing = original.clone();
    missing.as_object_mut().unwrap().remove("generation");
    assert!(serde_json::from_str::<OwnershipLedger>(&missing.to_string()).is_err());
    for generation in ["not-a-uuid", "00000000-0000-0000-0000-000000000000"] {
        let mut invalid = original.clone();
        invalid["generation"] = generation.into();
        assert!(serde_json::from_str::<OwnershipLedger>(&invalid.to_string()).is_err());
    }
    let mut legacy_metadata = original;
    legacy_metadata.as_object_mut().unwrap().remove("metadata");
    assert!(serde_json::from_str::<OwnershipLedger>(&legacy_metadata.to_string()).is_ok());
}

#[test]
fn metadata_is_isolated_requires_actual_ownership_and_clears_only_on_full_sale() {
    let mut ledger = OwnershipLedger::default();
    assert!(ledger
        .set_position_metadata(&key(), serde_json::json!({"highWater":120}))
        .is_err());
    ledger
        .apply_cumulative(&fill("buy", OrderSide::Buy, 2, 200))
        .unwrap();
    let protection = serde_json::json!({"entryPrice":100,"highWater":120,"observations":37});
    ledger
        .set_position_metadata(&key(), protection.clone())
        .unwrap();
    let mut other = key();
    other.identity.profile_id = Some("other-profile".into());
    assert!(ledger.position_metadata(&other).is_none());
    assert!(ledger
        .set_position_metadata(&other, protection.clone())
        .is_err());
    let restored: OwnershipLedger = serde_json::from_str(&bytes(&ledger)).unwrap();
    assert_eq!(restored.position_metadata(&key()), Some(&protection));
    ledger
        .apply_cumulative(&fill("partial-sell", OrderSide::Sell, 1, 110))
        .unwrap();
    assert_eq!(ledger.position_metadata(&key()), Some(&protection));
    ledger
        .apply_cumulative(&fill("last-sell", OrderSide::Sell, 1, 115))
        .unwrap();
    assert!(ledger.position_metadata(&key()).is_none());
    assert!(ledger.metadata.is_empty());
}

#[test]
fn invalid_oversized_unowned_and_duplicate_metadata_fail_atomically_and_on_restore() {
    let mut ledger = OwnershipLedger::default();
    ledger
        .apply_cumulative(&fill("buy", OrderSide::Buy, 1, 100))
        .unwrap();
    ledger
        .set_position_metadata(&key(), serde_json::json!({"highWater":120}))
        .unwrap();
    let before = bytes(&ledger);
    assert!(ledger
        .set_position_metadata(
            &key(),
            serde_json::Value::String("x".repeat(MAX_METADATA_BYTES))
        )
        .is_err());
    assert_eq!(bytes(&ledger), before);
    let mut deep = serde_json::Value::Null;
    for _ in 0..MAX_METADATA_DEPTH + 1 {
        deep = serde_json::Value::Array(vec![deep]);
    }
    assert!(ledger.set_position_metadata(&key(), deep).is_err());
    assert_eq!(bytes(&ledger), before);
    let original: serde_json::Value = serde_json::from_str(&before).unwrap();
    let mut invalid = original.clone();
    invalid["metadata"][0]["value"] = serde_json::Value::String("x".repeat(MAX_METADATA_BYTES));
    assert!(serde_json::from_str::<OwnershipLedger>(&invalid.to_string()).is_err());
    let mut invalid = original.clone();
    invalid["metadata"][0]["key"]["symbol"] = "UNOWNED".into();
    assert!(serde_json::from_str::<OwnershipLedger>(&invalid.to_string()).is_err());
    let mut invalid = original.clone();
    invalid["metadata"]
        .as_array_mut()
        .unwrap()
        .push(original["metadata"][0].clone());
    assert!(serde_json::from_str::<OwnershipLedger>(&invalid.to_string()).is_err());
}

#[test]
fn replay_keeps_zero_delta_but_exact_durable_application_summary_survives_restart() {
    let mut ledger = OwnershipLedger::default();
    ledger
        .apply_cumulative(&fill("buy", OrderSide::Buy, 3, 1001))
        .unwrap();
    let sell = fill("sell", OrderSide::Sell, 1, 400);
    let applied = ledger.apply_cumulative(&sell).unwrap();
    assert_eq!((applied.released_cost, applied.realized_pnl), (333, 67));
    assert_eq!(ledger.apply_cumulative(&sell).unwrap().delta_quantity, 0);
    assert_eq!(ledger.last_application("sell"), Some(applied.clone()));
    // A later fill and metadata update must not rewrite the earlier sale snapshot.
    ledger
        .apply_cumulative(&fill("later-buy", OrderSide::Buy, 2, 250))
        .unwrap();
    ledger
        .set_position_metadata(&key(), serde_json::json!({"highWater":200}))
        .unwrap();
    assert_ne!(ledger.position(&key()), applied.position);
    let mut restored: OwnershipLedger = serde_json::from_str(&bytes(&ledger)).unwrap();
    assert_eq!(restored.last_application("sell"), Some(applied));
    assert_eq!(restored.apply_cumulative(&sell).unwrap().delta_quantity, 0);
    assert!(restored.last_application("unknown").is_none());
}

#[test]
fn cumulative_progress_replaces_only_the_latest_durable_application_result() {
    let mut ledger = OwnershipLedger::default();
    let mut buy = fill("buy", OrderSide::Buy, 1, 100);
    buy.ordered_quantity = 3;
    ledger.apply_cumulative(&buy).unwrap();
    buy.cumulative_quantity = Some(3);
    buy.cumulative_notional = Some(360);
    let latest = ledger.apply_cumulative(&buy).unwrap();
    assert_eq!((latest.delta_quantity, latest.delta_notional), (2, 260));
    assert_eq!(ledger.last_application("buy"), Some(latest));
    assert_eq!(ledger.watermarks.len(), 1);
}

#[test]
fn restore_rejects_tampered_application_source_binding_and_arithmetic() {
    let mut ledger = OwnershipLedger::default();
    ledger
        .apply_cumulative(&fill("buy", OrderSide::Buy, 3, 300))
        .unwrap();
    ledger
        .apply_cumulative(&fill("sell", OrderSide::Sell, 1, 120))
        .unwrap();
    let original: serde_json::Value = serde_json::from_str(&bytes(&ledger)).unwrap();
    let mut variants = Vec::new();
    let mut changed = original.clone();
    changed["watermarks"]["sell"]
        .as_object_mut()
        .unwrap()
        .remove("lastApplication");
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["sell"]["key"]["symbol"] = "OTHER".into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["sell"]["side"] = "buy".into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["sell"]["orderedQuantity"] = 2.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["sell"]["cumulativeNotional"] = 121.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["sell"]["lastApplication"]["applied"]["releasedCost"] = 99.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["sell"]["lastApplication"]["applied"]["realizedPnl"] = 21.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["sell"]["lastApplication"]["applied"]["deltaQuantity"] = 2.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["buy"]["lastApplication"]["applied"]["releasedCost"] = 1.into();
    variants.push(changed);
    let mut changed = original.clone();
    changed["watermarks"]["buy"]["lastApplication"]["applied"]["position"]["costNotional"] =
        301.into();
    variants.push(changed);
    for changed in variants {
        assert!(serde_json::from_str::<OwnershipLedger>(&changed.to_string()).is_err());
    }
}
