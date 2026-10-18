//! Shared KIS query/parse/publication; notifications run inside the final gate.
use super::*;
use crate::api::rest::{BalanceResponse, OverseasBalanceResponse};

fn quantity(value: &str, symbol: &str) -> Result<u64> {
    value
        .trim()
        .replace(',', "")
        .parse()
        .map_err(|_| anyhow::anyhow!("{symbol} 보유수량 형식 오류"))
}
fn money(value: &str, multiplier: f64, symbol: &str) -> Result<u64> {
    let parsed: f64 = value
        .trim()
        .replace(',', "")
        .parse()
        .map_err(|_| anyhow::anyhow!("{symbol} 잔고 가격 형식 오류"))?;
    let scaled = parsed * multiplier;
    anyhow::ensure!(
        scaled.is_finite() && scaled >= 0.0 && scaled <= u64::MAX as f64,
        "{symbol} 잔고 가격 범위 오류"
    );
    Ok(scaled.round() as u64)
}

impl HoldingsUpdate {
    pub(crate) fn from_kis_domestic(response: &BalanceResponse) -> Result<Self> {
        let items = response
            .items
            .iter()
            .map(|item| {
                Ok((
                    item.pdno.clone(),
                    item.prdt_name.clone(),
                    quantity(&item.hldg_qty, &item.pdno)?,
                    money(&item.pchs_avg_pric, 1.0, &item.pdno)?,
                    money(&item.prpr, 1.0, &item.pdno)?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self::Domestic(items))
    }
    pub(crate) fn from_kis_overseas(response: &OverseasBalanceResponse) -> Result<Self> {
        let items = response
            .items
            .iter()
            .map(|item| {
                Ok((
                    item.ovrs_pdno.clone(),
                    item.ovrs_item_name.clone(),
                    normalize_overseas_order_exchange(&item.ovrs_excg_cd),
                    quantity(&item.ovrs_cblc_qty, &item.ovrs_pdno)?,
                    money(&item.pchs_avg_pric, 100.0, &item.ovrs_pdno)?,
                    money(&item.now_pric2, 100.0, &item.ovrs_pdno)?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self::Overseas(items))
    }
}

impl OrderManager {
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
