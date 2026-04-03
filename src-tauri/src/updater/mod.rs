/// 앱 업데이트 확인 모듈
///
/// GitHub Releases API를 통해 최신 버전을 확인합니다.
/// 실제 다운로드·설치는 사용자가 브라우저에서 수행합니다.
///
/// # 설정
/// 아래 두 상수를 본 프로젝트의 GitHub 저장소 정보로 변경하세요.
/// ```
/// const GITHUB_OWNER: &str = "your-github-username";
/// const GITHUB_REPO:  &str = "KISAutoTrade";
/// ```
use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────
// TODO: 실제 GitHub 저장소 정보로 변경
// ────────────────────────────────────────────────────────────────────
const GITHUB_OWNER: &str = "CountJung";
const GITHUB_REPO: &str = "KISAutoTrade";

/// GitHub Releases API 응답 (필요한 필드만 발췌)
#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    /// pre-release 여부 (true면 업데이트 대상에서 제외)
    prerelease: bool,
    draft: bool,
}

/// 프런트엔드에 반환할 업데이트 정보
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// 새 버전이 존재하면 true
    pub has_update: bool,
    /// 현재 앱 버전 (Cargo.toml 기준)
    pub current_version: String,
    /// GitHub 최신 릴리스 버전 (v 제거)
    pub latest_version: String,
    /// GitHub 릴리스 페이지 URL
    pub release_url: String,
    /// 릴리스 노트 (Markdown)
    pub release_notes: Option<String>,
}

/// GitHub Releases API로 최신 버전을 조회합니다.
pub async fn check(client: &reqwest::Client) -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    let resp = client
        .get(&url)
        .header(
            "User-Agent",
            format!("AutoConditionTrade/{}", current_version),
        )
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("GitHub API 요청 실패: {e}"))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        // 저장소가 없거나 릴리스가 없는 경우 → 업데이트 없음으로 처리
        tracing::debug!("GitHub 릴리스 없음 (404) — 업데이트 확인 건너뜀");
        return Ok(UpdateInfo {
            has_update: false,
            current_version: current_version.to_string(),
            latest_version: current_version.to_string(),
            release_url: String::new(),
            release_notes: None,
        });
    }

    if !resp.status().is_success() {
        return Err(format!("GitHub API 오류: {}", resp.status()));
    }

    let release: GitHubRelease = resp
        .json()
        .await
        .map_err(|e| format!("GitHub API 응답 파싱 실패: {e}"))?;

    // draft / pre-release는 건너뜀
    if release.draft || release.prerelease {
        return Ok(UpdateInfo {
            has_update: false,
            current_version: current_version.to_string(),
            latest_version: current_version.to_string(),
            release_url: release.html_url,
            release_notes: None,
        });
    }

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let has_update = is_newer(&latest_version, current_version);

    tracing::info!(
        "업데이트 확인 완료: 현재={} 최신={} 업데이트={}",
        current_version,
        latest_version,
        has_update
    );

    Ok(UpdateInfo {
        has_update,
        current_version: current_version.to_string(),
        latest_version,
        release_url: release.html_url,
        release_notes: release.body,
    })
}

/// `latest` 가 `current` 보다 새 버전이면 true
///
/// 간단한 semver 비교: major.minor.patch 각 숫자를 튜플로 비교합니다.
fn is_newer(latest: &str, current: &str) -> bool {
    fn parse(v: &str) -> (u64, u64, u64) {
        let mut parts = v.splitn(4, '.').map(|s| s.parse::<u64>().unwrap_or(0));
        (
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
        )
    }
    parse(latest) > parse(current)
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
    }
}
