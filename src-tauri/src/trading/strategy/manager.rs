use crate::broker::{BrokerId, BrokerMarket, BrokerScope};

mod configuration;
mod live_history;
pub use live_history::{DailyHistoryStatus, DailyHistoryTicket};

use super::{
    initialize_strategy_warmup, BrokerPositionSnapshot, ConsecutiveMoveStrategy, DeviationStrategy,
    FailedBreakoutStrategy, FiftyTwoWeekHighStrategy, LeveragedTrendHoldStrategy,
    MeanReversionStrategy, MomentumStrategy, MovingAverageCrossStrategy, OhlcCandle,
    PriceConditionStrategy, RsiStrategy, Signal, Strategy, StrategyConfig, StrategySignal,
    StrongCloseStrategy, TrendFilterStrategy, VolatilityExpansionStrategy,
};

pub fn build_strategy(config: StrategyConfig) -> Box<dyn Strategy> {
    if config.id.starts_with("ma_cross") {
        Box::new(MovingAverageCrossStrategy::new(config))
    } else if config.id.starts_with("rsi") {
        Box::new(RsiStrategy::new(config))
    } else if config.id.starts_with("momentum") {
        Box::new(MomentumStrategy::new(config))
    } else if config.id.starts_with("deviation") {
        Box::new(DeviationStrategy::new(config))
    } else if config.id.starts_with("fifty_two_week_high") {
        Box::new(FiftyTwoWeekHighStrategy::new(config))
    } else if config.id.starts_with("consecutive_move") {
        Box::new(ConsecutiveMoveStrategy::new(config))
    } else if config.id.starts_with("failed_breakout") {
        Box::new(FailedBreakoutStrategy::new(config))
    } else if config.id.starts_with("strong_close") {
        Box::new(StrongCloseStrategy::new(config))
    } else if config.id.starts_with("volatility_expansion") {
        Box::new(VolatilityExpansionStrategy::new(config))
    } else if config.id.starts_with("mean_reversion") {
        Box::new(MeanReversionStrategy::new(config))
    } else if config.id.starts_with("trend_filter") {
        Box::new(TrendFilterStrategy::new(config))
    } else if config.id.starts_with("leveraged_trend_hold") {
        Box::new(LeveragedTrendHoldStrategy::new(config))
    } else if config.id.starts_with("price_condition") {
        Box::new(PriceConditionStrategy::new(config))
    } else {
        tracing::warn!(
            "알 수 없는 전략 ID '{}' — MovingAverageCrossStrategy로 대체합니다.",
            config.id
        );
        Box::new(MovingAverageCrossStrategy::new(config))
    }
}

pub struct StrategyManager {
    strategies: Vec<Box<dyn Strategy>>,
    history_generation: u64,
    daily_history: std::collections::BTreeMap<(String, String), live_history::DailyHistoryEntry>,
    history_execution_identity: Option<(crate::broker::BrokerScope, Option<String>)>,
    configuration_edit:
        Option<std::sync::Weak<crate::trading::configuration::ConfigurationEditReservation>>,
    unsettled_signals: std::collections::HashMap<(String, String), u64>,
    pending_orders: std::collections::HashSet<(BrokerScope, String, String)>,
}

impl StrategyManager {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
            history_generation: 0,
            daily_history: std::collections::BTreeMap::new(),
            history_execution_identity: None,
            configuration_edit: None,
            unsettled_signals: std::collections::HashMap::new(),
            pending_orders: std::collections::HashSet::new(),
        }
    }

    pub fn add(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }

    pub fn on_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Vec<StrategySignal> {
        self.on_tick_filtered(symbol, price, volume, |_| true)
    }

    pub fn on_tick_filtered<F>(
        &mut self,
        symbol: &str,
        price: u64,
        volume: u64,
        mut allow: F,
    ) -> Vec<StrategySignal>
    where
        F: FnMut(&StrategyConfig) -> bool,
    {
        let paused = self
            .configuration_edit
            .as_ref()
            .and_then(std::sync::Weak::upgrade);
        let live = self.history_execution_identity.is_some();
        let pending_orders = &self.pending_orders;
        let unsettled = &self.unsettled_signals;
        let signals: Vec<_> = self
            .strategies
            .iter_mut()
            .filter(|s| {
                !paused.as_ref().is_some_and(|edit| {
                    s.config().matches_scope(
                        edit.scope.broker_id,
                        edit.scope.account_id.as_ref().map(|id| id.0.as_str()),
                    )
                })
            })
            .filter(|s| allow(s.config()))
            .filter_map(|s| {
                let signal = if live {
                    s.on_live_tick(symbol, price, volume)
                } else {
                    s.on_tick(symbol, price, volume)
                };
                let config = s.config();
                let scope = BrokerScope {
                    broker_id: config.broker_id,
                    account_id: config
                        .broker_account_id
                        .clone()
                        .map(crate::broker::BrokerAccountId),
                };
                let pending = live
                    && (pending_orders.contains(&(scope, s.id().to_string(), symbol.to_string()))
                        || unsettled.contains_key(&(s.id().to_string(), symbol.to_string())));
                if signal == Signal::Hold || pending {
                    None
                } else {
                    Some(StrategySignal {
                        strategy_id: s.id().to_string(),
                        signal,
                    })
                }
            })
            .collect();
        if self.history_execution_identity.is_some() {
            for signal in &signals {
                let symbol = match &signal.signal {
                    Signal::Buy { symbol, .. } | Signal::Sell { symbol, .. } => symbol,
                    Signal::Hold => continue,
                };
                self.unsettled_signals.insert(
                    (signal.strategy_id.clone(), symbol.clone()),
                    self.history_generation,
                );
            }
        }
        signals
    }

    pub fn any_active_config_for_symbol<F>(&self, symbol: &str, mut predicate: F) -> bool
    where
        F: FnMut(&StrategyConfig) -> bool,
    {
        if self.configuration_edit_active() {
            return false;
        }
        self.strategies.iter().any(|strategy| {
            let config = strategy.config();
            strategy.is_enabled() && config.targets_symbol(symbol) && predicate(config)
        })
    }

    pub fn reset_all(&mut self) {
        self.advance_history_generation();
        self.strategies.iter_mut().for_each(|s| s.reset());
    }

    pub fn active_names(&self) -> Vec<String> {
        self.strategies
            .iter()
            .filter(|s| s.is_enabled())
            .map(|s| s.name().to_string())
            .collect()
    }

    /// 활성 전략에 등록된 구독 종목 코드 목록 (중복 제거)
    pub fn active_symbols(&self) -> Vec<String> {
        if self.configuration_edit_active() {
            return Vec::new();
        }
        let mut symbols: Vec<String> = self
            .strategies
            .iter()
            .filter(|s| s.is_enabled())
            .flat_map(|s| s.config().target_symbols.clone())
            .collect();
        symbols.sort_unstable();
        symbols.dedup();
        symbols
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 가격 데이터 전달 (52주 신고가 등 히스토리 기반 전략 초기화)
    pub fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        for s in &mut self.strategies {
            if s.config().targets_symbol(symbol) {
                s.initialize_historical(symbol, prices);
            }
        }
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 (고가, 종가) 쌍 데이터 전달 (강한 종가 등)
    pub fn initialize_candles(&mut self, symbol: &str, candles: &[(u64, u64)]) {
        for s in &mut self.strategies {
            if s.config().targets_symbol(symbol) {
                s.initialize_candles(symbol, candles);
            }
        }
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 OHLC 데이터 전달 (ADX/갭/양봉 판단)
    pub fn initialize_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        for s in &mut self.strategies {
            if s.config().targets_symbol(symbol) {
                s.initialize_ohlc(symbol, candles);
            }
        }
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 장중 가격 데이터 전달 (반동/매수세 판단)
    pub fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        for s in &mut self.strategies {
            if s.config().targets_symbol(symbol) {
                s.initialize_intraday_prices(symbol, prices);
            }
        }
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 장중 OHLC 데이터 전달 (1분봉 기반 추세/반동 판단)
    pub fn initialize_intraday_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        for s in &mut self.strategies {
            if s.config().targets_symbol(symbol) {
                s.initialize_intraday_ohlc(symbol, candles);
            }
        }
    }

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 변동 범위(고가-저가) 데이터 전달 (변동성 확장 전략)
    pub fn initialize_range_data(&mut self, symbol: &str, ranges: &[u64]) {
        for s in &mut self.strategies {
            if s.config().targets_symbol(symbol) {
                s.initialize_range_data(symbol, ranges);
            }
        }
    }

    /// live provider 응답을 공통 warmup 경계에 전달한다.
    pub fn initialize_warmup(
        &mut self,
        symbol: &str,
        daily_ohlc: &[OhlcCandle],
        intraday_ohlc: &[OhlcCandle],
    ) {
        for strategy in &mut self.strategies {
            if strategy.config().targets_symbol(symbol) {
                // Provider 일봉에는 완료 플래그가 없다. 신고가 live 경로의 기존
                // 최신 봉 제외 정책을 유지한다. Preview의 엄격한 과거 prefix는
                // manager를 거치지 않으므로 완료된 마지막 봉까지 모두 사용한다.
                let daily_ohlc =
                    if strategy.id().starts_with("fifty_two_week_high") && !daily_ohlc.is_empty() {
                        let history = &daily_ohlc[..daily_ohlc.len() - 1];
                        if history.is_empty() {
                            strategy.initialize_ohlc(symbol, history);
                        }
                        history
                    } else {
                        daily_ohlc
                    };
                initialize_strategy_warmup(strategy.as_mut(), symbol, daily_ohlc, intraday_ohlc);
                if strategy.legacy_warmup_requires_daily_verification() {
                    strategy.invalidate_completed_daily_history(symbol);
                    // A delayed legacy warmup can arrive after scoped history is
                    // ready. Its unverified bars must also expire that status so
                    // the next preparation schedules a genuine completed fetch.
                    self.daily_history
                        .remove(&(strategy.id().to_owned(), symbol.to_owned()));
                }
            }
        }
    }

    /// 실제 잔고를 전략별 내부 포지션 상태에 반영한다.
    pub fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        let market = if crate::market_hours::is_domestic_symbol(symbol) {
            BrokerMarket::Kr
        } else {
            BrokerMarket::Us
        };
        self.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Kis,
            market,
            symbol: symbol.to_string(),
            quantity,
            avg_price,
        });
    }

    /// broker/account scope가 있는 실제 잔고를 전략별 내부 포지션 상태에 반영한다.
    pub fn sync_position_for_broker(&mut self, snapshot: &BrokerPositionSnapshot) {
        for s in &mut self.strategies {
            s.sync_position_for_broker(snapshot);
        }
    }

    /// Reconciliation targets one exact owner; shared symbols and manual holdings
    /// must never broadcast confirmed quantity into other strategies.
    pub(crate) fn sync_owned_position_for_strategy(
        &mut self,
        strategy_id: &str,
        scope: &BrokerScope,
        market: BrokerMarket,
        symbol: &str,
        quantity: u64,
        avg_price: u64,
    ) -> bool {
        let Some(strategy) = self.strategies.iter_mut().find(|strategy| {
            owned_target_matches(strategy.as_ref(), strategy_id, scope, market, symbol)
        }) else {
            return false;
        };
        strategy.sync_owned_position(symbol, quantity, avg_price);
        true
    }

    /// Pending suppression is scoped to the actual owner and survives stop/start.
    /// Calling this for a different account or market cannot mutate a candidate.
    pub(crate) fn note_order_pending(
        &mut self,
        strategy_id: &str,
        scope: &BrokerScope,
        market: BrokerMarket,
        symbol: &str,
        pending: bool,
    ) -> bool {
        let Some(strategy) = self.strategies.iter_mut().find(|strategy| {
            owned_target_matches(strategy.as_ref(), strategy_id, scope, market, symbol)
        }) else {
            return false;
        };
        strategy.set_order_pending(symbol, pending);
        let key = (scope.clone(), strategy_id.to_string(), symbol.to_string());
        if pending {
            self.pending_orders.insert(key);
        } else {
            self.pending_orders.remove(&key);
        }
        true
    }

    pub(crate) fn owned_position_metadata_for_strategy(
        &self,
        strategy_id: &str,
        scope: &BrokerScope,
        market: BrokerMarket,
        symbol: &str,
    ) -> Option<serde_json::Value> {
        self.strategies
            .iter()
            .find(|strategy| {
                owned_target_matches(strategy.as_ref(), strategy_id, scope, market, symbol)
            })?
            .owned_position_metadata(symbol)
    }

    pub(crate) fn restore_owned_position_metadata_for_strategy(
        &mut self,
        strategy_id: &str,
        scope: &BrokerScope,
        market: BrokerMarket,
        symbol: &str,
        metadata: &serde_json::Value,
    ) -> Result<bool, String> {
        let Some(strategy) = self.strategies.iter_mut().find(|strategy| {
            owned_target_matches(strategy.as_ref(), strategy_id, scope, market, symbol)
        }) else {
            return Ok(false);
        };
        strategy.restore_owned_position_metadata(symbol, metadata)?;
        Ok(true)
    }

    /// 저장된 전략 설정으로 인메모리 전략 상태 업데이트 (프로그램 재시작 또는 프로필 전환 후 복원)
    ///
    /// 모든 전략을 기본값(비활성화, 종목 없음)으로 먼저 리셋한 뒤 저장된 설정을 덮어씀.
    /// 저장된 설정이 없는 프로필로 전환할 때 이전 프로필 종목이 잔류하는 버그를 방지함.
    pub fn apply_saved_configs(&mut self, saved: &[StrategyConfig]) {
        self.apply_saved_configs_for_scope(saved, BrokerId::Kis, None);
    }

    pub fn apply_saved_configs_for_scope(
        &mut self,
        saved: &[StrategyConfig],
        broker_id: BrokerId,
        broker_account_id: Option<String>,
    ) {
        self.advance_history_generation();
        for strategy in &mut self.strategies {
            let mut config = strategy.config().clone();
            config.enabled = false;
            config.target_symbols.clear();
            config.set_scope(broker_id, broker_account_id.clone());
            if let Some(saved_config) = saved.iter().rev().find(|saved| saved.id == config.id) {
                config.enabled = saved_config.enabled;
                config.target_symbols = saved_config.target_symbols.clone();
                config.order_quantity = saved_config.order_quantity;
                config.params = saved_config.params.clone();
            }
            // Same-account application preserves each strategy's held protection.
            // Profile publication explicitly rebuilds first in the method below.
            if !strategy.reconfigure_preserving_position(config.clone()) {
                *strategy = build_strategy(config);
            }
        }
    }

    /// Profile publication deliberately starts without another profile's private
    /// holdings, even when the broker/account strings happen to be identical.
    pub(crate) fn replace_saved_configs_for_scope(
        &mut self,
        saved: &[StrategyConfig],
        broker_id: BrokerId,
        account_id: Option<String>,
    ) {
        self.clear_history_execution_identity();
        self.pending_orders.clear();
        for strategy in &mut self.strategies {
            let mut defaults = strategy.config().clone();
            defaults.enabled = false;
            defaults.target_symbols.clear();
            defaults.order_quantity = 1;
            // PC/LTH derive targets from params. An empty saved profile must not
            // inherit another profile's entry list or customized parameters.
            defaults.params = serde_json::json!({});
            *strategy = build_strategy(defaults);
        }
        self.apply_saved_configs_for_scope(saved, broker_id, account_id);
    }

    /// 전체 전략 설정 반환
    pub fn all_configs(&self) -> Vec<&StrategyConfig> {
        self.strategies.iter().map(|s| s.config()).collect()
    }

    /// 특정 ID의 전략 설정 가변 참조 반환
    pub fn get_config_mut(&mut self, id: &str) -> Option<&mut StrategyConfig> {
        if self.strategies.iter().any(|strategy| strategy.id() == id) {
            self.advance_history_generation();
        }
        self.strategies
            .iter_mut()
            .find(|s| s.id() == id)
            .map(|s| s.config_mut())
    }

    pub fn update_config<F>(&mut self, id: &str, update: F) -> Option<StrategyConfig>
    where
        F: FnOnce(&mut StrategyConfig),
    {
        let index = self.strategies.iter().position(|s| s.id() == id)?;
        self.advance_history_generation();
        let previous = self.strategies[index].config();
        let mut updated = previous.clone();
        update(&mut updated);
        let rebuild = previous.params != updated.params
            || previous.target_symbols != updated.target_symbols
            || previous.id != updated.id
            || !previous.matches_scope(updated.broker_id, updated.broker_account_id.as_deref());
        if !rebuild {
            *self.strategies[index].config_mut() = updated.clone();
        } else if !self.strategies[index].reconfigure_preserving_position(updated.clone()) {
            self.strategies[index] = build_strategy(updated.clone());
        }
        Some(updated)
    }
}

impl Default for StrategyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;

    #[test]
    fn live_high_excludes_unconfirmed_latest_bar_without_shortening_252_requirement() {
        let mut manager = StrategyManager::new();
        manager.add(build_strategy(StrategyConfig::new(
            "fifty_two_week_high",
            "fixture",
            true,
            vec!["SOXQ".into()],
            1,
            serde_json::to_value(super::super::FiftyTwoWeekHighParams::default()).unwrap(),
        )));
        let completed = OhlcCandle {
            open: 10_000,
            high: 10_000,
            low: 10_000,
            close: 10_000,
        };
        let mut live = vec![completed; 252];
        live.push(OhlcCandle {
            high: 30_000,
            ..completed
        });
        manager.initialize_warmup("SOXQ", &live, &[]);
        assert_eq!(
            manager.strategies[0]
                .history_readiness("SOXQ")
                .unwrap()
                .available_bars,
            252
        );
        assert!(matches!(
            manager.on_tick("SOXQ", 12_000, 1000)[0].signal,
            Signal::Buy { .. }
        ));
        manager.initialize_warmup("SOXQ", &live[..252], &[]);
        let state = manager.strategies[0].history_readiness("SOXQ").unwrap();
        assert_eq!(state.available_bars, 251);
        assert!(!state.ready);
        assert!(matches!(
            manager.on_tick("SOXQ", 11_000, 1000)[0].signal,
            Signal::Sell { .. }
        ));
        manager.sync_position("SOXQ", 1, 12_000);
        manager.initialize_warmup("SOXQ", &live[..1], &[]);
        let state = manager.strategies[0].history_readiness("SOXQ").unwrap();
        assert_eq!(state.available_bars, 0);
        assert!(!state.ready);
        assert!(matches!(
            manager.on_tick("SOXQ", 11_000, 1000)[0].signal,
            Signal::Sell { .. }
        ));
    }
}

#[cfg(test)]
#[path = "manager/live_feedback_tests.rs"]
mod live_feedback_tests;

fn owned_target_matches(
    strategy: &dyn Strategy,
    strategy_id: &str,
    scope: &BrokerScope,
    market: BrokerMarket,
    symbol: &str,
) -> bool {
    strategy.id() == strategy_id
        && strategy.config().matches_scope(
            scope.broker_id,
            scope.account_id.as_ref().map(|id| id.0.as_str()),
        )
        && crate::trading::configuration::effective_target_markets(strategy.config()).get(symbol)
            == Some(&(market == BrokerMarket::Us))
}
