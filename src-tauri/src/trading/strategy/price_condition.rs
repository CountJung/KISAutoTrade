use serde::{Deserialize, Serialize};

use super::{Signal, Strategy, StrategyConfig};

// ────────────────────────────────────────────────────────────────────
// 11. 가격 조건 매매 전략 (PriceConditionStrategy) — 종목별 독립 설정
// ────────────────────────────────────────────────────────────────────
// 각 종목마다 매수가·익절가·익절%·손절%·수량을 독립적으로 설정한다.
// 동작 (종목별):
//  1. buy_trigger_price > 0 && 미포지션 && 현재가 ≤ buy_trigger_price → 매수
//  2. 포지션 보유 중, 다음 중 먼저 충족되는 조건에서 매도:
//     a) stop_loss_pct > 0 && 손실률 ≥ stop_loss_pct (최우선 — 손절)
//     b) sell_trigger_price > 0 && 현재가 ≥ sell_trigger_price (지정가 익절)
//     c) take_profit_pct > 0 && 수익률 ≥ take_profit_pct (비율 익절)
// ────────────────────────────────────────────────────────────────────

fn pc_default_qty() -> u64 {
    1
}
fn pc_default_tp() -> f64 {
    5.0
}
fn pc_default_sl() -> f64 {
    3.0
}

/// 가격 조건 매매 — 종목별 개별 설정 단위
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceConditionSymbolConfig {
    /// 종목코드
    pub symbol: String,
    /// 종목명 (UI 표시용)
    #[serde(default)]
    pub symbol_name: String,
    /// 1회 주문 수량 (종목별 독립)
    #[serde(default = "pc_default_qty")]
    pub quantity: u64,
    /// 매수 트리거가.
    /// - 국내(is_overseas=false): 원화 정수 (e.g. 55000)
    /// - 해외(is_overseas=true) : USD face value (e.g. 620.5)
    ///   on_tick에서 ×100(cents)으로 변환 후 비교
    ///   0이면 비활성.
    #[serde(default)]
    pub buy_trigger_price: f64,
    /// 지정 익절가. 단위는 buy_trigger_price와 동일. 0이면 비활성.
    #[serde(default)]
    pub sell_trigger_price: f64,
    /// % 익절 기준. 0이면 비활성.
    #[serde(default = "pc_default_tp")]
    pub take_profit_pct: f64,
    /// % 손절 기준. 0이면 비활성.
    #[serde(default = "pc_default_sl")]
    pub stop_loss_pct: f64,
    /// 해외 주식 여부. true이면 가격 단위 = USD (on_tick에서 ×100 변환)
    #[serde(default)]
    pub is_overseas: bool,
}

/// 가격 조건 매매 전략 파라미터 (종목 목록)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceConditionParams {
    #[serde(default)]
    pub symbols: Vec<PriceConditionSymbolConfig>,
}

pub struct PriceConditionStrategy {
    config: StrategyConfig,
    params: PriceConditionParams,
    /// symbol → (in_position, entry_price)
    positions: std::collections::HashMap<String, (bool, Option<u64>)>,
    /// params 변경 감지를 위한 마지막 파싱 기준 JSON
    last_params: serde_json::Value,
}

impl PriceConditionStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: PriceConditionParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        let last_params = config.params.clone();
        Self {
            config,
            params,
            positions: std::collections::HashMap::new(),
            last_params,
        }
    }

    /// config.params가 변경됐을 때 params 재파싱 + target_symbols 동기화
    fn sync_params(&mut self) {
        if self.config.params != self.last_params {
            self.params = serde_json::from_value(self.config.params.clone()).unwrap_or_default();
            self.last_params = self.config.params.clone();
            // target_symbols를 params.symbols 기반으로 자동 갱신 (engine 구독 목록 일치)
            self.config.target_symbols = self
                .params
                .symbols
                .iter()
                .map(|s| s.symbol.clone())
                .collect();
        }
    }
}

impl Strategy for PriceConditionStrategy {
    fn id(&self) -> &str {
        &self.config.id
    }
    fn name(&self) -> &str {
        &self.config.name
    }
    fn config(&self) -> &StrategyConfig {
        &self.config
    }
    fn config_mut(&mut self) -> &mut StrategyConfig {
        &mut self.config
    }
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        self.sync_params();

        let sym_cfg = match self.params.symbols.iter().find(|s| s.symbol == symbol) {
            Some(s) => s.clone(),
            None => return Signal::Hold,
        };

        // 해외 종목: on_tick price = USD×100(cents). 저장된 트리거가도 ×100으로 스케일 맞춤
        // 국내 종목: on_tick price = KRW 정수. 저장값 그대로 사용
        let scale: f64 = if sym_cfg.is_overseas { 100.0 } else { 1.0 };
        let unit: &str = if sym_cfg.is_overseas { "USD" } else { "원" };
        let buy_thresh = (sym_cfg.buy_trigger_price * scale).round() as u64;
        let sell_thresh = (sym_cfg.sell_trigger_price * scale).round() as u64;

        // 표시용 가격 변환 (cents → USD, 또는 KRW 그대로)
        let to_disp = |p: u64| -> f64 { p as f64 / scale };

        let pos = self
            .positions
            .entry(symbol.to_string())
            .or_insert((false, None));

        if pos.0 {
            let ep = match pos.1 {
                Some(v) => v,
                None => return Signal::Hold,
            };

            // 1) 손절 최우선
            if sym_cfg.stop_loss_pct > 0.0 && price < ep {
                let loss_pct = (ep as f64 - price as f64) / ep as f64 * 100.0;
                if loss_pct >= sym_cfg.stop_loss_pct {
                    pos.0 = false;
                    pos.1 = None;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: sym_cfg.quantity,
                        reason: format!(
                            "가격조건 손절: -{:.1}% ({:.2}{unit} → {:.2}{unit})",
                            loss_pct,
                            to_disp(ep),
                            to_disp(price)
                        ),
                    };
                }
            }

            // 2) 지정가 익절
            if sell_thresh > 0 && price >= sell_thresh {
                pos.0 = false;
                pos.1 = None;
                return Signal::Sell {
                    symbol: symbol.to_string(),
                    quantity: sym_cfg.quantity,
                    reason: format!(
                        "지정가 익절: {:.2}{unit} ≥ 목표 {:.2}{unit}",
                        to_disp(price),
                        sym_cfg.sell_trigger_price
                    ),
                };
            }

            // 3) % 익절
            if sym_cfg.take_profit_pct > 0.0 && price > ep {
                let profit_pct = (price as f64 - ep as f64) / ep as f64 * 100.0;
                if profit_pct >= sym_cfg.take_profit_pct {
                    pos.0 = false;
                    pos.1 = None;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: sym_cfg.quantity,
                        reason: format!(
                            "비율 익절: +{:.1}% ({:.2}{unit} → {:.2}{unit})",
                            profit_pct,
                            to_disp(ep),
                            to_disp(price)
                        ),
                    };
                }
            }

            return Signal::Hold;
        }

        // 미포지션: 매수 조건
        if buy_thresh > 0 && price <= buy_thresh {
            pos.0 = true;
            pos.1 = Some(price);
            return Signal::Buy {
                symbol: symbol.to_string(),
                quantity: sym_cfg.quantity,
                reason: format!(
                    "가격조건 매수: {:.2}{unit} ≤ 트리거 {:.2}{unit}",
                    to_disp(price),
                    sym_cfg.buy_trigger_price
                ),
            };
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        self.sync_params();
        if quantity == 0 || !self.params.symbols.iter().any(|s| s.symbol == symbol) {
            return;
        }
        self.positions
            .insert(symbol.to_string(), (true, Some(avg_price)));
        tracing::info!(
            "가격 조건 전략 포지션 동기화: {} {}주 @ {}",
            symbol,
            quantity,
            avg_price
        );
    }

    fn reset(&mut self) {
        self.positions.clear();
    }
}
