use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 단일 종목 포지션
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub symbol_name: String,
    /// 보유 수량
    pub quantity: u64,
    /// 평균 매입 단가
    pub avg_price: f64,
    /// 최근 현재가
    pub current_price: u64,
}

impl Position {
    pub fn new(symbol: String, symbol_name: String, quantity: u64, price: u64) -> Self {
        Self {
            symbol,
            symbol_name,
            quantity,
            avg_price: price as f64,
            current_price: price,
        }
    }

    /// 매수 추가
    pub fn add_buy(&mut self, quantity: u64, price: u64) {
        let total_cost = self.avg_price * self.quantity as f64 + price as f64 * quantity as f64;
        self.quantity += quantity;
        self.avg_price = total_cost / self.quantity as f64;
    }

    /// 매도
    pub fn reduce(&mut self, quantity: u64) {
        self.quantity = self.quantity.saturating_sub(quantity);
    }

    /// 평가손익 (원)
    pub fn unrealized_pnl(&self) -> i64 {
        (self.current_price as i64 - self.avg_price as i64) * self.quantity as i64
    }

    /// 평가손익률 (%)
    pub fn unrealized_pnl_rate(&self) -> f64 {
        if self.avg_price == 0.0 {
            return 0.0;
        }
        (self.current_price as f64 - self.avg_price) / self.avg_price * 100.0
    }
}

/// 전체 포지션 추적기
pub struct PositionTracker {
    positions: HashMap<String, Position>,
}

impl PositionTracker {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    /// 매수 체결 반영
    pub fn on_buy(&mut self, symbol: String, symbol_name: String, quantity: u64, price: u64) {
        self.positions
            .entry(symbol.clone())
            .and_modify(|p| p.add_buy(quantity, price))
            .or_insert_with(|| Position::new(symbol, symbol_name, quantity, price));
    }

    /// 매도 체결 반영
    pub fn on_sell(&mut self, symbol: &str, quantity: u64) {
        if let Some(pos) = self.positions.get_mut(symbol) {
            pos.reduce(quantity);
            if pos.quantity == 0 {
                self.positions.remove(symbol);
            }
        }
    }

    /// 현재가 업데이트
    pub fn update_price(&mut self, symbol: &str, price: u64) {
        if let Some(pos) = self.positions.get_mut(symbol) {
            pos.current_price = price;
        }
    }

    /// 전체 포지션 목록
    pub fn all(&self) -> Vec<&Position> {
        self.positions.values().collect()
    }

    /// 특정 종목 포지션
    pub fn get(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// 신뢰할 수 있는 브로커 잔고 스냅샷으로 전체 포지션을 교체한다.
    ///
    /// 입력: `(symbol, name, qty, avg_price, current_price)` 이터레이터
    /// - 국내: avg_price/current_price = KRW 정수
    /// - 해외: avg_price/current_price = USD × 100 (센트 정수화)
    pub fn replace<I>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (String, String, u64, u64, u64)>,
    {
        self.positions.clear();
        for (symbol, name, qty, avg_price, current_price) in entries {
            if qty == 0 {
                continue;
            }
            let mut pos = Position::new(symbol.clone(), name, qty, avg_price);
            pos.current_price = current_price;
            self.positions.insert(symbol, pos);
        }
    }

    /// 전체 포지션 초기화. broker/account 전환 시 이전 scope 스냅샷이 새어들지 않게 한다.
    pub fn clear(&mut self) {
        self.positions.clear();
    }

    /// 총 평가손익
    pub fn total_pnl(&self) -> i64 {
        self.positions.values().map(|p| p.unrealized_pnl()).sum()
    }

    /// 보유 종목 수
    pub fn count(&self) -> usize {
        self.positions.len()
    }
}

impl Default for PositionTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// 해외 단일 종목 포지션.
///
/// 가격은 자동매매 내부 단위와 동일하게 USD × 100(cents) 정수로 저장한다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverseasPosition {
    pub symbol: String,
    pub symbol_name: String,
    pub exchange: String,
    pub quantity: u64,
    pub avg_price_cents: f64,
    pub current_price_cents: u64,
}

impl OverseasPosition {
    pub fn new(
        symbol: String,
        symbol_name: String,
        exchange: String,
        quantity: u64,
        price_cents: u64,
    ) -> Self {
        Self {
            symbol,
            symbol_name,
            exchange,
            quantity,
            avg_price_cents: price_cents as f64,
            current_price_cents: price_cents,
        }
    }

    pub fn add_buy(&mut self, quantity: u64, price_cents: u64) {
        let total_cost =
            self.avg_price_cents * self.quantity as f64 + price_cents as f64 * quantity as f64;
        self.quantity += quantity;
        self.avg_price_cents = total_cost / self.quantity as f64;
    }

    pub fn reduce(&mut self, quantity: u64) {
        self.quantity = self.quantity.saturating_sub(quantity);
    }

    pub fn unrealized_pnl_cents(&self) -> i64 {
        (self.current_price_cents as i64 - self.avg_price_cents as i64) * self.quantity as i64
    }
}

/// 해외 포지션 추적기.
///
/// 국내 `PositionTracker`와 분리해 USD cents, 거래소 코드, 해외 보유 수량을 보존한다.
pub struct OverseasPositionTracker {
    positions: HashMap<String, OverseasPosition>,
}

impl OverseasPositionTracker {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    pub fn on_buy(
        &mut self,
        symbol: String,
        symbol_name: String,
        exchange: String,
        quantity: u64,
        price_cents: u64,
    ) {
        self.positions
            .entry(symbol.clone())
            .and_modify(|p| {
                p.exchange = exchange.clone();
                p.add_buy(quantity, price_cents);
            })
            .or_insert_with(|| {
                OverseasPosition::new(symbol, symbol_name, exchange, quantity, price_cents)
            });
    }

    pub fn on_sell(&mut self, symbol: &str, quantity: u64) {
        if let Some(pos) = self.positions.get_mut(symbol) {
            pos.reduce(quantity);
            if pos.quantity == 0 {
                self.positions.remove(symbol);
            }
        }
    }

    pub fn update_price(&mut self, symbol: &str, price_cents: u64) {
        if let Some(pos) = self.positions.get_mut(symbol) {
            pos.current_price_cents = price_cents;
        }
    }

    pub fn all(&self) -> Vec<&OverseasPosition> {
        self.positions.values().collect()
    }

    pub fn get(&self, symbol: &str) -> Option<&OverseasPosition> {
        self.positions.get(symbol)
    }

    /// 전체 포지션 초기화. broker/account 전환 시 이전 scope 스냅샷이 새어들지 않게 한다.
    pub fn clear(&mut self) {
        self.positions.clear();
    }

    /// 신뢰할 수 있는 해외 브로커 잔고 스냅샷으로 전체 포지션을 교체한다.
    ///
    /// 입력: `(symbol, name, exchange, qty, avg_price_cents, current_price_cents)` 이터레이터
    pub fn replace<I>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (String, String, String, u64, u64, u64)>,
    {
        self.positions.clear();
        for (symbol, name, exchange, qty, avg_price_cents, current_price_cents) in entries {
            if qty == 0 {
                continue;
            }
            let mut pos =
                OverseasPosition::new(symbol.clone(), name, exchange, qty, avg_price_cents);
            pos.current_price_cents = current_price_cents;
            self.positions.insert(symbol, pos);
        }
    }
}

impl Default for OverseasPositionTracker {
    fn default() -> Self {
        Self::new()
    }
}
