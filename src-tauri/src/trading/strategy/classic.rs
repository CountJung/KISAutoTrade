#[cfg(test)]
use super::{Signal, Strategy, StrategyConfig};

mod rsi;
pub use rsi::{validate_rsi_params, RsiParams, RsiStrategy};

mod momentum;
pub use momentum::{
    validate_momentum_entry_params, MomentumEntryMode, MomentumParams, MomentumStrategy,
};

mod deviation;
pub use deviation::{
    validate_deviation_entry_params, DeviationEntryMode, DeviationParams, DeviationStrategy,
};

#[cfg(test)]
mod history_tests {
    use super::*;

    fn config(params: serde_json::Value) -> StrategyConfig {
        StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
    }

    #[test]
    fn rsi_initializer_restores_crossing_baseline_without_historical_signals() {
        let mut strategy = RsiStrategy::new(config(serde_json::json!({
            "period": 3, "oversold": 30.0, "overbought": 70.0,
        })));
        strategy.initialize_historical("SOXQ", &[100, 90, 80, 70]);
        assert_eq!(strategy.states["SOXQ"].previous_rsi, Some(0.0));
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(
            strategy.on_tick("SOXQ", 120, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_historical("SOXQ", &[100, 90, 80]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
        assert_eq!(strategy.states["SOXQ"].previous_rsi, None);
    }

    #[test]
    fn momentum_history_does_not_create_or_erase_order_state() {
        let mut strategy = MomentumStrategy::new(config(serde_json::json!({
            "lookback_period": 3, "threshold_pct": 5.0,
        })));
        strategy.initialize_historical("SOXQ", &[100, 100, 100, 100]);
        assert_eq!(strategy.states["SOXQ"].owned_quantity, 0);
        assert!(matches!(
            strategy.on_tick("SOXQ", 110, 0),
            Signal::Buy { .. }
        ));
        strategy.sync_position("SOXQ", 2, 110);
        strategy.initialize_historical("SOXQ", &[100, 100, 100, 100]);
        assert_eq!(strategy.states["SOXQ"].owned_quantity, 2);
        assert!(matches!(strategy.on_tick("SOXQ", 115, 0), Signal::Hold));
        strategy.initialize_historical("SOXQ", &[100, 100]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    }

    #[test]
    fn deviation_history_is_ready_before_first_evaluation_and_bounded() {
        let mut strategy = DeviationStrategy::new(config(serde_json::json!({
            "ma_period": 3, "buy_threshold_pct": -5.0, "sell_threshold_pct": 5.0,
        })));
        strategy.initialize_historical("SOXQ", &vec![100; 1000]);
        assert_eq!(strategy.states["SOXQ"].prices.len(), 3);
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(
            strategy.on_tick("SOXQ", 50, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_intraday_prices("SOXQ", &[20, 20]);
        assert_eq!(strategy.states["SOXQ"].prices.front(), Some(&20));
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    }
    #[test]
    fn momentum_counts_first_comparison_but_excludes_seed_prices() {
        let mut strategy = MomentumStrategy::new(config(serde_json::json!({
            "lookback_period": 3, "threshold_pct": 5.0,
        })));
        let mut evaluated = 0;
        for (index, price) in [100, 100, 100, 110].into_iter().enumerate() {
            let can_evaluate = strategy.can_evaluate_next_tick("SOXQ").unwrap();
            assert_eq!(can_evaluate, index == 3);
            evaluated += usize::from(can_evaluate);
            let signal = strategy.on_tick("SOXQ", price, 0);
            assert_eq!(matches!(signal, Signal::Buy { .. }), index == 3);
        }
        assert_eq!(evaluated, 1);
    }

    #[test]
    fn deviation_counts_current_price_as_last_required_observation() {
        let mut strategy = DeviationStrategy::new(config(serde_json::json!({
            "ma_period": 3, "buy_threshold_pct": -5.0, "sell_threshold_pct": 5.0,
        })));
        let mut evaluated = 0;
        for (index, price) in [100, 100, 50].into_iter().enumerate() {
            let can_evaluate = strategy.can_evaluate_next_tick("SOXQ").unwrap();
            assert_eq!(can_evaluate, index == 2);
            evaluated += usize::from(can_evaluate);
            let signal = strategy.on_tick("SOXQ", price, 0);
            assert_eq!(matches!(signal, Signal::Buy { .. }), index == 2);
        }
        assert_eq!(evaluated, 1);
        let mut flat = DeviationStrategy::new(config(serde_json::json!({
            "ma_period": 3, "buy_threshold_pct": -5.0, "sell_threshold_pct": 5.0,
        })));
        flat.initialize_historical("SOXQ", &[100, 100]);
        assert!(!flat.history_readiness("SOXQ").unwrap().ready);
        assert_eq!(flat.can_evaluate_next_tick("SOXQ"), Some(true));
        assert!(matches!(flat.on_tick("SOXQ", 100, 0), Signal::Hold));
    }
}

#[cfg(test)]
mod reconfigure_tests;

#[cfg(test)]
#[path = "classic/live_feedback_tests.rs"]
mod live_feedback_tests;
