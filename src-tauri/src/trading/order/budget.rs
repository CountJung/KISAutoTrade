//! Dedicated capital ledger. Integer native units (KRW / USD cents); mutations are
//! serialized by OrderManager and durably saved before a provider can consume them.
use super::*;
use std::collections::BTreeMap;

const MAX_AMOUNT: u64 = 9_007_199_254_740_991;
const FEE_BPS: u32 = 100;

#[derive(Clone, Default, Serialize, Deserialize)]
pub(super) struct BudgetLedger {
    accounts: Vec<BudgetAccount>,
}
#[derive(Clone, Serialize, Deserialize)]
struct BudgetAccount {
    scope: BrokerScope,
    krw: CurrencyLedger,
    usd: CurrencyLedger,
}
#[derive(Clone, Default, Serialize, Deserialize)]
struct CurrencyLedger {
    allocated: u64,
    cash: u64,
    positions: BTreeMap<String, u64>,
    reservations: BTreeMap<String, BudgetReservation>,
    blocked: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct BudgetReservation {
    symbol: String,
    buy: bool,
    quantity: u64,
    price: u64,
    filled: u64,
    notional: u64,
    reserved: u64,
    accepted: bool,
    #[serde(default)]
    terminal: bool,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTradingBudgetView {
    pub scope: BrokerScope,
    pub krw: BudgetCurrencyView,
    pub usd: BudgetCurrencyView,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetCurrencyView {
    pub allocated_amount: u64,
    pub cash_amount: u64,
    pub reserved_amount: u64,
    pub available_amount: u64,
    pub owned_position_count: usize,
    pub blocked_reason: Option<String>,
    pub fee_buffer_bps: u32,
}
fn cost(notional: u64) -> Result<u64> {
    let value =
        u128::from(notional) + (u128::from(notional) * u128::from(FEE_BPS)).div_ceil(10_000);
    anyhow::ensure!(
        value <= u128::from(MAX_AMOUNT),
        "자동매매 금액이 허용 범위를 초과합니다."
    );
    Ok(value as u64)
}
impl CurrencyLedger {
    fn reserved(&self) -> u64 {
        self.reservations.values().map(|r| r.reserved).sum()
    }
    fn blocked_reason(&self) -> Option<String> {
        self.blocked.clone().or_else(|| self.reservations.values().any(|r| !r.accepted && !r.terminal)
            .then(|| "접수 여부가 확인되지 않은 자동매매 주문이 있어 예산을 보류합니다. 주문 내역을 확인하세요.".into()))
    }
    fn view(&self) -> BudgetCurrencyView {
        let blocked_reason = self.blocked_reason();
        BudgetCurrencyView {
            allocated_amount: self.allocated,
            cash_amount: self.cash,
            reserved_amount: self.reserved(),
            available_amount: if blocked_reason.is_some() {
                0
            } else {
                self.cash.saturating_sub(self.reserved())
            },
            owned_position_count: self.positions.values().filter(|q| **q > 0).count(),
            blocked_reason,
            fee_buffer_bps: FEE_BPS,
        }
    }
    fn allocate(&mut self, amount: u64) -> Result<()> {
        anyhow::ensure!(
            amount <= MAX_AMOUNT,
            "자동매매 배정 금액이 허용 범위를 초과합니다."
        );
        anyhow::ensure!(
            self.blocked_reason().is_none(),
            "미확인 주문 또는 원장 장애를 먼저 확인해야 합니다."
        );
        if amount >= self.allocated {
            self.cash = self
                .cash
                .checked_add(amount - self.allocated)
                .filter(|v| *v <= MAX_AMOUNT)
                .ok_or_else(|| anyhow::anyhow!("자동매매 현금 한도 초과"))?;
        } else {
            let withdrawal = self.allocated - amount;
            anyhow::ensure!(
                withdrawal <= self.cash.saturating_sub(self.reserved()),
                "보유 주식·미체결 주문에 사용 중인 자동매매 금액은 줄일 수 없습니다."
            );
            self.cash -= withdrawal;
        }
        self.allocated = amount;
        Ok(())
    }
}
impl BudgetLedger {
    fn account(&mut self, scope: &BrokerScope) -> &mut BudgetAccount {
        let index = self
            .accounts
            .iter()
            .position(|a| &a.scope == scope)
            .unwrap_or_else(|| {
                self.accounts.push(BudgetAccount {
                    scope: scope.clone(),
                    krw: CurrencyLedger::default(),
                    usd: CurrencyLedger::default(),
                });
                self.accounts.len() - 1
            });
        &mut self.accounts[index]
    }
    fn currency(&mut self, scope: &BrokerScope, usd: bool) -> &mut CurrencyLedger {
        let a = self.account(scope);
        if usd {
            &mut a.usd
        } else {
            &mut a.krw
        }
    }
}
impl OrderManager {
    async fn load_budget(&mut self) -> Result<()> {
        {
            // Re-read through storage routing so a stopped backend switch cannot reuse stale capital.
            // A previous JSON backup may predate a submitted order. Never roll capital back.
            let loaded: Result<BudgetLedger> = async {
                let content = crate::storage::database::read_managed_json(
                    &self.pending_order_store.budget_path(),
                )
                .await?;
                match content {
                    Some(text) => Ok(serde_json::from_str(&text)?),
                    None => Ok(BudgetLedger::default()),
                }
            }
            .await;
            match loaded {
                Ok(mut ledger) => {
                    let ledger: &mut BudgetLedger = &mut ledger;
                    for a in &mut ledger.accounts {
                        for c in [&mut a.krw, &mut a.usd] {
                            c.reservations.retain(|id, r| {
                                !r.terminal || self.pending.values().any(|p| &p.record.id == id)
                            });
                            for r in c.reservations.values_mut() {
                                r.accepted = false;
                            }
                        }
                    }
                    for pending in self
                        .pending
                        .values()
                        .filter(|p| p.record.id.starts_with("auto-budget-"))
                    {
                        let c = ledger.currency(
                            &pending.broker_scope,
                            pending.exchange.is_some()
                                || !crate::market_hours::is_domestic_symbol(&pending.record.symbol),
                        );
                        if let Some(r) = c.reservations.get_mut(&pending.record.id) {
                            r.accepted = true;
                        } else {
                            c.blocked =
                                Some("미체결 자동매매 주문의 예산 예약이 유실되었습니다.".into());
                        }
                    }
                    self.budget = Some(ledger.clone());
                }
                Err(error) => {
                    self.block_for_persistence_failure(format!(
                        "자동매매 예산 원장 읽기 실패: {error}"
                    ));
                    return Err(error);
                }
            }
        }
        Ok(())
    }
    async fn save_budget(&mut self, ledger: BudgetLedger) -> Result<()> {
        if let Err(error) =
            crate::storage::write_json(&self.pending_order_store.budget_path(), &ledger).await
        {
            self.block_for_persistence_failure(format!("자동매매 예산 원장 저장 실패: {error}"));
            return Err(error);
        }
        self.budget = Some(ledger);
        Ok(())
    }
    pub async fn auto_trading_budget(
        &mut self,
        scope: &BrokerScope,
    ) -> Result<AutoTradingBudgetView> {
        self.load_budget().await?;
        let a = self.budget.as_mut().expect("loaded ledger").account(scope);
        Ok(AutoTradingBudgetView {
            scope: scope.clone(),
            krw: a.krw.view(),
            usd: a.usd.view(),
        })
    }
    pub async fn update_auto_trading_budget(
        &mut self,
        scope: &BrokerScope,
        krw_amount: u64,
        usd_amount: u64,
    ) -> Result<AutoTradingBudgetView> {
        self.load_budget().await?;
        anyhow::ensure!(
            scope.account_id.is_some(),
            "자동매매 예산은 명시적인 계좌가 필요합니다."
        );
        let mut ledger = self.budget.clone().expect("loaded ledger");
        let a = ledger.account(scope);
        a.krw.allocate(krw_amount)?;
        a.usd.allocate(usd_amount)?;
        self.save_budget(ledger).await?;
        self.auto_trading_budget(scope).await
    }
    pub(super) async fn budget_owned(
        &mut self,
        scope: &BrokerScope,
        symbol: &str,
        usd: bool,
    ) -> Result<u64> {
        self.load_budget().await?;
        Ok(*self
            .budget
            .as_mut()
            .expect("loaded ledger")
            .currency(scope, usd)
            .positions
            .get(symbol)
            .unwrap_or(&0))
    }
    pub(super) async fn reserve_budget(
        &mut self,
        scope: &BrokerScope,
        symbol: &str,
        usd: bool,
        buy: bool,
        quantity: u64,
        price: u64,
    ) -> Result<String> {
        self.load_budget().await?;
        let mut ledger = self.budget.clone().expect("loaded ledger");
        let c = ledger.currency(scope, usd);
        anyhow::ensure!(
            c.blocked_reason().is_none(),
            "자동매매 예산에 미확인 주문 또는 원장 장애가 있습니다."
        );
        anyhow::ensure!(
            quantity > 0 && price > 0,
            "자동매매 주문 수량·지정가가 유효하지 않습니다."
        );
        let reserved = if buy {
            anyhow::ensure!(c.allocated > 0, "자동매매 배정 금액이 0입니다.");
            let notional = price
                .checked_mul(quantity)
                .ok_or_else(|| anyhow::anyhow!("주문 금액 초과"))?;
            let amount = cost(price)?
                .checked_mul(quantity)
                .filter(|v| *v <= MAX_AMOUNT)
                .ok_or_else(|| anyhow::anyhow!("자동매매 주문 금액 한도 초과"))?;
            let _ = notional;
            anyhow::ensure!(
                amount <= c.cash.saturating_sub(c.reserved()),
                "자동매매 전용 거래가능금액이 부족합니다 (비용 여유 1% 포함)."
            );
            amount
        } else {
            let pending_qty: u64 = c
                .reservations
                .values()
                .filter(|r| !r.buy && r.symbol == symbol)
                .map(|r| r.quantity - r.filled)
                .sum();
            anyhow::ensure!(
                quantity
                    <= c.positions
                        .get(symbol)
                        .copied()
                        .unwrap_or(0)
                        .saturating_sub(pending_qty),
                "자동매매 전용 보유수량이 부족합니다."
            );
            0
        };
        let id = format!("auto-budget-{}", uuid::Uuid::new_v4());
        c.reservations.insert(
            id.clone(),
            BudgetReservation {
                symbol: symbol.into(),
                buy,
                quantity,
                price,
                filled: 0,
                notional: 0,
                reserved,
                accepted: false,
                terminal: false,
            },
        );
        self.save_budget(ledger).await?;
        Ok(id)
    }
    pub(super) async fn reject_budget(&mut self, id: &str) -> Result<()> {
        self.load_budget().await?;
        let mut ledger = self.budget.clone().expect("loaded ledger");
        for a in &mut ledger.accounts {
            a.krw.reservations.remove(id);
            a.usd.reservations.remove(id);
        }
        self.save_budget(ledger).await
    }
    pub(super) async fn accept_budget(&mut self, pending: &PendingOrder) -> Result<()> {
        if !pending.record.id.starts_with("auto-budget-") {
            return Ok(());
        }
        self.load_budget().await?;
        let mut ledger = self.budget.clone().expect("loaded ledger");
        let c = ledger.currency(
            &pending.broker_scope,
            pending.exchange.is_some()
                || !crate::market_hours::is_domestic_symbol(&pending.record.symbol),
        );
        let r = c
            .reservations
            .get_mut(&pending.record.id)
            .ok_or_else(|| anyhow::anyhow!("자동매매 주문의 예산 예약이 없습니다."))?;
        r.accepted = true;
        self.save_budget(ledger).await
    }
    pub(super) async fn release_budget(&mut self, pending: &PendingOrder) -> Result<()> {
        if !pending.record.id.starts_with("auto-budget-") {
            return Ok(());
        }
        self.load_budget().await?;
        let mut ledger = self.budget.clone().expect("loaded ledger");
        let c = ledger.currency(
            &pending.broker_scope,
            pending.exchange.is_some()
                || !crate::market_hours::is_domestic_symbol(&pending.record.symbol),
        );
        if let Some(reservation) = c.reservations.get_mut(&pending.record.id) {
            reservation.terminal = true;
            reservation.reserved = 0;
        }
        self.save_budget(ledger).await
    }
    pub(super) async fn apply_budget_fill(
        &mut self,
        pending: &PendingOrder,
        cumulative: u64,
        notional: u128,
    ) -> Result<()> {
        if !pending.record.id.starts_with("auto-budget-") {
            return Ok(());
        }
        self.load_budget().await?;
        let mut ledger = self.budget.clone().expect("loaded ledger");
        let c = ledger.currency(
            &pending.broker_scope,
            pending.exchange.is_some()
                || !crate::market_hours::is_domestic_symbol(&pending.record.symbol),
        );
        let r = c
            .reservations
            .get_mut(&pending.record.id)
            .ok_or_else(|| anyhow::anyhow!("체결 주문의 자동매매 예산 예약이 없습니다."))?;
        anyhow::ensure!(
            cumulative <= r.quantity,
            "자동매매 체결수량이 주문수량을 초과합니다."
        );
        if r.terminal || cumulative <= r.filled {
            return Ok(());
        }
        anyhow::ensure!(
            notional <= u128::from(MAX_AMOUNT) && notional >= u128::from(r.notional),
            "자동매매 누적 체결금액 오류"
        );
        let amount = notional as u64 - r.notional;
        let qty = cumulative - r.filled;
        if r.buy {
            let debit = cost(notional as u64)? - cost(r.notional)?;
            if debit > c.cash || notional > u128::from(r.price) * u128::from(cumulative) {
                c.blocked = Some(
                    "자동매매 체결금액이 예약 범위를 초과했습니다. 계좌 대조가 필요합니다.".into(),
                );
            }
            c.cash = c.cash.saturating_sub(debit);
            *c.positions.entry(r.symbol.clone()).or_default() += qty;
            r.reserved = cost(r.price)?
                .checked_mul(r.quantity - cumulative)
                .ok_or_else(|| anyhow::anyhow!("예약금 계산 오류"))?;
        } else {
            let fee = cost(notional as u64)? - notional as u64 - (cost(r.notional)? - r.notional);
            c.cash = c
                .cash
                .checked_add(amount.saturating_sub(fee))
                .filter(|v| *v <= MAX_AMOUNT)
                .ok_or_else(|| anyhow::anyhow!("자동매매 현금 범위 초과"))?;
            let owned = c.positions.entry(r.symbol.clone()).or_default();
            anyhow::ensure!(*owned >= qty, "자동매매 매도 체결 보유수량 불일치");
            *owned -= qty;
        }
        c.positions.retain(|_, quantity| *quantity > 0);
        r.filled = cumulative;
        r.notional = notional as u64;
        r.accepted = true;
        self.save_budget(ledger).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::token::TokenManager,
        broker::{BrokerAccountId, BrokerId},
        config::AppConfig,
    };
    fn manager(path: std::path::PathBuf) -> OrderManager {
        let config = Arc::new(AppConfig {
            broker_id: BrokerId::Kis,
            broker_account_id: "test".into(),
            kis_app_key: String::new(),
            kis_app_secret: String::new(),
            kis_account_no: String::new(),
            kis_is_paper_trading: true,
            discord_bot_token: None,
            discord_channel_id: None,
            notification_levels: vec![],
        });
        let client = Arc::new(KisRestClient::new(
            "http://127.0.0.1:1".into(),
            String::new(),
            String::new(),
            String::new(),
            true,
            Arc::new(RwLock::new(TokenManager::new(config))),
        ));
        OrderManager::new(
            Arc::new(RwLock::new(client)),
            Arc::new(RwLock::new(ProfilesConfig::default())),
            Arc::new(OrderStore::new(path.clone())),
            Arc::new(PendingOrderStore::new(path.clone())),
            Arc::new(TradeStore::new(path.clone())),
            Arc::new(Mutex::new(PositionTracker::new())),
            Arc::new(Mutex::new(OverseasPositionTracker::new())),
            Arc::new(StatsStore::new(path.clone())),
            Arc::new(RwLock::new(1400.0)),
            Arc::new(Mutex::new(RiskManager::default())),
            Arc::new(RiskStore::new(path)),
            None,
        )
    }
    fn scope(account: &str) -> BrokerScope {
        BrokerScope::new(BrokerId::Kis, Some(BrokerAccountId(account.into())))
    }
    fn pending(id: String, scope: BrokerScope, buy: bool) -> PendingOrder {
        let mut record = OrderRecord::new(
            "005930".into(),
            "test".into(),
            if buy { OrderSide::Buy } else { OrderSide::Sell },
            10,
            100,
            "Limit".into(),
        );
        record.id = id;
        record.kis_order_id = Some("test-order".into());
        PendingOrder {
            record,
            signal_reason: "test".into(),
            strategy_id: None,
            signal_price: 100,
            order_price: 100,
            exchange: None,
            broker_scope: scope,
            filled_quantity: 0,
            filled_notional: 0,
            confirmed_filled_quantity: 0,
            confirmed_avg_price: 0,
            application_started: false,
            application_pnl: None,
            client_order_id: None,
            provider_status: None,
        }
    }
    #[tokio::test]
    async fn budget_reservation_fill_restart_and_cancel_are_isolated_and_idempotent() {
        let path = std::env::temp_dir().join(format!("budget-test-{}", uuid::Uuid::new_v4()));
        let mut m = manager(path.clone());
        let s = scope("one");
        assert!(m
            .reserve_budget(&s, "005930", false, true, 10, 100)
            .await
            .is_err());
        m.update_auto_trading_budget(&s, 2020, 500).await.unwrap();
        let id = m
            .reserve_budget(&s, "005930", false, true, 10, 100)
            .await
            .unwrap();
        // Unknown response keeps cash unavailable, including after process recreation.
        let mut restarted = manager(path.clone());
        assert!(restarted
            .auto_trading_budget(&s)
            .await
            .unwrap()
            .krw
            .blocked_reason
            .is_some());
        let p = pending(id, s.clone(), true);
        m.track_pending_order("test-order".into(), p.clone());
        m.accept_budget(&p).await.unwrap();
        assert_eq!(
            m.auto_trading_budget(&s)
                .await
                .unwrap()
                .krw
                .available_amount,
            1010
        );
        assert!(m
            .reserve_budget(&s, "OTHER", false, true, 11, 100)
            .await
            .is_err());
        m.apply_budget_fill(&p, 4, 400).await.unwrap();
        m.apply_budget_fill(&p, 4, 400).await.unwrap();
        assert_eq!(m.budget_owned(&s, "005930", false).await.unwrap(), 4);
        assert_eq!(
            m.auto_trading_budget(&s).await.unwrap().krw.cash_amount,
            1616
        );
        assert!(m.update_auto_trading_budget(&s, 500, 500).await.is_err());
        m.release_budget(&p).await.unwrap();
        m.pending.clear();
        assert_eq!(
            m.auto_trading_budget(&s)
                .await
                .unwrap()
                .krw
                .available_amount,
            1616
        );
        assert_eq!(
            m.auto_trading_budget(&scope("two"))
                .await
                .unwrap()
                .krw
                .allocated_amount,
            0
        );
        assert_eq!(
            m.auto_trading_budget(&s).await.unwrap().usd.cash_amount,
            500
        );
        assert!(m
            .reserve_budget(&s, "005930", false, false, 5, 100)
            .await
            .is_err());
        let sell = m
            .reserve_budget(&s, "005930", false, false, 4, 100)
            .await
            .unwrap();
        let p = pending(sell, s.clone(), false);
        m.track_pending_order("sell".into(), p.clone());
        m.accept_budget(&p).await.unwrap();
        m.apply_budget_fill(&p, 4, 440).await.unwrap();
        assert_eq!(m.budget_owned(&s, "005930", false).await.unwrap(), 0);
        assert_eq!(
            m.auto_trading_budget(&s).await.unwrap().krw.cash_amount,
            2051
        );
        tokio::fs::remove_dir_all(path).await.unwrap();
    }
    #[test]
    fn allocation_cannot_withdraw_committed_cash_or_overflow() {
        let mut c = CurrencyLedger::default();
        c.allocate(1000).unwrap();
        c.cash = 500;
        assert!(c.allocate(499).is_err());
        assert_eq!(c.allocated, 1000);
        assert!(c.allocate(MAX_AMOUNT + 1).is_err());
        assert_eq!(cost(1).unwrap(), 2);
    }

    #[tokio::test]
    async fn terminal_marker_recovers_crash_before_pending_removal() {
        let path = std::env::temp_dir().join(format!("budget-terminal-{}", uuid::Uuid::new_v4()));
        let mut m = manager(path.clone());
        let s = scope("one");
        m.update_auto_trading_budget(&s, 2020, 0).await.unwrap();
        let id = m
            .reserve_budget(&s, "005930", false, true, 10, 100)
            .await
            .unwrap();
        let p = pending(id, s.clone(), true);
        m.track_pending_order("test-order".into(), p.clone());
        m.accept_budget(&p).await.unwrap();
        m.apply_budget_fill(&p, 10, 1000).await.unwrap();
        m.release_budget(&p).await.unwrap();
        let mut restarted = manager(path.clone());
        restarted.track_pending_order("test-order".into(), p.clone());
        restarted.apply_budget_fill(&p, 10, 1000).await.unwrap();
        assert_eq!(
            restarted
                .auto_trading_budget(&s)
                .await
                .unwrap()
                .krw
                .cash_amount,
            1010
        );
        restarted.pending.clear();
        let view = restarted.auto_trading_budget(&s).await.unwrap();
        assert!(view.krw.blocked_reason.is_none());
        assert_eq!(view.krw.reserved_amount, 0);
        assert_eq!(view.krw.available_amount, 1010);
        tokio::fs::remove_dir_all(path).await.unwrap();
    }
}
