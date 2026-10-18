//! 자동매매 실행 세대와 시작 시점 provider 의존성을 함께 고정한다.

use super::*;
use crate::{broker::BrokerId, config::AccountProfile};

/// 비민감 실행 식별자. 예약 전에 같은 OrderManager mutex에서 검증한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRunContext {
    pub epoch: u64,
    pub configuration_revision: u64,
    pub broker_scope: BrokerScope,
    pub profile_id: Option<String>,
}

// AccountProfile에는 credential이 있으므로 이 컨테이너는 Debug를 구현하지 않는다.
pub(super) struct ExecutionRun {
    pub(super) context: ExecutionRunContext,
    pub(super) rest_client: Arc<KisRestClient>,
    pub(super) profile: Option<AccountProfile>,
}

impl OrderManager {
    /// 시작 검증에 사용한 client/profile snapshot을 해당 실행의 모든 예약에 고정한다.
    pub fn begin_execution_run(
        &mut self,
        scope: BrokerScope,
        profile_id: Option<String>,
        configuration_revision: u64,
        rest_snapshot: Arc<KisRestClient>,
        profile_snapshot: Option<AccountProfile>,
    ) -> Result<ExecutionRunContext> {
        self.end_execution_run();
        validate_snapshot(
            &scope,
            profile_id.as_deref(),
            &rest_snapshot,
            profile_snapshot.as_ref(),
        )?;
        let epoch = self.execution_epoch.checked_add(1).ok_or_else(|| {
            anyhow::anyhow!("자동매매 실행 세대 한도에 도달했습니다. 앱을 재시작하세요.")
        })?;
        self.execution_epoch = epoch;
        self.execution_scope = scope.clone();
        let context = ExecutionRunContext {
            epoch,
            configuration_revision,
            broker_scope: scope,
            profile_id,
        };
        self.active_execution_run = Some(ExecutionRun {
            context: context.clone(),
            rest_client: rest_snapshot,
            profile: profile_snapshot,
        });
        Ok(context)
    }

    /// Scope/client publication holds this OrderManager gate while resetting the
    /// single-active-account trackers. Pending orders and budget remain intact.
    pub(crate) async fn clear_position_trackers_for_scope_change(&mut self) {
        self.position_tracker.lock().await.clear();
        self.overseas_position_tracker.lock().await.clear();
    }

    /// 이미 예약한 provider 요청과 pending·예산은 계속 추적한다.
    pub fn end_execution_run(&mut self) {
        self.execution_epoch = self.execution_epoch.saturating_add(1);
        self.active_execution_run = None;
    }

    /// 전략 manager publication과 같은 critical section에서 호출한다.
    pub fn publish_strategy_revision(&mut self, revision: u64) {
        if self
            .active_execution_run
            .as_ref()
            .is_some_and(|run| revision < run.context.configuration_revision)
        {
            tracing::warn!("전략 revision 역행으로 자동매매 실행을 무효화합니다.");
            self.end_execution_run();
            return;
        }
        if let Some(run) = &mut self.active_execution_run {
            run.context.configuration_revision = revision;
        }
    }

    pub fn execution_run_context(&self) -> Option<ExecutionRunContext> {
        self.active_execution_run
            .as_ref()
            .map(|run| run.context.clone())
    }

    /// 조회에도 같은 실행에서 검증한 client/profile을 사용한다. 민감 snapshot은 직렬화하지 않는다.
    pub(crate) fn execution_run_snapshots(
        &self,
        expected: &ExecutionRunContext,
    ) -> Option<(Arc<KisRestClient>, Option<AccountProfile>)> {
        self.active_execution_run
            .as_ref()
            .filter(|run| &run.context == expected)
            .map(|run| (Arc::clone(&run.rest_client), run.profile.clone()))
    }

    pub fn execution_run_matches(&self, expected: &ExecutionRunContext) -> bool {
        self.active_execution_run
            .as_ref()
            .is_some_and(|run| &run.context == expected)
    }
}

pub(super) fn validate_snapshot(
    scope: &BrokerScope,
    profile_id: Option<&str>,
    client: &KisRestClient,
    profile: Option<&AccountProfile>,
) -> Result<()> {
    if let Some(profile) = profile {
        anyhow::ensure!(
            Some(profile.id.as_str()) == profile_id,
            "자동매매 프로파일 식별자가 변경되었습니다."
        );
        anyhow::ensure!(
            profile.broker_id == scope.broker_id,
            "자동매매 broker가 변경되었습니다."
        );
        anyhow::ensure!(
            scope
                .account_id
                .as_ref()
                .is_some_and(|account| account.0 == profile.broker_account_id()),
            "자동매매 계좌가 변경되었습니다."
        );
        if scope.broker_id == BrokerId::Kis {
            anyhow::ensure!(
                client.matches_account(
                    &profile.app_key,
                    &profile.app_secret,
                    &profile.account_no,
                    profile.is_paper_trading
                ),
                "자동매매 KIS client의 계좌·인증·모드가 시작 프로파일과 다릅니다."
            );
        }
    } else {
        anyhow::ensure!(
            profile_id.is_none() && scope.broker_id == BrokerId::Kis,
            "자동매매 프로파일을 확인할 수 없습니다."
        );
        if let Some(account) = &scope.account_id {
            anyhow::ensure!(
                client.matches_account(
                    client.app_key(),
                    client.app_secret(),
                    &account.0,
                    client.is_paper()
                ),
                "자동매매 KIS client의 계좌가 시작 scope와 다릅니다."
            );
        }
    }
    Ok(())
}
