//! Explicit native quotations; the preview function alone is credential free.
use super::{AccountProfile, AppState, BrokerId, CmdError, CmdResult, State};
use crate::{
    api::rest::KisRestClient,
    broker::toss::TossOpenApiClient,
    trading::{
        broker_chart::*,
        sector_data::SectorQuoteScope,
        sector_portfolio::{
            preview_broker_chart_price_approximation, BrokerChartBroker, BrokerChartInput,
            BrokerChartReport,
        },
    },
};
use std::sync::Arc;

fn error(code: &str) -> CmdError {
    CmdError {
        code: code.into(),
        message: "브로커 차트 근사 작업이 보류되었습니다. 자료·프로필·동의 상태를 확인하세요."
            .into(),
    }
}

struct Snapshot {
    profile: AccountProfile,
    witness: Arc<KisRestClient>,
    client: BrokerChartClient,
}

fn same_profile(a: &AccountProfile, b: &AccountProfile) -> bool {
    a.id == b.id
        && a.broker_id == b.broker_id
        && a.app_key == b.app_key
        && a.app_secret == b.app_secret
        && a.account_no == b.account_no
        && a.is_paper_trading == b.is_paper_trading
}

async fn snapshot(state: &AppState, expected: Option<&str>) -> CmdResult<Snapshot> {
    let _guard = state.strategy_update_lock.lock().await;
    let profile = state
        .profiles
        .read()
        .await
        .get_active()
        .cloned()
        .ok_or_else(|| error("BROKER_CHART_NO_PROFILE"))?;
    if !profile.is_configured() || expected.is_some_and(|id| id != profile.id) {
        return Err(error("BROKER_CHART_SCOPE_CHANGED_OR_UNCONFIGURED"));
    }
    let witness = state.rest_client.read().await.clone();
    let client = match profile.broker_id {
        BrokerId::Kis => {
            if !witness.matches_account(
                &profile.app_key,
                &profile.app_secret,
                &profile.account_no,
                profile.is_paper_trading,
            ) {
                return Err(error("BROKER_CHART_CLIENT_SCOPE_CHANGED"));
            }
            BrokerChartClient::Kis(witness.clone())
        }
        BrokerId::Toss => BrokerChartClient::Toss(Arc::new(TossOpenApiClient::new(
            "https://openapi.tossinvest.com",
            profile.app_key.clone(),
            profile.app_secret.clone(),
            None::<String>,
        ))),
    };
    Ok(Snapshot {
        profile,
        witness,
        client,
    })
}

async fn check_scope(state: &AppState, prior: &Snapshot) -> Result<(), String> {
    let now = snapshot(state, Some(&prior.profile.id))
        .await
        .map_err(|_| "BROKER_CHART_SCOPE_CHANGED")?;
    if !same_profile(&prior.profile, &now.profile) || !Arc::ptr_eq(&prior.witness, &now.witness) {
        return Err("BROKER_CHART_SCOPE_CHANGED".into());
    }
    Ok(())
}

struct NativeSource<'a> {
    state: &'a AppState,
    scope: Snapshot,
}
impl BrokerChartQuoteSource for NativeSource<'_> {
    async fn fetch(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        end: &str,
        cursor: Option<&str>,
    ) -> Result<RawBrokerChartPage, String> {
        check_scope(self.state, &self.scope).await?;
        let result = self.scope.client.fetch(symbol, exchange, end, cursor).await;
        check_scope(self.state, &self.scope).await?;
        result
    }
}

#[tauri::command]
pub async fn get_broker_chart_scope(state: State<'_, AppState>) -> CmdResult<SectorQuoteScope> {
    let snap = snapshot(&state, None).await?;
    Ok(SectorQuoteScope {
        profile_id: snap.profile.id,
        provider: match snap.profile.broker_id {
            BrokerId::Kis => "kis",
            BrokerId::Toss => "toss",
        }
        .into(),
        paper: snap.profile.is_paper_trading,
    })
}

#[tauri::command]
pub async fn collect_broker_chart_data(
    input: CollectBrokerChartInput,
    state: State<'_, AppState>,
) -> CmdResult<BrokerChartCollection> {
    if !input.authenticated_quotes_acknowledged
        || input.expected_profile_id.is_empty()
        || input.expected_profile_id.len() > 128
    {
        return Err(error("BROKER_CHART_QUOTE_ACK_REQUIRED"));
    }
    let now = chrono::Utc::now();
    let end = input.end_date.unwrap_or_else(|| {
        now.with_timezone(&chrono_tz::America::New_York)
            .date_naive()
            .pred_opt()
            .expect("current date predecessor")
            .format("%Y-%m-%d")
            .to_string()
    });
    let scope = snapshot(&state, Some(&input.expected_profile_id)).await?;
    let broker = match scope.profile.broker_id {
        BrokerId::Kis => BrokerChartBroker::Kis,
        BrokerId::Toss => BrokerChartBroker::Toss,
    };
    let source = NativeSource {
        state: &state,
        scope,
    };
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        collect_broker_chart(
            &source,
            broker,
            input.evaluation_bars,
            &end,
            &now.to_rfc3339(),
            &input.exchange_mappings,
        ),
    )
    .await
    .unwrap_or_else(|_| Err("BROKER_CHART_COLLECTION_TIMEOUT".into()));
    check_scope(&state, &source.scope)
        .await
        .map_err(|code| error(&code))?;
    result.map_err(|code| error(&code))
}

#[tauri::command]
pub async fn preview_broker_chart_portfolio(
    input: BrokerChartInput,
) -> CmdResult<BrokerChartReport> {
    if serde_json::to_vec(&input)
        .map_err(|_| error("BROKER_CHART_INVALID_INPUT"))?
        .len()
        > 2 * 1024 * 1024
    {
        return Err(error("BROKER_CHART_INPUT_TOO_LARGE"));
    }
    tokio::task::spawn_blocking(move || preview_broker_chart_price_approximation(&input))
        .await
        .map_err(|_| error("BROKER_CHART_CALCULATION_FAILED"))?
        .map_err(|code| error(&code))
}

#[cfg(test)]
mod tests {
    #[test]
    fn broker_chart_quote_source_has_no_rest_collection_or_research_auth_registration() {
        let research = include_str!("../research.rs");
        assert!(!research.contains("collect_broker_chart_data"));
        let server = include_str!("../server/mod.rs");
        assert!(!server.contains("collect_broker_chart_data"));
    }
}
