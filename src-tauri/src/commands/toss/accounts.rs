//! Toss account lookup and masked account options.
use super::*;

#[derive(Debug, Serialize)]
pub struct TossAccountOptionView {
    pub account_seq: String,
    pub account_no_masked: String,
    pub account_type: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct TossAccountLookupInput {
    pub client_id: String,
    pub client_secret: String,
}

fn mask_toss_account_no(account_no: &str) -> String {
    let suffix: String = account_no
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if suffix.is_empty() {
        "(계좌번호 없음)".to_string()
    } else {
        format!("****{suffix}")
    }
}

fn toss_account_option(account: crate::broker::toss::TossAccount) -> TossAccountOptionView {
    let account_seq = account.account_seq.to_string();
    let account_no_masked = mask_toss_account_no(&account.account_no);
    let label = format!(
        "accountSeq {} · {} · {}",
        account_seq, account_no_masked, account.account_type
    );
    TossAccountOptionView {
        account_seq,
        account_no_masked,
        account_type: account.account_type,
        label,
    }
}

pub(crate) async fn lookup_toss_accounts_with_credentials(
    client_id: String,
    client_secret: String,
) -> CmdResult<Vec<TossAccountOptionView>> {
    let client_id = client_id.trim();
    let client_secret = client_secret.trim();
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(CmdError {
            code: "MISSING_CREDENTIALS".into(),
            message: "토스증권 Client ID와 Client Secret을 모두 입력하세요.".into(),
        });
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        client_id.to_string(),
        client_secret.to_string(),
        None::<String>,
    );
    adapter
        .list_accounts()
        .await
        .map(|accounts| accounts.into_iter().map(toss_account_option).collect())
        .map_err(|e| CmdError {
            code: "TOSS_ACCOUNTS_ERROR".into(),
            message: e.to_string(),
        })
}

#[tauri::command]
pub async fn list_toss_accounts(
    input: TossAccountLookupInput,
) -> CmdResult<Vec<TossAccountOptionView>> {
    lookup_toss_accounts_with_credentials(input.client_id, input.client_secret).await
}

#[tauri::command]
pub async fn list_toss_profile_accounts(
    profile_id: String,
    state: State<'_, AppState>,
) -> CmdResult<Vec<TossAccountOptionView>> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .cloned()
            .ok_or_else(|| CmdError {
                code: "NOT_FOUND".into(),
                message: format!("프로파일을 찾을 수 없습니다: {profile_id}"),
            })?
    };

    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_MISMATCH".into(),
            message: "토스증권 프로파일만 accountSeq 목록을 조회할 수 있습니다.".into(),
        });
    }

    lookup_toss_accounts_with_credentials(profile.app_key, profile.app_secret).await
}
