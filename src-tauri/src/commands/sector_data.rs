use super::{AppState, BrokerId, CmdError, CmdResult, KisRestClient, State};
use crate::trading::sector_data::*;
use std::sync::Arc;

fn error(code: &str) -> CmdError {
    CmdError {
        code: code.into(),
        message:
            "섹터 자료 준비가 보류되었습니다. 계정 범위·동의·manifest·자료 검증 코드를 확인하세요."
                .into(),
    }
}

struct QuoteSnapshot {
    profile_id: String,
    client: Arc<KisRestClient>,
    paper: bool,
}

async fn snapshot(state: &AppState, expected: Option<&str>) -> CmdResult<QuoteSnapshot> {
    let _guard = state.strategy_update_lock.lock().await;
    let profiles = state.profiles.read().await;
    let profile = profiles
        .get_active()
        .ok_or_else(|| error("SECTOR_QUOTE_NO_PROFILE"))?;
    if profile.broker_id != BrokerId::Kis || !profile.is_configured() {
        return Err(error("SECTOR_QUOTE_KIS_PROFILE_REQUIRED"));
    }
    if expected.is_some_and(|id| id != profile.id) {
        return Err(error("SECTOR_QUOTE_SCOPE_CHANGED"));
    }
    let client = state.rest_client.read().await.clone();
    if !client.matches_account(
        &profile.app_key,
        &profile.app_secret,
        &profile.account_no,
        profile.is_paper_trading,
    ) {
        return Err(error("SECTOR_QUOTE_SCOPE_CHANGED"));
    }
    Ok(QuoteSnapshot {
        profile_id: profile.id.clone(),
        client,
        paper: profile.is_paper_trading,
    })
}

fn same_scope<T>(
    expected_id: &str,
    expected_client: &Arc<T>,
    actual_id: &str,
    actual_client: &Arc<T>,
) -> bool {
    expected_id == actual_id && Arc::ptr_eq(expected_client, actual_client)
}

async fn check_scope(state: &AppState, expected: &QuoteSnapshot) -> Result<(), String> {
    let current = snapshot(state, Some(&expected.profile_id))
        .await
        .map_err(|_| "SECTOR_QUOTE_SCOPE_CHANGED".to_string())?;
    if !same_scope(
        &expected.profile_id,
        &expected.client,
        &current.profile_id,
        &current.client,
    ) {
        return Err("SECTOR_QUOTE_SCOPE_CHANGED".into());
    }
    Ok(())
}

struct NativeQuoteSource<'a> {
    state: &'a AppState,
    scope: QuoteSnapshot,
}

impl SectorQuoteSource for NativeQuoteSource<'_> {
    async fn fetch(
        &self,
        symbol: &str,
        exchange: &str,
        anchor: &str,
        adjusted: bool,
    ) -> Result<Vec<SectorQuoteRow>, String> {
        check_scope(self.state, &self.scope).await?;
        let result = self
            .scope
            .client
            .get_sector_daily_quotes(symbol, exchange, anchor, adjusted)
            .await;
        // Compare Arc identity as well as opaque profile ID after every provider await.
        // A→B→A publishes a new client and cannot revive an in-flight old snapshot.
        check_scope(self.state, &self.scope).await?;
        result.map_err(|_| "SECTOR_QUOTE_FETCH_FAILED".into())
    }
}

/// Explicitly invoked native-only getter; never returns account number/name/key fragments.
#[tauri::command]
pub async fn get_sector_quote_scope(state: State<'_, AppState>) -> CmdResult<SectorQuoteScope> {
    let scope = snapshot(&state, None).await?;
    Ok(SectorQuoteScope {
        profile_id: scope.profile_id,
        provider: "kis".into(),
        paper: scope.paper,
    })
}

/// Authenticated quotations only. This command is deliberately absent from REST/research runtime.
#[tauri::command]
pub async fn collect_sector_data(
    input: CollectSectorDataInput,
    state: State<'_, AppState>,
) -> CmdResult<SectorDataBundle> {
    if !input.authenticated_quotes_acknowledged
        || input.expected_profile_id.is_empty()
        || input.expected_profile_id.len() > 128
    {
        return Err(error("SECTOR_QUOTE_ACK_REQUIRED"));
    }
    if input.manifest.synthetic {
        return Err(error("SECTOR_QUOTE_SYNTHETIC_MANIFEST"));
    }
    let retrieved_at = chrono::Utc::now().to_rfc3339();
    validate_manifest(&input.manifest, &retrieved_at)
        .map_err(|_| error("INVALID_SECTOR_MANIFEST"))?;
    let scope = snapshot(&state, Some(&input.expected_profile_id)).await?;
    let source = NativeQuoteSource {
        state: &state,
        scope,
    };
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        collect_sector_bundle(&source, input.manifest, retrieved_at),
    )
    .await
    .unwrap_or_else(|_| Err("SECTOR_QUOTE_TIME_BUDGET_EXCEEDED".into()));
    check_scope(&state, &source.scope)
        .await
        .map_err(|_| error("SECTOR_QUOTE_SCOPE_CHANGED"))?;
    result.map_err(|code| error(&code))
}

/// Credential-free validation, shared by native IPC and protected REST.
#[tauri::command]
pub async fn validate_sector_data(
    input: ValidateSectorDataInput,
) -> CmdResult<SectorDataValidationReport> {
    if serde_json::to_vec(&input)
        .map_err(|_| error("INVALID_SECTOR_DATA_JSON"))?
        .len()
        > 2 * 1024 * 1024
    {
        return Err(error("SECTOR_DATA_INPUT_TOO_LARGE"));
    }
    tokio::task::spawn_blocking(move || validate_sector_bundle(&input))
        .await
        .map_err(|_| error("SECTOR_DATA_VALIDATION_FAILED"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_identity_rejects_switches_and_same_id_aba() {
        let first_a = Arc::new(());
        let b = Arc::new(());
        let second_a = Arc::new(());
        assert!(same_scope("A", &first_a, "A", &first_a.clone()));
        assert!(!same_scope("A", &first_a, "B", &b));
        assert!(!same_scope("A", &first_a, "A", &second_a));
        assert!(!same_scope("A", &first_a, "B", &first_a));
    }

    #[test]
    fn scope_view_contains_only_opaque_id_provider_and_paper() {
        let value = serde_json::to_value(SectorQuoteScope {
            profile_id: "opaque".into(),
            provider: "kis".into(),
            paper: true,
        })
        .unwrap();
        assert_eq!(
            value,
            serde_json::json!({"profileId":"opaque","provider":"kis","paper":true})
        );
    }
}
