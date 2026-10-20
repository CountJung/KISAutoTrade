//! Save first, then publish a strategy configuration without replacing held state.
use super::{
    order::OrderManager,
    strategy::{
        build_strategy, LeveragedTrendHoldParams, PriceConditionParams, StrategyConfig,
        StrategyManager,
    },
};
use crate::{broker::BrokerScope, storage::StrategyStore};
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    sync::Arc,
};
use tokio::sync::Mutex;

/// Only the transaction task owns this Arc. Runtime gates retain Weak references,
/// so cancellation or an error releases the pause without asynchronous cleanup.
pub(crate) struct ConfigurationEditReservation {
    pub strategy_id: String,
    pub profile_id: String,
    pub scope: BrokerScope,
    pub symbols: HashSet<String>,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub(crate) struct ConfigurationUpdateError {
    pub code: &'static str,
    pub message: String,
}
impl ConfigurationUpdateError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Configs passed here have already been normalized by the production constructor.
/// Keep exact symbol identities: a differently cased key is not silently treated
/// as retained while a strategy's exact-key position map drops it.
pub(crate) fn effective_target_markets(config: &StrategyConfig) -> HashMap<String, bool> {
    let mut markets: HashMap<_, _> = config
        .target_symbols
        .iter()
        .map(|symbol| {
            (
                symbol.clone(),
                !crate::market_hours::is_domestic_symbol(symbol),
            )
        })
        .collect();
    let mut entry_markets = HashMap::new();
    if config.id.starts_with("price_condition") {
        let params: PriceConditionParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        for entry in params.symbols {
            entry_markets
                .entry(entry.symbol)
                .or_insert(entry.is_overseas);
        }
    } else if config.id.starts_with("leveraged_trend_hold") {
        let params: LeveragedTrendHoldParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        for entry in params.entries {
            // Live LTH selects the first leveraged-symbol entry. Legacy inverse
            // and base fields do not establish an independently traded target.
            entry_markets
                .entry(entry.leveraged_symbol)
                .or_insert(entry.is_overseas);
        }
    }
    for (symbol, is_overseas) in entry_markets {
        if let Some(market) = markets.get_mut(&symbol) {
            *market = is_overseas;
        }
    }
    markets
}

pub(crate) struct ConfigurationUpdateRequest {
    pub id: String,
    pub expected_profile_id: String,
    pub expected_scope: BrokerScope,
    pub enabled: Option<bool>,
    pub target_symbols: Option<Vec<String>>,
    pub order_quantity: Option<u64>,
    pub params: Option<serde_json::Value>,
}

#[derive(Clone)]
pub(crate) struct ConfigurationUpdateState {
    pub update_lock: Arc<Mutex<()>>,
    pub strategies: Arc<Mutex<StrategyManager>>,
    pub orders: Arc<Mutex<OrderManager>>,
    pub store: Arc<StrategyStore>,
}

pub(crate) async fn update_strategy_configuration(
    state: ConfigurationUpdateState,
    request: ConfigurationUpdateRequest,
) -> Result<StrategyConfig, ConfigurationUpdateError> {
    // Dropping an IPC/HTTP waiter must not interrupt the interval between a
    // durable save and publication. The owned task completes or restores disk.
    let store = state.store.clone();
    spawn_update(state, request, move |profile, configs| {
        let store = store.clone();
        async move { store.save(&profile, &configs).await }
    })
    .await
    .map_err(|error| {
        ConfigurationUpdateError::new(
            "STRATEGY_UPDATE_FAILED",
            format!("전략 설정 작업을 완료하지 못했습니다: {error}"),
        )
    })?
}

fn spawn_update<P, F>(
    state: ConfigurationUpdateState,
    request: ConfigurationUpdateRequest,
    persist: P,
) -> tokio::task::JoinHandle<Result<StrategyConfig, ConfigurationUpdateError>>
where
    P: FnMut(String, Vec<StrategyConfig>) -> F + Send + 'static,
    F: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    tokio::spawn(run_update(state, request, persist))
}

async fn run_update<P, F>(
    state: ConfigurationUpdateState,
    request: ConfigurationUpdateRequest,
    mut persist: P,
) -> Result<StrategyConfig, ConfigurationUpdateError>
where
    P: FnMut(String, Vec<StrategyConfig>) -> F,
    F: Future<Output = anyhow::Result<()>>,
{
    let _update = state.update_lock.lock().await;
    let (candidate, previous_all, next_all, reservation) = {
        let mut manager = state.strategies.lock().await;
        let previous_all: Vec<_> = manager.all_configs().into_iter().cloned().collect();
        let previous = previous_all
            .iter()
            .find(|config| config.id == request.id)
            .cloned()
            .ok_or_else(|| {
                ConfigurationUpdateError::new("STRATEGY_NOT_FOUND", "전략을 찾을 수 없습니다.")
            })?;
        if !previous.matches_scope(
            request.expected_scope.broker_id,
            request
                .expected_scope
                .account_id
                .as_ref()
                .map(|id| id.0.as_str()),
        ) {
            return Err(ConfigurationUpdateError::new(
                "SCOPE_MISMATCH",
                "활성 계좌가 변경되었습니다. 전략을 다시 불러오세요.",
            ));
        }
        // A raw signal has already modified an optimistic cache but is not yet a
        // pending order. Let its current-run feedback settle before taking a lease.
        if manager.has_unsettled_signals() {
            return Err(ConfigurationUpdateError::new(
                "CONFIGURATION_BUSY",
                "전략 신호를 처리 중입니다. 주문 상태가 확정된 뒤 다시 저장하세요.",
            ));
        }
        let mut candidate = previous.clone();
        if let Some(value) = request.enabled {
            candidate.enabled = value;
        }
        if let Some(value) = request.target_symbols {
            candidate.target_symbols = value;
        }
        if let Some(value) = request.order_quantity {
            candidate.order_quantity = value;
        }
        if let Some(value) = request.params {
            candidate.params = value;
        }
        super::strategy::validate_consecutive_entry_params(&candidate.id, &candidate.params)
            .map_err(|message| ConfigurationUpdateError::new("INVALID_STRATEGY_PARAMS", message))?;
        super::strategy::validate_momentum_entry_params(&candidate.id, &candidate.params)
            .map_err(|message| ConfigurationUpdateError::new("INVALID_STRATEGY_PARAMS", message))?;
        super::strategy::validate_deviation_entry_params(&candidate.id, &candidate.params)
            .map_err(|message| ConfigurationUpdateError::new("INVALID_STRATEGY_PARAMS", message))?;
        let candidate = build_strategy(candidate).config().clone();
        manager.validate_configuration_targets(&candidate)?;
        let symbols = previous
            .target_symbols
            .iter()
            .chain(&candidate.target_symbols)
            .cloned()
            .collect();
        let reservation = Arc::new(ConfigurationEditReservation {
            strategy_id: request.id,
            profile_id: request.expected_profile_id.clone(),
            scope: request.expected_scope,
            symbols,
        });
        let mut orders = state.orders.lock().await;
        orders
            .reserve_configuration_edit(&reservation, &previous, &candidate)
            .await?;
        manager.pause_configuration_edit(Arc::downgrade(&reservation));
        let next_all = previous_all
            .iter()
            .map(|config| {
                if config.id == candidate.id {
                    candidate.clone()
                } else {
                    config.clone()
                }
            })
            .collect();
        (candidate, previous_all, next_all, reservation)
    };
    let persistence = persist(request.expected_profile_id.clone(), next_all)
        .await
        .map_err(|error| {
            ConfigurationUpdateError::new(
                "PERSISTENCE_ERROR",
                format!("전략 설정을 저장하지 못했습니다: {error}"),
            )
        });

    let validation = if let Err(error) = persistence {
        // Atomic replacement may have succeeded before fsync/commit reports an
        // error. Restore even on the first save failure; its outcome is uncertain.
        Err(error)
    } else {
        let mut manager = state.strategies.lock().await;
        let mut orders = state.orders.lock().await;
        match orders.configuration_position_snapshots(&reservation).await {
            Ok(_) => {
                match manager.update_config(&candidate.id, |config| *config = candidate.clone()) {
                    Some(updated) => {
                        // Preserve private held protection exactly. Shared tracker data
                        // does not establish ownership or clear a strategy's holdings.
                        orders.publish_strategy_revision(manager.history_generation());
                        Ok(updated)
                    }
                    None => Err(ConfigurationUpdateError::new(
                        "STRATEGY_NOT_FOUND",
                        "저장 후 전략을 찾을 수 없습니다.",
                    )),
                }
            }
            Err(error) => Err(error),
        }
    };
    if let Err(error) = validation {
        if let Err(restore) = persist(request.expected_profile_id, previous_all).await {
            let reason =
                format!("전략 설정 저장을 완료하지 못했고 원본 복구도 실패했습니다: {restore}");
            state
                .orders
                .lock()
                .await
                .block_for_persistence_failure(reason.clone());
            return Err(ConfigurationUpdateError::new("PERSISTENCE_ERROR", reason));
        }
        return Err(error);
    }
    validation
}

#[cfg(test)]
mod tests;
