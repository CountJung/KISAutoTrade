//! Public US1m reservation boundary. Handles contain no profile or credential snapshot.
use super::*;
use lth_runtime::{Handle, Scope};

#[cfg(test)]
tokio::task_local! { static FORBID_PROVIDER_FACTORY: (); }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LthResearchReserveInput {
    pub preview: LeveragedTrendHoldPreviewInput,
    pub expected_broker_id: BrokerId,
    pub expected_paper_trading: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LthResearchHandle {
    pub job_id: String,
    pub owner: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LthResearchCancelView {
    pub status: &'static str,
}

fn rejected() -> CmdError {
    CmdError {
        code: "LTH_RESEARCH_REJECTED".into(),
        message: "US 분봉 연구 입력 또는 현재 프로파일·계좌·브로커·모드가 예약 기대값과 다릅니다."
            .into(),
    }
}
impl LthResearchHandle {
    fn decode(&self) -> CmdResult<Handle> {
        Handle::from_wire(&self.job_id, &self.owner)
    }
    fn encode(handle: Handle) -> Self {
        let (job_id, owner) = handle.to_wire();
        Self { job_id, owner }
    }
}

pub(crate) fn reserve_lth_research_for_profile(
    input: LthResearchReserveInput,
    profile: AccountProfile,
    runtime: &LthResearchRuntime,
) -> CmdResult<LthResearchHandle> {
    if input.expected_broker_id != profile.broker_id
        || input.expected_paper_trading != profile.is_paper_trading
        || input.preview.expected_profile_id.as_deref() != Some(profile.id.as_str())
        || input.preview.expected_broker_account_id.as_deref() != Some(profile.account_no.as_str())
    {
        return Err(rejected());
    }
    if input
        .preview
        .count
        .is_some_and(|count| !(20..=200).contains(&count))
    {
        return Err(rejected());
    }
    // Validate scenario syntax/value limits before reservation. Membership in the
    // eventual normalized bars remains the existing post-collection admission gate.
    let labels: Vec<String> = input
        .preview
        .assumptions
        .fx_scenario
        .iter()
        .map(|p| p.bar_time.clone())
        .chain(
            input
                .preview
                .assumptions
                .dividend_scenario
                .iter()
                .map(|p| p.bar_time.clone()),
        )
        .collect();
    input
        .preview
        .assumptions
        .validate_scenarios(true, &labels)
        .map_err(|_| rejected())?;
    let prepared = prepare_lth_preview(input.preview, profile).map_err(|_| rejected())?;
    if prepared.interval != "1m"
        || !prepared.entry_is_overseas
        || !prepared.params.entries.iter().any(|entry| {
            entry.is_overseas
                && entry
                    .leveraged_symbol
                    .eq_ignore_ascii_case(&prepared.symbol)
        })
    {
        return Err(rejected());
    }
    runtime.reserve(prepared).map(LthResearchHandle::encode)
}

pub(crate) async fn run_lth_research_for_profile(
    input: LthResearchHandle,
    profile: AccountProfile,
    runtime: &LthResearchRuntime,
) -> CmdResult<LeveragedTrendHoldPreviewView> {
    let handle = input.decode()?;
    let scope = Scope::from_profile(&profile);
    let result = runtime
        .run(&handle, &scope, |prepared, context| {
            #[cfg(test)]
            assert!(
                FORBID_PROVIDER_FACTORY.try_with(|_| ()).is_err(),
                "provider factory forbidden by offline guard"
            );
            async move { execute_lth_preview(prepared, context.as_of, Some(context)).await }
        })
        .await;
    result.map_err(|error| {
        if runtime.terminal_failure_receipt(&handle, &scope) {
            CmdError {
                code: "LTH_RESEARCH_FAILED".into(),
                message: "US 분봉 연구 수집·계산을 완료하지 못했고 작업은 종료됐습니다. 실제 공급·발행·coverage는 미검증입니다.".into(),
            }
        } else {
            error
        }
    })
}

pub(crate) async fn cancel_lth_research_owned(
    input: LthResearchHandle,
    runtime: &LthResearchRuntime,
) -> CmdResult<LthResearchCancelView> {
    runtime.cancel_owned(&input.decode()?).await?;
    Ok(LthResearchCancelView {
        status: "quiescent",
    })
}

async fn selected_profile(state: &AppState) -> CmdResult<AccountProfile> {
    state
        .profiles
        .read()
        .await
        .get_active()
        .cloned()
        .ok_or_else(rejected)
}

#[tauri::command]
pub async fn reserve_lth_research(
    input: LthResearchReserveInput,
    state: State<'_, AppState>,
) -> CmdResult<LthResearchHandle> {
    reserve_lth_research_for_profile(
        input,
        selected_profile(&state).await?,
        &state.lth_research_runtime,
    )
}
#[tauri::command]
pub async fn run_lth_research(
    input: LthResearchHandle,
    state: State<'_, AppState>,
) -> CmdResult<LeveragedTrendHoldPreviewView> {
    run_lth_research_for_profile(
        input,
        selected_profile(&state).await?,
        &state.lth_research_runtime,
    )
    .await
}
#[tauri::command]
pub async fn cancel_lth_research(
    input: LthResearchHandle,
    state: State<'_, AppState>,
) -> CmdResult<LthResearchCancelView> {
    cancel_lth_research_owned(input, &state.lth_research_runtime).await
}

#[cfg(test)]
mod tests;
