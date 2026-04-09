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

    /// 잔고 API 응답으로부터 포지션 초기화 (비어있을 때만 적용)
    ///
    /// 앱 재시작 시 in-memory tracker가 비어있을 때 KIS 잔고 응답으로 복원한다.
    /// 이미 포지션이 있으면 세션 상태를 보존하기 위해 아무것도 하지 않는다.
    ///
    /// 입력: `(symbol, name, qty, avg_price, current_price)` 이터레이터
    /// - 국내: avg_price/current_price = KRW 정수
    /// - 해외: avg_price/current_price = USD × 100 (센트 정수화)
    pub fn load_if_empty<I>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (String, String, u64, u64, u64)>,
    {
        if !self.positions.is_empty() {
            return; // 이미 포지션이 있으면 세션 상태 유지
        }
        for (symbol, name, qty, avg_price, current_price) in entries {
            if qty == 0 {
                continue;
            }
            let mut pos = Position::new(symbol.clone(), name, qty, avg_price);
            pos.current_price = current_price;
            self.positions.insert(symbol, pos);
        }
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

