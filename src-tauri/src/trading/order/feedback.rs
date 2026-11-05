//! Strategy ownership publication always takes strategy -> order locks.
use super::ownership::{OwnershipIdentity, OwnershipKey, OwnershipOwner};
use super::*;
use crate::{broker::BrokerMarket, trading::strategy::StrategyManager};

impl OrderManager {
    pub(crate) async fn reconcile_strategy_state_shared(
        orders: &Arc<Mutex<Self>>,
        strategies: &Arc<Mutex<StrategyManager>>,
    ) -> Result<()> {
        let reconciliation = Self::confirm_pending_fills_from_broker_shared(orders).await;
        let configured = {
            let manager = orders.lock().await;
            let profile = manager.profiles.read().await.get_active().cloned();
            let client = manager.rest_client.read().await.clone();
            profile.as_ref().map_or_else(
                || !client.account_no().trim().is_empty() && !client.app_key().trim().is_empty(),
                |p| p.is_configured(),
            )
        };
        // The permanent daemon also runs before first account setup. That is not
        // a corrupt ownership document and must not latch a persistence block.
        if !configured {
            return reconciliation;
        }
        let feedback = Self::sync_strategy_ownership_shared(orders, strategies).await;
        let (dirty, refresh_scope, refresh_epoch) = {
            let manager = orders.lock().await;
            (
                manager.holdings_refresh_required != 0,
                manager.execution_scope.clone(),
                manager.execution_epoch,
            )
        };
        let holdings = if dirty {
            Self::refresh_active_holdings_shared(orders).await
        } else {
            Ok(())
        };
        if let Err(error) = &holdings {
            let mut manager = orders.lock().await;
            if manager.execution_scope == refresh_scope && manager.execution_epoch == refresh_epoch
            {
                manager.mark_holdings_sync(Some(error.to_string()));
            }
        }
        reconciliation.and(feedback).and(holdings)
    }

    async fn current_ownership_identity(&self) -> Result<OwnershipIdentity> {
        let profile = self.profiles.read().await.get_active().cloned();
        let client = self.rest_client.read().await.clone();
        OwnershipIdentity::capture(&self.execution_scope, profile.as_ref(), &client)
    }

    /// Actual fills may cross a stop/restart, but never transfer to another
    /// profile/account/mode. An optional run fence protects live tick checkpoints.
    pub(crate) async fn sync_strategy_ownership_shared(
        orders: &Arc<Mutex<Self>>,
        strategies: &Arc<Mutex<StrategyManager>>,
    ) -> Result<()> {
        let mut strategies = strategies.lock().await;
        let mut orders = orders.lock().await;
        orders.sync_strategy_ownership(&mut strategies).await
    }

    pub(crate) async fn sync_strategy_ownership(
        &mut self,
        strategies: &mut StrategyManager,
    ) -> Result<()> {
        // Profiles may be staged before the atomic scope/client publication.
        // Such a stale context is retryable, not evidence of corrupt storage.
        let identity = self.current_ownership_identity().await?;
        let result = self.apply_strategy_ownership(strategies, identity).await;
        if let Err(error) = &result {
            self.block_for_persistence_failure(format!(
                "전략 소유 상태를 확인하지 못했습니다: {error}"
            ));
        }
        result
    }

    async fn apply_strategy_ownership(
        &mut self,
        strategies: &mut StrategyManager,
        identity: OwnershipIdentity,
    ) -> Result<()> {
        let mut ledger = self.load_ownership_ledger().await?;
        let targets: Vec<_> = strategies
            .all_configs()
            .into_iter()
            .filter(|config| {
                config.matches_scope(
                    identity.scope.broker_id,
                    identity.scope.account_id.as_ref().map(|a| a.0.as_str()),
                )
            })
            .flat_map(|config| {
                crate::trading::configuration::effective_target_markets(config)
                    .into_iter()
                    .map(|(symbol, us)| (config.id.clone(), symbol, us))
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut metadata_changed = false;
        for (strategy_id, symbol, us) in targets {
            let market = if us {
                BrokerMarket::Us
            } else {
                BrokerMarket::Kr
            };
            let key = OwnershipKey {
                identity: identity.clone(),
                owner: OwnershipOwner::Strategy(strategy_id.clone()),
                market,
                symbol: symbol.clone(),
            };
            let position = ledger.position(&key);
            let avg = if position.quantity == 0 {
                0
            } else {
                u64::try_from(
                    position
                        .cost_notional
                        .div_ceil(u128::from(position.quantity)),
                )?
            };
            let had_metadata = strategies
                .owned_position_metadata_for_strategy(
                    &strategy_id,
                    &identity.scope,
                    market,
                    &symbol,
                )
                .is_some();
            strategies.sync_owned_position_for_strategy(
                &strategy_id,
                &identity.scope,
                market,
                &symbol,
                position.quantity,
                avg,
            );
            let pending = self
                .pending
                .values()
                .find(|pending| pending.ownership_key.as_ref() == Some(&key));
            if let Some(pending) = pending {
                anyhow::ensure!(
                    pending.ownership_generation.as_deref()
                        == Some(&ledger.generation().to_string()),
                    "미체결 주문의 소유 원장 세대가 일치하지 않습니다."
                );
            }
            strategies.note_order_pending(
                &strategy_id,
                &identity.scope,
                market,
                &symbol,
                pending.is_some(),
            );
            let metadata = ledger
                .position_metadata(&key)
                .cloned()
                .or_else(|| pending.and_then(|p| p.strategy_position_metadata.clone()));
            if !had_metadata || position.quantity == 0 {
                if let Some(metadata) = metadata {
                    strategies
                        .restore_owned_position_metadata_for_strategy(
                            &strategy_id,
                            &identity.scope,
                            market,
                            &symbol,
                            &metadata,
                        )
                        .map_err(anyhow::Error::msg)?;
                }
            }
            if position.quantity > 0 {
                if let Some(metadata) = strategies.owned_position_metadata_for_strategy(
                    &strategy_id,
                    &identity.scope,
                    market,
                    &symbol,
                ) {
                    if ledger.position_metadata(&key) != Some(&metadata) {
                        ledger.set_position_metadata(&key, metadata)?;
                        metadata_changed = true;
                    }
                }
            }
        }
        if metadata_changed {
            self.save_ownership_ledger(&ledger).await?;
        }
        Ok(())
    }

    /// Save only confirmed owners' updated protection after a live observation;
    /// do not clear a newly emitted (still unsubmitted) entry candidate.
    pub(crate) async fn checkpoint_strategy_positions_shared(
        orders: &Arc<Mutex<Self>>,
        strategies: &Arc<Mutex<StrategyManager>>,
        expected: &ExecutionRunContext,
    ) -> Result<bool> {
        let strategies = strategies.lock().await;
        let mut orders = orders.lock().await;
        if !orders.execution_run_matches(expected)
            || strategies.history_generation() != expected.configuration_revision
        {
            return Ok(false);
        }
        let identity = orders.current_ownership_identity().await?;
        let mut ledger = orders.load_ownership_ledger().await?;
        let mut changed = false;
        let keys: Vec<_> = ledger
            .positions()
            .filter(|(key, _)| key.identity == identity)
            .map(|(key, _)| key.clone())
            .collect();
        for key in keys {
            let OwnershipOwner::Strategy(strategy_id) = &key.owner else {
                continue;
            };
            if let Some(metadata) = strategies.owned_position_metadata_for_strategy(
                strategy_id,
                &identity.scope,
                key.market,
                &key.symbol,
            ) {
                if ledger.position_metadata(&key) != Some(&metadata) {
                    ledger.set_position_metadata(&key, metadata)?;
                    changed = true;
                }
            }
        }
        if changed {
            orders.save_ownership_ledger(&ledger).await?;
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
