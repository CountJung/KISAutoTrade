use super::tests::{entry, strategy_with_params};
use super::*;

#[test]
fn reconfigure_lth_keeps_exact_protection_lifecycle_and_uses_new_exit_parameters() {
    let mut strategy = strategy_with_params(LeveragedTrendHoldParams {
        entries: vec![entry("SOXQ")],
        trailing_stop_pct: 20.0,
        min_hold_observations: 1,
        ..LeveragedTrendHoldParams::default()
    });
    strategy.positions.insert(
        "SOXQ".into(),
        LeveragedTrendHoldPosition {
            in_position: true,
            entry_price: Some(100),
            high_water: Some(120),
            held_observations: 37,
        },
    );
    for change in ["id", "account", "broker"] {
        let mut other = strategy.config().clone();
        match change {
            "id" => other.id.push_str("_other"),
            "account" => other.broker_account_id = Some("other".into()),
            _ => other.broker_id = crate::broker::BrokerId::Toss,
        }
        assert!(!strategy.reconfigure_preserving_position(other));
    }
    let mut next = strategy.config().clone();
    let mut params = strategy.params.clone();
    params.trailing_stop_pct = 2.0;
    params.entries[0].quantity = 7;
    params.entries.push(entry("NEW"));
    next.params = serde_json::to_value(params).unwrap();
    next.target_symbols = vec!["IGNORED".into()];
    assert!(strategy.reconfigure_preserving_position(next));
    let held = &strategy.positions["SOXQ"];
    assert!(held.in_position);
    assert_eq!(held.entry_price, Some(100));
    assert_eq!(held.high_water, Some(120));
    assert_eq!(held.held_observations, 37);
    assert!(!strategy.positions.contains_key("NEW"));
    assert_eq!(strategy.config.target_symbols, vec!["NEW", "SOXQ"]);
    // No indicator seed is needed for trailing protection against the exact historical high.
    assert!(
        matches!(strategy.on_tick("SOXQ", 115, 0), Signal::Sell { quantity: 7, reason, .. } if reason.contains("추적손절"))
    );
}

#[test]
fn reconfigure_lth_keeps_initial_stop_without_new_window_warmup() {
    let mut strategy = strategy_with_params(LeveragedTrendHoldParams {
        entries: vec![entry("SOXQ")],
        initial_stop_loss_pct: 10.0,
        ..LeveragedTrendHoldParams::default()
    });
    strategy.sync_position("SOXQ", 2, 100);
    let mut next = strategy.config().clone();
    next.params["initial_stop_loss_pct"] = 2.0.into();
    next.params["ema_long_period"] = 200.into();
    assert!(strategy.reconfigure_preserving_position(next));
    assert!(
        matches!(strategy.on_tick("SOXQ", 97, 0), Signal::Sell { reason, .. } if reason.contains("초기 손절"))
    );
}

#[test]
fn reconfigure_lth_trims_raw_observations_to_new_caps_without_reconstructing_position() {
    let mut strategy = strategy_with_params(LeveragedTrendHoldParams {
        entries: vec![entry("SOXQ")],
        ema_long_period: 200,
        ..LeveragedTrendHoldParams::default()
    });
    let candles = vec![
        OhlcCandle {
            open: 100,
            high: 110,
            low: 90,
            close: 100
        };
        250
    ];
    strategy.initialize_intraday_ohlc("SOXQ", &candles);
    strategy.initialize_ohlc("SOXQ", &candles);
    strategy.sync_position("SOXQ", 2, 100);
    let mut next = strategy.config().clone();
    next.params["ema_long_period"] = 20.into();
    next.params["rebound_baseline_ticks"] = 3.into();
    next.params["rebound_confirm_ticks"] = 2.into();
    next.params["rapid_rebound_lookback_ticks"] = 4.into();
    assert!(strategy.reconfigure_preserving_position(next));
    assert_eq!(
        strategy.states["SOXQ"].intraday_candles.len(),
        strategy.window_cap()
    );
    assert!(strategy.states["SOXQ"].rebound_prices.len() <= strategy.rebound_price_cap());
    assert_eq!(strategy.daily_context["SOXQ"].len(), strategy.window_cap());
    assert!(strategy.positions["SOXQ"].in_position);
}
