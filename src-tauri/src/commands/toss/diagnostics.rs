//! Sequential Toss connection and account readiness diagnostics.
use super::*;

#[derive(Debug, Serialize)]
pub struct TossConnectionStep {
    pub id: String,
    pub label: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct TossConnectionDiagnostic {
    pub profile_id: String,
    pub profile_name: String,
    pub broker_id: BrokerId,
    pub account_seq: String,
    pub openapi_title: Option<String>,
    pub openapi_version: Option<String>,
    pub openapi_server: Option<String>,
    pub openapi_paths_count: Option<usize>,
    pub token_type: Option<String>,
    pub token_expires_at: Option<String>,
    pub accounts_count: Option<usize>,
    pub matched_account_no: Option<String>,
    pub holdings_count: Option<usize>,
    pub buying_power_krw: Option<String>,
    pub buying_power_usd: Option<String>,
    pub commissions_count: Option<usize>,
    pub sellable_quantity_symbol: Option<String>,
    pub sellable_quantity: Option<String>,
    pub is_ready: bool,
    pub steps: Vec<TossConnectionStep>,
    pub issues: Vec<String>,
}

fn toss_diag_step(
    id: impl Into<String>,
    label: impl Into<String>,
    ok: bool,
    message: impl Into<String>,
) -> TossConnectionStep {
    TossConnectionStep {
        id: id.into(),
        label: label.into(),
        ok,
        message: message.into(),
    }
}

pub(crate) async fn run_toss_connection_diagnostic(
    profile: AccountProfile,
) -> TossConnectionDiagnostic {
    let account_seq = profile.broker_account_id();
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(account_seq.clone()),
    );

    let mut steps = Vec::new();
    let mut issues = Vec::new();
    let mut openapi_title = None;
    let mut openapi_version = None;
    let mut openapi_server = None;
    let mut openapi_paths_count = None;
    let mut token_type = None;
    let mut token_expires_at = None;
    let mut accounts_count = None;
    let mut matched_account_no = None;
    let mut holdings_count = None;
    let mut first_holding_symbol = None;
    let mut buying_power_krw = None;
    let mut buying_power_usd = None;
    let mut commissions_count = None;
    let mut sellable_quantity_symbol = None;
    let mut sellable_quantity = None;

    match adapter.openapi_overview().await {
        Ok(overview) => {
            let ok = overview.server == TossBrokerAdapter::DEFAULT_BASE_URL
                && !overview.version.is_empty()
                && overview.paths_count > 0;
            if !ok {
                issues.push("토스증권 OpenAPI 스펙 메타데이터가 예상과 다릅니다.".into());
            }
            steps.push(toss_diag_step(
                "openapi",
                "OpenAPI 스펙",
                ok,
                format!(
                    "{} v{} · paths {}",
                    overview.title, overview.version, overview.paths_count
                ),
            ));
            openapi_title = Some(overview.title);
            openapi_version = Some(overview.version);
            openapi_server = Some(overview.server);
            openapi_paths_count = Some(overview.paths_count);
        }
        Err(e) => {
            let message = e.to_string();
            issues.push(message.clone());
            steps.push(toss_diag_step("openapi", "OpenAPI 스펙", false, message));
        }
    }

    let credentials_present =
        !profile.app_key.trim().is_empty() && !profile.app_secret.trim().is_empty();
    let account_seq_valid = account_seq.trim().parse::<i64>().is_ok();

    if !credentials_present {
        let message = "토스증권 client_id/client_secret이 설정되지 않았습니다.".to_string();
        issues.push(message.clone());
        steps.push(toss_diag_step("token", "토큰 발급", false, message));
    } else {
        match adapter.check_token().await {
            Ok(token) => {
                token_type = Some(token.token_type.clone());
                token_expires_at = Some(token.expires_at.to_rfc3339());
                steps.push(toss_diag_step(
                    "token",
                    "토큰 발급",
                    true,
                    format!("{} token · 만료 {}", token.token_type, token.expires_at),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("token", "토큰 발급", false, message));
            }
        }
    }

    if credentials_present {
        match adapter.list_accounts().await {
            Ok(accounts) => {
                accounts_count = Some(accounts.len());
                matched_account_no = accounts
                    .iter()
                    .find(|account| account.account_seq.to_string() == account_seq)
                    .map(|account| account.account_no.clone());
                let ok = account_seq.trim().is_empty()
                    || matched_account_no.is_some()
                    || !account_seq_valid;
                if !ok {
                    issues.push(format!(
                        "저장된 accountSeq({account_seq})와 일치하는 토스 계좌를 찾지 못했습니다."
                    ));
                }
                let message = match &matched_account_no {
                    Some(account_no) => {
                        format!("{}개 계좌 조회 · 저장 계좌 {}", accounts.len(), account_no)
                    }
                    None if account_seq.trim().is_empty() => {
                        format!("{}개 계좌 조회 · accountSeq를 저장하세요", accounts.len())
                    }
                    None if !account_seq_valid => {
                        format!(
                            "{}개 계좌 조회 · accountSeq는 숫자여야 합니다",
                            accounts.len()
                        )
                    }
                    None => format!("{}개 계좌 조회 · 저장 계좌 불일치", accounts.len()),
                };
                steps.push(toss_diag_step("accounts", "계좌 조회", ok, message));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("accounts", "계좌 조회", false, message));
            }
        }
    } else {
        steps.push(toss_diag_step(
            "accounts",
            "계좌 조회",
            false,
            "토큰 발급 전이라 계좌 조회를 건너뛰었습니다.",
        ));
    }

    if account_seq.trim().is_empty() {
        let message = "토스증권 accountSeq가 설정되지 않았습니다.".to_string();
        issues.push(message.clone());
        steps.push(toss_diag_step("holdings", "잔고 조회", false, message));
    } else if !account_seq_valid {
        let message = "토스증권 accountSeq는 숫자여야 합니다.".to_string();
        issues.push(message.clone());
        steps.push(toss_diag_step("holdings", "잔고 조회", false, message));
    } else if credentials_present {
        let account_id = BrokerAccountId(account_seq.clone());
        match adapter.list_holdings(Some(&account_id)).await {
            Ok(holdings) => {
                holdings_count = Some(holdings.len());
                first_holding_symbol = holdings.first().map(|holding| holding.symbol.0.clone());
                steps.push(toss_diag_step(
                    "holdings",
                    "잔고 조회",
                    true,
                    format!("{}개 보유 종목 조회", holdings.len()),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("holdings", "잔고 조회", false, message));
            }
        }
    } else {
        steps.push(toss_diag_step(
            "holdings",
            "잔고 조회",
            false,
            "토큰 발급 전이라 잔고 조회를 건너뛰었습니다.",
        ));
    }

    if credentials_present && account_seq_valid && !account_seq.trim().is_empty() {
        match adapter
            .get_buying_power(Some(&account_seq), BrokerCurrency::Krw)
            .await
        {
            Ok(power) => {
                buying_power_krw = Some(power.cash_buying_power.clone());
                steps.push(toss_diag_step(
                    "buyingPowerKrw",
                    "매수가능금액(KRW)",
                    true,
                    format!("{} {}", power.cash_buying_power, power.currency),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step(
                    "buyingPowerKrw",
                    "매수가능금액(KRW)",
                    false,
                    message,
                ));
            }
        }

        match adapter
            .get_buying_power(Some(&account_seq), BrokerCurrency::Usd)
            .await
        {
            Ok(power) => {
                buying_power_usd = Some(power.cash_buying_power.clone());
                steps.push(toss_diag_step(
                    "buyingPowerUsd",
                    "매수가능금액(USD)",
                    true,
                    format!("{} {}", power.cash_buying_power, power.currency),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step(
                    "buyingPowerUsd",
                    "매수가능금액(USD)",
                    false,
                    message,
                ));
            }
        }

        match adapter.list_commissions(Some(&account_seq)).await {
            Ok(commissions) => {
                commissions_count = Some(commissions.len());
                steps.push(toss_diag_step(
                    "commissions",
                    "수수료 조회",
                    true,
                    format!("{}개 수수료 정책 조회", commissions.len()),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("commissions", "수수료 조회", false, message));
            }
        }

        if let Some(symbol) = &first_holding_symbol {
            let broker_symbol = BrokerSymbol(symbol.clone());
            match adapter
                .get_sellable_quantity(Some(&account_seq), &broker_symbol)
                .await
            {
                Ok(quantity) => {
                    sellable_quantity_symbol = Some(symbol.clone());
                    sellable_quantity = Some(quantity.sellable_quantity.clone());
                    steps.push(toss_diag_step(
                        "sellableQuantity",
                        "매도가능수량",
                        true,
                        format!("{}: {}", symbol, quantity.sellable_quantity),
                    ));
                }
                Err(e) => {
                    let message = e.to_string();
                    issues.push(message.clone());
                    steps.push(toss_diag_step(
                        "sellableQuantity",
                        "매도가능수량",
                        false,
                        message,
                    ));
                }
            }
        } else {
            steps.push(toss_diag_step(
                "sellableQuantity",
                "매도가능수량",
                true,
                "보유 종목이 없어 매도가능수량 조회를 건너뛰었습니다.",
            ));
        }
    }

    let is_ready = issues.is_empty() && steps.iter().all(|step| step.ok);

    TossConnectionDiagnostic {
        profile_id: profile.id,
        profile_name: profile.name,
        broker_id: BrokerId::Toss,
        account_seq,
        openapi_title,
        openapi_version,
        openapi_server,
        openapi_paths_count,
        token_type,
        token_expires_at,
        accounts_count,
        matched_account_no,
        holdings_count,
        buying_power_krw,
        buying_power_usd,
        commissions_count,
        sellable_quantity_symbol,
        sellable_quantity,
        is_ready,
        steps,
        issues,
    }
}

#[tauri::command]
pub async fn check_toss_profile_connection(
    profile_id: String,
    state: State<'_, AppState>,
) -> CmdResult<TossConnectionDiagnostic> {
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
            message: "토스증권 프로파일만 Toss 연결 진단을 실행할 수 있습니다.".into(),
        });
    }

    Ok(run_toss_connection_diagnostic(profile).await)
}
