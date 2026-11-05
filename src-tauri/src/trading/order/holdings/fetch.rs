//! Shared KIS query/parse/publication; notifications run inside the final gate.
use super::*;
use crate::api::rest::{BalanceResponse, OverseasBalanceResponse};

impl OrderManager {
    /// Re-query the account authority after a fill; never add a fill delta to it.
    pub(crate) async fn refresh_active_holdings_shared(orders: &Arc<Mutex<Self>>) -> Result<()> {
        use crate::broker::{BrokerAdapter, TossBrokerAdapter};
        let (query, markets) = {
            let mut manager = orders.lock().await;
            let broker = manager.execution_scope.broker_id;
            let markets = if broker == BrokerId::Kis {
                match manager.holdings_refresh_required & 3 {
                    1 => HoldingsMarkets::Domestic,
                    2 => HoldingsMarkets::Overseas,
                    _ => HoldingsMarkets::Both,
                }
            } else {
                HoldingsMarkets::Both
            };
            (
                manager.begin_holdings_query(broker, markets).await?,
                markets,
            )
        };
        let update = if query.scope.broker_id == BrokerId::Kis {
            match markets {
                HoldingsMarkets::Domestic => {
                    HoldingsUpdate::from_kis_domestic(&query.client().get_balance().await?)?
                }
                HoldingsMarkets::Overseas => HoldingsUpdate::from_kis_overseas(
                    &query.client().get_overseas_balance().await?,
                )?,
                _ => {
                    let domestic = query.client().get_balance().await?;
                    let overseas = query.client().get_overseas_balance().await?;
                    let HoldingsUpdate::Domestic(domestic) =
                        HoldingsUpdate::from_kis_domestic(&domestic)?
                    else {
                        unreachable!()
                    };
                    let HoldingsUpdate::Overseas(overseas) =
                        HoldingsUpdate::from_kis_overseas(&overseas)?
                    else {
                        unreachable!()
                    };
                    HoldingsUpdate::Both { domestic, overseas }
                }
            }
        } else {
            let profile = query
                .profile()
                .ok_or_else(|| anyhow::anyhow!("활성 Toss 프로파일이 없습니다."))?;
            let adapter = TossBrokerAdapter::with_credentials(
                TossBrokerAdapter::DEFAULT_BASE_URL,
                profile.app_key.clone(),
                profile.app_secret.clone(),
                Some(profile.broker_account_id()),
            );
            let holdings = adapter
                .list_holdings(query.scope.account_id.as_ref())
                .await?;
            HoldingsUpdate::from_toss(&query.scope, &holdings)?
        };
        let mut manager = orders.lock().await;
        manager.publish_holdings(query, update).await?;
        manager.mark_holdings_sync(None);
        Ok(())
    }

    pub(crate) async fn refresh_kis_domestic_shared<P>(
        orders: &Arc<Mutex<Self>>,
        stock_store: Option<&crate::storage::StockStore>,
        on_published: P,
    ) -> Result<BalanceResponse>
    where
        P: FnOnce(&BalanceResponse) + Send,
    {
        let query = orders
            .lock()
            .await
            .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Domestic)
            .await?;
        let response = query.client().get_balance().await?;
        let update = HoldingsUpdate::from_kis_domestic(&response)?;
        if let Some(store) = stock_store {
            store
                .upsert_many(
                    response
                        .items
                        .iter()
                        .map(|item| (item.pdno.clone(), item.prdt_name.clone())),
                )
                .await;
        }
        let mut manager = orders.lock().await;
        manager.publish_holdings(query, update).await?;
        on_published(&response);
        drop(manager);
        Ok(response)
    }

    pub(crate) async fn refresh_kis_overseas_shared<P>(
        orders: &Arc<Mutex<Self>>,
        on_published: P,
    ) -> Result<OverseasBalanceResponse>
    where
        P: FnOnce(&OverseasBalanceResponse) + Send,
    {
        let query = orders
            .lock()
            .await
            .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Overseas)
            .await?;
        let response = query.client().get_overseas_balance().await?;
        let update = HoldingsUpdate::from_kis_overseas(&response)?;
        let mut manager = orders.lock().await;
        manager.publish_holdings(query, update).await?;
        on_published(&response);
        drop(manager);
        Ok(response)
    }
}
