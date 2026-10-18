//! Exact ownership captured before provider submission, independent of broker totals.
use super::*;
use crate::trading::order::ownership::{OwnershipIdentity, OwnershipKey, OwnershipOwner};

pub(super) async fn prepare_ownership(
    manager: &mut OrderManager,
    submission: &mut OrderSubmission,
    deps: &SubmissionDeps,
    observed: u64,
    observed_avg: Option<u64>,
    budget_owned: u64,
) -> Result<(u64, Option<u64>)> {
    let identity = OwnershipIdentity::capture(
        &deps.broker_scope,
        deps.active_profile.as_ref(),
        &deps.rest_client,
    )?;
    let market = if submission.exchange.is_some()
        || !crate::market_hours::is_domestic_symbol(&submission.symbol)
    {
        BrokerMarket::Us
    } else {
        BrokerMarket::Kr
    };
    let ledger = manager.load_ownership_ledger().await?;
    // Create the generation durably before an order can refer to it. Losing this
    // primary document later must never silently reconstruct ownership from zero.
    manager.save_ownership_ledger(&ledger).await?;
    let total_owned = ledger
        .positions()
        .filter(|(key, _)| {
            key.identity.scope == identity.scope
                && key.identity.is_paper == identity.is_paper
                && key.market == market
                && key.symbol == submission.symbol
                && matches!(key.owner, OwnershipOwner::Strategy(_))
        })
        .try_fold(0u64, |sum, (_, position)| {
            sum.checked_add(position.quantity)
                .ok_or_else(|| anyhow::anyhow!("전략 소유수량 합계가 허용 범위를 초과합니다."))
        })?;
    if submission.is_manual {
        submission.submitted_cost_basis = observed_avg.filter(|p| *p > 0);
        return Ok(if matches!(submission.side, OrderSide::Sell) {
            (
                observed.saturating_sub(total_owned.max(budget_owned)),
                observed_avg,
            )
        } else {
            (observed.max(total_owned).max(budget_owned), observed_avg)
        });
    }
    let key = OwnershipKey {
        identity,
        owner: OwnershipOwner::Strategy(
            submission
                .strategy_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("주문 소유 전략이 없습니다."))?,
        ),
        market,
        symbol: submission.symbol.clone(),
    };
    let position = ledger.position(&key);
    submission.ownership_key = Some(key);
    submission.ownership_generation = Some(ledger.generation().to_string());
    if matches!(submission.side, OrderSide::Sell) {
        let avg = if position.quantity > 0 {
            Some(u64::try_from(
                position
                    .cost_notional
                    .div_ceil(u128::from(position.quantity)),
            )?)
        } else {
            None
        };
        Ok((observed.min(position.quantity), avg))
    } else {
        Ok((observed.max(total_owned).max(budget_owned), observed_avg))
    }
}

#[cfg(test)]
mod tests;
