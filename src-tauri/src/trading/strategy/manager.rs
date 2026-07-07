use crate::broker::{BrokerId, BrokerMarket};

use super::{
    BrokerPositionSnapshot, ConsecutiveMoveStrategy, DeviationStrategy, FailedBreakoutStrategy,
    FiftyTwoWeekHighStrategy, LeveragedTrendHoldStrategy, MeanReversionStrategy, MomentumStrategy,
    MovingAverageCrossStrategy, OhlcCandle, PriceConditionStrategy, RsiStrategy, Signal, Strategy,
    StrategyConfig, StrategySignal, StrongCloseStrategy, TrendFilterStrategy,
    VolatilityExpansionStrategy,
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
}

impl StrategyManager {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
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
        self.strategies
            .iter_mut()
            .filter(|s| allow(s.config()))
            .filter_map(|s| {
                let signal = s.on_tick(symbol, price, volume);
                if signal == Signal::Hold {
                    None
                } else {
                    Some(StrategySignal {
                        strategy_id: s.id().to_string(),
                        signal,
                    })
                }
            })
            .collect()
    }

    pub fn any_active_config_for_symbol<F>(&self, symbol: &str, mut predicate: F) -> bool
    where
        F: FnMut(&StrategyConfig) -> bool,
    {
        self.strategies.iter().any(|strategy| {
            let config = strategy.config();
            strategy.is_enabled() && config.targets_symbol(symbol) && predicate(config)
        })
    }

    pub fn reset_all(&mut self) {
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

    /// 특정 종목을 타겟으로 하는 모든 전략에 일봉 변동 범위(고가-저가) 데이터 전달 (변동성 확장 전략)
    pub fn initialize_range_data(&mut self, symbol: &str, ranges: &[u64]) {
        for s in &mut self.strategies {
            if s.config().targets_symbol(symbol) {
                s.initialize_range_data(symbol, ranges);
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
        // 1) 모든 전략 기본값으로 초기화 (프로필 전환 시 이전 상태 잔류 방지)
        for s in &mut self.strategies {
            let cfg = s.config_mut();
            cfg.enabled = false;
            cfg.target_symbols = Vec::new();
            cfg.set_scope(broker_id, broker_account_id.clone());
        }
        // 2) 저장된 설정 적용
        for saved_cfg in saved {
            if let Some(cfg) = self.get_config_mut(&saved_cfg.id) {
                cfg.enabled = saved_cfg.enabled;
                cfg.target_symbols = saved_cfg.target_symbols.clone();
                cfg.order_quantity = saved_cfg.order_quantity;
                cfg.params = saved_cfg.params.clone();
                cfg.set_scope(broker_id, broker_account_id.clone());
            }
        }
        for strategy in &mut self.strategies {
            let config = strategy.config().clone();
            *strategy = build_strategy(config);
        }
    }

    /// 전체 전략 설정 반환
    pub fn all_configs(&self) -> Vec<&StrategyConfig> {
        self.strategies.iter().map(|s| s.config()).collect()
    }

    /// 특정 ID의 전략 설정 가변 참조 반환
    pub fn get_config_mut(&mut self, id: &str) -> Option<&mut StrategyConfig> {
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
        let previous_params = self.strategies[index].config().params.clone();
        let previous_targets = self.strategies[index].config().target_symbols.clone();
        {
            let cfg = self.strategies[index].config_mut();
            update(cfg);
        }
        let updated = self.strategies[index].config().clone();
        if previous_params != updated.params || previous_targets != updated.target_symbols {
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
