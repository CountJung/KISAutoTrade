use std::collections::HashMap;

use serde::Serialize;

use crate::{
    broker::BrokerId, storage::stock_store::StockStore, trading::strategy::StrategyConfig,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyView {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub broker_id: BrokerId,
    pub broker_account_id: Option<String>,
    pub target_symbols: Vec<String>,
    pub target_symbol_names: HashMap<String, String>,
    pub order_quantity: u64,
    pub params: serde_json::Value,
}

pub async fn build_strategy_view(cfg: &StrategyConfig, stock_store: &StockStore) -> StrategyView {
    let mut symbol_names = HashMap::new();
    for code in &cfg.target_symbols {
        let name = stock_store
            .get_name(code)
            .await
            .unwrap_or_else(|| code.clone());
        symbol_names.insert(code.clone(), name);
    }

    StrategyView {
        id: cfg.id.clone(),
        name: cfg.name.clone(),
        enabled: cfg.enabled,
        broker_id: cfg.broker_id,
        broker_account_id: cfg.broker_account_id.clone(),
        target_symbols: cfg.target_symbols.clone(),
        target_symbol_names: symbol_names,
        order_quantity: cfg.order_quantity,
        params: cfg.params.clone(),
    }
}
