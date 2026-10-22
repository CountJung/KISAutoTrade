#[cfg(test)]
use super::{Signal, Strategy, StrategyConfig};

mod consecutive;
pub use consecutive::{
    validate_consecutive_entry_params, ConsecutiveMoveParams, ConsecutiveMoveStrategy,
};

mod failed_breakout;
pub use failed_breakout::{
    validate_failed_breakout_params, FailedBreakoutBasis, FailedBreakoutParams,
    FailedBreakoutStrategy,
};

#[cfg(test)]
mod history_tests {
    use super::*;

    fn config(params: serde_json::Value) -> StrategyConfig {
        StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
    }

    #[test]
    fn consecutive_history_preserves_flat_and_existing_positions() {
        let mut strategy = ConsecutiveMoveStrategy::new(config(serde_json::json!({
            "buy_days": 3, "sell_days": 3,
        })));
        strategy.initialize_historical("SOXQ", &[100, 101, 102, 103]);
        assert!(!strategy.states["SOXQ"].in_position);
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(
            strategy.on_tick("SOXQ", 104, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_historical("SOXQ", &[104, 103, 102, 101]);
        assert!(strategy.states["SOXQ"].in_position);
        assert!(matches!(
            strategy.on_tick("SOXQ", 100, 0),
            Signal::Sell { .. }
        ));
        strategy.initialize_historical("SOXQ", &[100, 101]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    }

    #[test]
    fn failed_breakout_history_retains_actual_breakout_reference() {
        let mut strategy = FailedBreakoutStrategy::new(config(serde_json::json!({
            "lookback_days": 3, "buffer_pct": 0.5,
        })));
        strategy.initialize_historical("SOXQ", &[100, 100]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
        strategy.initialize_historical("SOXQ", &[100, 100, 100]);
        assert_eq!(strategy.states["SOXQ"].owned_quantity, 0);
        assert!(matches!(
            strategy.on_tick("SOXQ", 120, 0),
            Signal::Buy { .. }
        ));
        strategy.sync_position("SOXQ", 1, 120);
        strategy.initialize_historical("SOXQ", &[110, 110, 110]);
        assert_eq!(strategy.states["SOXQ"].breakout_prev_high, Some(100));
        assert!(matches!(
            strategy.on_tick("SOXQ", 90, 0),
            Signal::Sell { .. }
        ));
        strategy.initialize_intraday_prices("SOXQ", &[20, 20, 20]);
        assert_eq!(strategy.states["SOXQ"].prices.front(), Some(&20));
    }
    #[test]
    fn consecutive_counts_current_price_and_uses_active_position_branch() {
        let mut strategy = ConsecutiveMoveStrategy::new(config(serde_json::json!({
            "buy_days": 2, "sell_days": 4,
        })));
        let mut evaluated = 0;
        for (index, price) in [100, 101, 102].into_iter().enumerate() {
            let can_evaluate = strategy.can_evaluate_next_tick("SOXQ").unwrap();
            assert_eq!(can_evaluate, index == 2);
            evaluated += usize::from(can_evaluate);
            let signal = strategy.on_tick("SOXQ", price, 0);
            assert_eq!(matches!(signal, Signal::Buy { .. }), index == 2);
        }
        assert_eq!(evaluated, 1);
        assert_eq!(strategy.can_evaluate_next_tick("SOXQ"), Some(false));
        assert!(matches!(strategy.on_tick("SOXQ", 101, 0), Signal::Hold));
        assert_eq!(strategy.can_evaluate_next_tick("SOXQ"), Some(true));
        strategy.initialize_historical("SOXQ", &[104, 103, 102, 101]);
        assert!(matches!(
            strategy.on_tick("SOXQ", 100, 0),
            Signal::Sell { .. }
        ));
    }

    #[test]
    fn failed_breakout_excludes_last_seed_tick_before_first_breakout_evaluation() {
        let mut strategy = FailedBreakoutStrategy::new(config(serde_json::json!({
            "lookback_days": 3, "buffer_pct": 0.5,
        })));
        let mut evaluated = 0;
        for (index, price) in [100, 101, 102, 110].into_iter().enumerate() {
            let can_evaluate = strategy.can_evaluate_next_tick("SOXQ").unwrap();
            assert_eq!(can_evaluate, index == 3);
            evaluated += usize::from(can_evaluate);
            let signal = strategy.on_tick("SOXQ", price, 0);
            assert_eq!(matches!(signal, Signal::Buy { .. }), index == 3);
        }
        assert_eq!(evaluated, 1);
    }
}

#[cfg(test)]
mod reconfigure_tests;

#[cfg(test)]
#[path = "sequence/live_feedback_tests.rs"]
mod live_feedback_tests;
