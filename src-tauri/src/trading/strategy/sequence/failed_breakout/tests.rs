use super::*;
fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new(
        "failed_breakout_test",
        "test",
        true,
        vec!["TEST".into()],
        2,
        params,
    )
}
fn s(params: serde_json::Value) -> FailedBreakoutStrategy {
    let mut s = FailedBreakoutStrategy::new(config(params));
    s.initialize_historical("TEST", &[100; 20]);
    s
}
#[test]
fn candidates_do_not_change_ownership_and_same_zero_does_not_erase_anchor() {
    let mut s = s(serde_json::json!({"lookback_days":3,"buffer_pct":0.5}));
    assert!(matches!(s.on_tick("TEST", 110, 0), Signal::Buy { .. }));
    assert_eq!(s.has_held_position("TEST"), Some(false));
    s.sync_position("TEST", 0, 0);
    assert_eq!(s.states["TEST"].candidate_prev_high, Some(100));
    s.on_live_tick("TEST", 120, 0); // submission is unsettled before pending acknowledgement
    assert_eq!(s.states["TEST"].candidate_prev_high, Some(100));
    s.set_order_pending("TEST", true);
    s.on_tick("TEST", 130, 0);
    assert_eq!(s.states["TEST"].candidate_prev_high, Some(100));
    s.sync_position("TEST", 7, 115);
    s.set_order_pending("TEST", false);
    for qty in [7, 3, 3] {
        s.sync_position("TEST", qty, 115);
        assert!(matches!(s.on_tick("TEST",99,0),Signal::Sell{quantity,..} if quantity==qty));
        assert_eq!(s.states["TEST"].breakout_prev_high, Some(100));
    }
    s.sync_position("TEST", 0, 0);
    assert_eq!(s.states["TEST"].breakout_prev_high, None);
}
fn bar(high: u64, close: u64, volume: Option<u64>) -> CompletedDailyBar {
    CompletedDailyBar {
        ohlc: OhlcCandle {
            open: close,
            high,
            low: close,
            close,
        },
        volume,
    }
}
fn prep(s: &FailedBreakoutStrategy) -> EntryFilterPreparation {
    s.entry_filter_preparation("TEST").unwrap()
}
#[test]
fn legacy_floor_threshold_inclusive_and_failure_strict_are_preserved() {
    let mut s = s(serde_json::json!({"lookback_days":3,"buffer_pct":0.5}));
    assert!(matches!(s.on_tick("TEST", 100, 0), Signal::Buy { .. })); // 100.5 -> legacy floor100
    s.sync_position("TEST", 5, 100);
    assert_eq!(s.on_tick("TEST", 100, 0), Signal::Hold);
    assert!(matches!(
        s.on_tick("TEST", 99, 0),
        Signal::Sell { quantity: 5, .. }
    ));
}
#[test]
fn actual_quantity_unknown_origin_has_no_invented_failure_anchor_or_age() {
    let mut s =
        s(serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"max_hold_observations":2}));
    s.sync_position("TEST", 8, 200);
    assert_eq!(s.states["TEST"].breakout_prev_high, None);
    assert_eq!(s.states["TEST"].held_observations, None);
    assert!(
        matches!(s.on_tick("TEST",90,0),Signal::Sell{quantity:8,reason,..}if reason.contains("나이 불명"))
    );
    assert_eq!(s.states["TEST"].held_observations, None);
}
#[test]
fn daily_high_basis_is_previous_completed_high_not_close_or_candidate_high() {
    let mut s =
        s(serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"breakout_basis":"completedHigh"}));
    s.apply_completed_daily_bars("TEST", &[bar(200, 100, Some(10)); 3]);
    assert!(s.history_readiness("TEST").unwrap().ready);
    assert_eq!(s.on_tick("TEST", 150, 0), Signal::Hold);
    s.on_trading_day_start("TEST", 150);
    assert!(matches!(
        s.on_daily_close_tick("TEST", &bar(9999, 202, None).ohlc, 9999),
        Signal::Buy { .. }
    ));
    assert_eq!(s.states["TEST"].candidate_prev_high, Some(200));
    s.on_completed_daily_bar("TEST", &bar(9999, 202, None));
    assert_eq!(s.states["TEST"].context.high(3), Some(9999));
    s.on_completed_daily_bar("TEST", &bar(88888, 202, None));
    assert_eq!(s.states["TEST"].context.high(3), Some(9999));
}
#[test]
fn scoped_completed_high_context_can_prepare_without_price_seed() {
    let mut s = FailedBreakoutStrategy::new(config(
        serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"breakout_basis":"completedHigh"}),
    ));
    s.apply_completed_daily_bars("TEST", &[bar(200, 100, None); 3]);
    assert!(s.history_readiness("TEST").unwrap().ready);
    assert!(matches!(s.on_tick("TEST", 202, 0), Signal::Buy { .. }));
}
#[test]
fn atr_uses_prior_gaps_fractional_threshold_and_zero_range_is_ready() {
    let mut s =
        s(serde_json::json!({"lookback_days":3,"buffer_pct":0,"atr_buffer_multiplier":0.5}));
    let mut bars = vec![bar(100, 100, None); 15];
    bars[14] = bar(103, 101, None);
    s.apply_completed_daily_bars("TEST", &bars);
    assert_eq!(prep(&s).prior_atr, Some(3.0 / 14.0));
    assert_eq!(s.on_tick("TEST", 100, 0), Signal::Hold); // effective ceil100.107
    assert_eq!(prep(&s).atr_rejections, 1);
    assert!(matches!(s.on_tick("TEST", 101, 0), Signal::Buy { .. }));
    let mut z =
        self::s(serde_json::json!({"lookback_days":3,"buffer_pct":0,"atr_buffer_multiplier":20}));
    z.apply_completed_daily_bars("TEST", &[bar(100, 100, None); 15]);
    assert_eq!(prep(&z).prior_atr, Some(0.0));
    assert!(prep(&z).ready);
    assert!(matches!(z.on_tick("TEST", 100, 0), Signal::Buy { .. }));
}
#[test]
fn atr_never_weakens_legacy_percent_limb() {
    let mut s =
        s(serde_json::json!({"lookback_days":3,"buffer_pct":10,"atr_buffer_multiplier":0.5}));
    s.apply_completed_daily_bars("TEST", &[bar(100, 100, None); 15]);
    assert_eq!(s.on_tick("TEST", 109, 0), Signal::Hold);
    s.initialize_historical("TEST", &[100; 3]);
    assert!(matches!(s.on_tick("TEST", 110, 0), Signal::Buy { .. }));
}
#[test]
fn volume_requires_21_prior_bars_distinguishes_missing_zero_and_positive_ratio() {
    for (last, status, ratio, buy) in [
        (None, "unverifiedVolume", None, false),
        (Some(0), "ready", Some(0.0), false),
        (Some(150), "ready", Some(1.5), true),
    ] {
        let mut s =
            s(serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"min_volume_ratio":1.5}));
        let mut bars = vec![bar(100, 100, Some(100)); 21];
        bars[20].volume = last;
        s.apply_completed_daily_bars("TEST", &bars);
        assert_eq!(prep(&s).status, status);
        assert_eq!(prep(&s).prior_volume_ratio, ratio);
        assert_eq!(
            matches!(s.on_tick("TEST", 101, u64::MAX), Signal::Buy { .. }),
            buy
        );
        assert_eq!(prep(&s).volume_rejections, Some(u64::from(last == Some(0))));
    }
    let mut s = s(serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"min_volume_ratio":1}));
    s.apply_completed_daily_bars("TEST", &[bar(100, 100, Some(0)); 21]);
    assert_eq!(prep(&s).status, "zeroReferenceVolume");
    assert_eq!(prep(&s).prior_volume_ratio, None);
    s.apply_completed_daily_bars("TEST", &[bar(100, 100, Some(100)); 20]);
    assert_eq!(prep(&s).status, "insufficientHistory");
}
#[test]
fn context_updates_do_not_clear_anchor_age_peak_armed_and_stays_bounded() {
    let mut s = s(
        serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"profit_activation_pct":5,"profit_trailing_pct":3}),
    );
    s.on_tick("TEST", 110, 0);
    s.sync_position("TEST", 7, 110);
    s.on_tick("TEST", 120, 0);
    assert!(s.states["TEST"].profit_armed);
    let metadata = s.owned_position_metadata("TEST");
    s.initialize_daily_bars("TEST", &vec![bar(999, 100, None); 2000]);
    s.invalidate_completed_daily_history("TEST");
    s.apply_completed_daily_bars("TEST", &vec![bar(999, 100, None); 2000]);
    assert_eq!(metadata, s.owned_position_metadata("TEST"));
    assert_eq!(
        s.states["TEST"]
            .context
            .preparation(&s.params, true)
            .available_completed_bars,
        512
    );
    for _ in 0..20 {
        s.on_trading_day_start("TEST", 100);
        s.on_completed_daily_bar("TEST", &bar(100, 100, None));
    }
    assert_eq!(
        s.states["TEST"]
            .context
            .preparation(&s.params, true)
            .available_completed_bars,
        512
    );
}
#[test]
fn profit_uses_actual_average_owned_observed_peak_and_remains_armed_after_rejection_partial() {
    let mut s = s(
        serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"profit_activation_pct":5,"profit_trailing_pct":10}),
    );
    s.on_tick("TEST", 110, 0);
    s.sync_position("TEST", 8, 200);
    assert_eq!(s.states["TEST"].observed_peak, None);
    assert_eq!(
        s.on_daily_close_tick("TEST", &bar(99999, 209, None).ohlc, 0),
        Signal::Hold
    );
    assert!(!s.states["TEST"].profit_armed);
    assert_eq!(s.on_tick("TEST", 210, 0), Signal::Hold);
    assert!(s.states["TEST"].profit_armed);
    assert_eq!(s.on_tick("TEST", 220, 0), Signal::Hold);
    assert!(
        matches!(s.on_tick("TEST",198,0),Signal::Sell{quantity:8,reason,..}if reason.contains("수익 보호"))
    );
    s.sync_position("TEST", 3, 200);
    assert!(s.states["TEST"].profit_armed);
    assert!(matches!(
        s.on_tick("TEST", 197, 0),
        Signal::Sell { quantity: 3, .. }
    ));
}
#[test]
fn failure_profit_cap_priority_is_sustained_and_warmup_reconfigure_preserve_age() {
    let mut s = s(
        serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"profit_activation_pct":5,"profit_trailing_pct":10,"max_hold_observations":2}),
    );
    s.on_tick("TEST", 110, 0);
    s.sync_position("TEST", 8, 110);
    assert_eq!(s.states["TEST"].held_observations, Some(0));
    s.on_tick("TEST", 130, 0);
    s.initialize_historical("TEST", &[110; 3]);
    let mut config = s.config.clone();
    config.params["lookback_days"] = serde_json::json!(100);
    assert!(s.reconfigure_preserving_position(config));
    assert_eq!(s.states["TEST"].held_observations, Some(1));
    assert!(
        matches!(s.on_tick("TEST",115,0),Signal::Sell{reason,..}if reason.contains("수익 보호"))
    );
    assert!(
        matches!(s.on_tick("TEST",99,0),Signal::Sell{reason,..}if reason.contains("돌파 실패"))
    );
    assert!(
        matches!(s.on_tick("TEST",125,0),Signal::Sell{reason,..}if reason.contains("관측 한도"))
    );
}
#[test]
fn strict_optional_validation_does_not_erase_custom_legacy_or_other_valid_exit() {
    for (key, value) in [
        ("breakout_basis", serde_json::json!("bad")),
        ("atr_buffer_multiplier", serde_json::json!(null)),
        ("min_volume_ratio", serde_json::json!("1")),
        ("max_hold_observations", serde_json::json!(0.5)),
        ("profit_activation_pct", serde_json::json!(1)),
        ("profit_trailing_pct", serde_json::json!(true)),
    ] {
        let mut params =
            serde_json::json!({"lookback_days":3,"buffer_pct":2,"max_hold_observations":2});
        params[key] = value;
        assert!(validate_failed_breakout_params("failed_breakout", &params).is_err());
        let mut s = s(params);
        assert_eq!(s.params.lookback_days, 3);
        assert_eq!(s.params.buffer_pct, 2.0);
        assert_eq!(s.on_tick("TEST", 110, 0), Signal::Hold);
        s.sync_position("TEST", 5, 110);
        s.states.get_mut("TEST").unwrap().breakout_prev_high = Some(100);
        assert!(matches!(
            s.on_tick("TEST", 99, 0),
            Signal::Sell { quantity: 5, .. }
        ));
    }
    for params in [
        serde_json::json!({"max_hold_observations":1000000.0}),
        serde_json::json!({"atr_buffer_multiplier":20,"min_volume_ratio":100,"profit_activation_pct":1000,"profit_trailing_pct":99.9}),
    ] {
        assert!(validate_failed_breakout_params("failed_breakout", &params).is_ok());
    }
}
#[test]
fn checkpoint_v1_unknown_and_v2_known_roundtrip_fail_atomically() {
    let params = serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"max_hold_observations":2});
    let mut s = s(params.clone());
    s.sync_position("TEST", 5, 110);
    s.restore_owned_position_metadata("TEST",&serde_json::json!({"version":1,"kind":"failedBreakout","confirmedAnchor":100,"candidateAnchor":null})).unwrap();
    assert_eq!(s.states["TEST"].held_observations, None);
    assert_eq!(s.states["TEST"].observed_peak, None);
    assert!(!s.states["TEST"].profit_armed);
    assert!(
        matches!(s.on_tick("TEST",110,0),Signal::Sell{reason,..}if reason.contains("나이 불명"))
    );
    let before = s.owned_position_metadata("TEST").unwrap();
    for patch in [
        serde_json::json!({"version":3}),
        serde_json::json!({"kind":"rsi"}),
        serde_json::json!({"extra":0}),
        serde_json::json!({"heldObservations":1}),
        serde_json::json!({"candidateAnchor":100}),
        serde_json::json!({"observedPeak":0}),
        serde_json::json!({"profitArmed":true,"observedPeak":null}),
    ] {
        let mut bad = before.clone();
        bad.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(s.restore_owned_position_metadata("TEST", &bad).is_err());
        assert_eq!(s.owned_position_metadata("TEST"), Some(before.clone()));
    }
    let mut s = self::s(params.clone());
    s.on_tick("TEST", 110, 0);
    let pending = s.owned_position_metadata("TEST").unwrap();
    s.sync_position("TEST", 4, 111);
    s.on_tick("TEST", 115, 0);
    let known = s.owned_position_metadata("TEST").unwrap();
    let mut restart = self::s(params);
    restart.sync_position("TEST", 4, 111);
    restart
        .restore_owned_position_metadata("TEST", &known)
        .unwrap();
    assert_eq!(restart.owned_position_metadata("TEST"), Some(known));
    restart
        .restore_owned_position_metadata("TEST", &pending)
        .unwrap();
    assert_eq!(restart.states["TEST"].held_observations, Some(0));
}
#[test]
fn unsubmitted_discard_and_scope_edits_do_not_change_confirmed_protection() {
    let mut s = s(serde_json::json!({"lookback_days":3,"buffer_pct":0.5}));
    s.on_tick("TEST", 110, 0);
    s.discard_unsubmitted_signal("TEST");
    assert_eq!(s.owned_position_metadata("TEST"), None);
    s.on_tick("TEST", 120, 0);
    s.set_order_pending("TEST", true);
    let before = s.owned_position_metadata("TEST");
    s.discard_unsubmitted_signal("TEST");
    assert_eq!(before, s.owned_position_metadata("TEST"));
    s.sync_position("TEST", 4, 120);
    let before = s.owned_position_metadata("TEST");
    s.discard_unsubmitted_signal("TEST");
    assert_eq!(before, s.owned_position_metadata("TEST"));
    let mut other = s.config.clone();
    other.target_symbols.clear();
    assert!(!s.reconfigure_preserving_position(other));
    let mut other = s.config.clone();
    other.broker_account_id = Some("other".into());
    assert!(!s.reconfigure_preserving_position(other));
    assert_eq!(before, s.owned_position_metadata("TEST"));
}

#[test]
fn pending_without_signal_checkpoint_never_invents_a_later_anchor() {
    let mut s =
        s(serde_json::json!({"lookback_days":3,"buffer_pct":0.5,"max_hold_observations":2}));
    s.set_order_pending("TEST", true);
    s.on_live_tick("TEST", 110, 0);
    assert_eq!(s.states["TEST"].candidate_prev_high, None);
    s.sync_owned_position("TEST", 4, 111);
    assert_eq!(s.states["TEST"].breakout_prev_high, None);
    assert_eq!(s.states["TEST"].held_observations, None);
    assert!(
        matches!(s.on_live_tick("TEST",110,0),Signal::Sell{quantity:4,reason,..}if reason.contains("나이 불명"))
    );
}
