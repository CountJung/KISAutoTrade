use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn initial_cash() -> f64 {
    10_000.0
}
fn cost_bps() -> f64 {
    10.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorPortfolioInput {
    #[serde(default = "initial_cash")]
    pub initial_cash_usd: f64,
    #[serde(default = "cost_bps")]
    pub cost_bps: f64,
    pub source: SectorDataSource,
    pub history: Vec<SectorDay>,
    pub evaluation: Vec<SectorDay>,
    pub expected_session_dates: Vec<String>,
    #[serde(default)]
    pub corporate_actions: Vec<SectorCorporateAction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorDataSource {
    pub name: String,
    pub retrieved_at: String,
    pub synthetic: bool,
    /// Caller declaration; the engine does not certify the provider's adjustment.
    pub adjustment_policy: String,
    pub calendar_source: String,
    pub coverage_verified: bool,
    /// Caller attests no split/dividend/receivable event in the evaluation window.
    /// History has no holdings; its adjustment policy is declared separately.
    pub corporate_actions_verified: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorDay {
    pub date: String,
    pub prices: BTreeMap<String, SectorPrice>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorPrice {
    pub adjusted_close: f64,
    pub open: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorCorporateAction {
    pub symbol: String,
    pub date: String,
    pub kind: String,
    pub split_ratio: Option<f64>,
    pub dividend_per_share: Option<f64>,
    pub pay_date: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SectorStrategy {
    Base,
    Defensive,
    Aggressive,
    Spy,
    Cash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DefensiveState {
    Normal,
    Defensive,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorHolding {
    pub symbol: String,
    pub quantity: u64,
    pub average_cost_usd: f64,
    pub remaining_buy_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorRank {
    pub symbol: String,
    pub r5: f64,
    pub r20: f64,
    pub rs5: f64,
    pub rs20: f64,
    pub percentile5: f64,
    pub percentile20: f64,
    pub score: f64,
    pub eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorTarget {
    pub symbol: String,
    pub weight: f64,
    pub budget_usd: f64,
    pub reference_weight: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorPlan {
    pub strategy: SectorStrategy,
    pub signal_date: String,
    pub defensive_state: DefensiveState,
    pub selected_symbols: Vec<String>,
    pub target_weight: f64,
    pub reference_equity_usd: f64,
    pub rebalance: bool,
    pub reasons: Vec<String>,
    pub targets: Vec<SectorTarget>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorRankingDay {
    pub date: String,
    pub signal_date: String,
    pub spy_r20: f64,
    pub previous_spy_r20: f64,
    pub rankings: Vec<SectorRank>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorTrade {
    pub date: String,
    pub symbol: String,
    pub side: String,
    pub requested_quantity: u64,
    pub quantity: u64,
    pub price_usd: f64,
    pub cost_usd: f64,
    pub cash_after_usd: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorDaily {
    pub date: String,
    pub plan: SectorPlan,
    pub cash_usd: f64,
    pub equity_usd: f64,
    pub daily_return_pct: f64,
    pub drawdown_pct: f64,
    pub exposure_pct: f64,
    pub holdings: Vec<SectorHolding>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorSummary {
    pub initial_cash_usd: f64,
    pub ending_equity_usd: f64,
    pub net_return_pct: f64,
    /// Same fills with costs added back arithmetically, not a no-cost rerun.
    pub same_path_return_before_costs_pct: f64,
    pub max_drawdown_pct: f64,
    pub max_daily_loss_pct: f64,
    pub daily_volatility_pct: f64,
    pub total_cost_usd: f64,
    pub realized_pnl_usd: f64,
    pub unrealized_pnl_usd: f64,
    pub average_exposure_pct: f64,
    pub turnover_pct: f64,
    pub filled_trade_count: usize,
    pub cash_limited_count: usize,
    pub ending_cash_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorPortfolioResult {
    pub strategy: SectorStrategy,
    pub summary: SectorSummary,
    pub daily: Vec<SectorDaily>,
    pub trades: Vec<SectorTrade>,
    pub ending_holdings: Vec<SectorHolding>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorPortfolioReport {
    pub engine_version: String,
    pub input_hash: String,
    pub source: SectorDataSource,
    pub history_bars: usize,
    pub evaluation_bars: usize,
    pub start_date: String,
    pub end_date: String,
    pub execution_model: String,
    pub warnings: Vec<String>,
    pub rankings: Vec<SectorRankingDay>,
    pub portfolios: Vec<SectorPortfolioResult>,
}
