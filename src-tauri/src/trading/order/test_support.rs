//! Synthetic credential과 localhost만 사용하는 주문 회귀 fixture.
use super::*;
use crate::{
    api::token::TokenManager,
    broker::{BrokerAccountId, BrokerId},
    config::{AccountProfile, AppConfig},
};

pub(super) fn profile(account: &str) -> AccountProfile {
    AccountProfile {
        id: format!("fixture-profile-{account}"),
        broker_id: BrokerId::Kis,
        name: "synthetic fixture".into(),
        is_paper_trading: true,
        live_trading_consent: false,
        app_key: format!("fixture-key-{account}"),
        app_secret: "fixture-secret".into(),
        account_no: account.into(),
    }
}
pub(super) fn scope(profile: &AccountProfile) -> BrokerScope {
    BrokerScope::new(
        profile.broker_id,
        Some(BrokerAccountId(profile.account_no.clone())),
    )
}
pub(super) async fn client(base_url: String, profile: &AccountProfile) -> Arc<KisRestClient> {
    assert!(base_url.starts_with("http://127.0.0.1:"));
    let config = Arc::new(AppConfig {
        broker_id: profile.broker_id,
        broker_account_id: profile.account_no.clone(),
        kis_app_key: profile.app_key.clone(),
        kis_app_secret: profile.app_secret.clone(),
        kis_account_no: profile.account_no.clone(),
        kis_is_paper_trading: profile.is_paper_trading,
        discord_bot_token: None,
        discord_channel_id: None,
        notification_levels: vec![],
    });
    let tokens = TokenManager::new(config);
    tokens
        .set_test_access_token("synthetic-localhost-token".into())
        .await;
    Arc::new(KisRestClient::new(
        base_url,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        profile.account_no.clone(),
        profile.is_paper_trading,
        Arc::new(RwLock::new(tokens)),
    ))
}
pub(super) struct Fixture {
    pub(super) manager: Arc<Mutex<OrderManager>>,
    pub(super) path: std::path::PathBuf,
    pub(super) profile: AccountProfile,
    pub(super) client: Arc<KisRestClient>,
}
impl Fixture {
    pub(super) async fn new(base_url: String) -> Self {
        let profile = profile("1234567801");
        let client = client(base_url, &profile).await;
        let path = std::env::temp_dir().join(format!("kis-execution-run-{}", uuid::Uuid::new_v4()));
        let manager = OrderManager::new(
            Arc::new(RwLock::new(Arc::clone(&client))),
            Arc::new(RwLock::new(ProfilesConfig {
                active_id: Some(profile.id.clone()),
                profiles: vec![profile.clone()],
            })),
            Arc::new(OrderStore::new(path.clone())),
            Arc::new(PendingOrderStore::new(path.clone())),
            Arc::new(TradeStore::new(path.clone())),
            Arc::new(Mutex::new(PositionTracker::new())),
            Arc::new(Mutex::new(OverseasPositionTracker::new())),
            Arc::new(StatsStore::new(path.clone())),
            Arc::new(RwLock::new(1400.0)),
            Arc::new(Mutex::new(RiskManager::default())),
            Arc::new(RiskStore::new(path.clone())),
            None,
        );
        Self {
            manager: Arc::new(Mutex::new(manager)),
            path,
            profile,
            client,
        }
    }
    pub(super) async fn begin(&self, revision: u64) -> ExecutionRunContext {
        self.manager
            .lock()
            .await
            .begin_execution_run(
                scope(&self.profile),
                Some(self.profile.id.clone()),
                revision,
                Arc::clone(&self.client),
                Some(self.profile.clone()),
            )
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
