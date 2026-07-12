//! DB password의 OS keychain/credential vault 저장.
//!
//! - macOS Keychain, Windows Credential Manager, Linux secret-service를 keyring crate로 사용한다.
//! - keychain을 쓸 수 없는 환경에서는 호출자가 기존 파일 저장(0o600 `database_config.json`)으로
//!   fallback한다 — password를 잃는 것보다 낫다.
//! - password는 로그·오류 메시지에 포함하지 않는다.

use std::sync::OnceLock;

use anyhow::{Context, Result};
use keyring::Entry;

const KEYCHAIN_SERVICE: &str = "KISAutoTrade";
const KEYCHAIN_ACCOUNT: &str = "database-password";

/// Entry handle을 process-wide로 재사용한다.
/// (테스트의 mock store는 Entry 인스턴스별 독립 상태라 단일 handle 공유가 필수이고,
///  실제 OS keychain에서도 handle 재생성 비용을 줄인다.)
fn entry() -> Result<&'static Entry> {
    static ENTRY: OnceLock<Entry> = OnceLock::new();
    if let Some(entry) = ENTRY.get() {
        return Ok(entry);
    }
    let created = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .context("OS keychain entry를 열지 못했습니다")?;
    Ok(ENTRY.get_or_init(|| created))
}

/// password를 keychain에 저장한다.
pub(super) fn store_database_password(password: &str) -> Result<()> {
    entry()?
        .set_password(password)
        .context("OS keychain에 DB password를 저장하지 못했습니다")
}

/// keychain에서 password를 읽는다. 항목이 없으면 Ok(None).
pub(super) fn load_database_password() -> Result<Option<String>> {
    match entry()?.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error).context("OS keychain에서 DB password를 읽지 못했습니다"),
    }
}

/// keychain 항목을 삭제한다. 항목이 없어도 성공으로 처리한다.
pub(super) fn delete_database_password() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error).context("OS keychain의 DB password를 삭제하지 못했습니다"),
    }
}

#[cfg(test)]
pub(super) fn use_mock_keychain_for_tests() {
    // keyring mock store는 process-global이며 한 번만 설정하면 된다.
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    });
}

/// keychain 항목은 (service, account) 고정 단일 항목이라 병렬 테스트가 서로 덮어쓴다.
/// keychain을 사용하는 테스트는 이 lock을 잡고 직렬 실행한다.
#[cfg(test)]
pub(super) fn keychain_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
