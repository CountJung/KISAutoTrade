/// 전략 설정 영구 저장소 — 프로파일별 JSON 파일
///
/// 저장 경로: `{data_dir}/strategies/{profile_id}/strategies.json`
use std::path::{Path, PathBuf};

use crate::trading::strategy::StrategyConfig;

pub struct StrategyStore {
    data_dir: PathBuf,
}

impl StrategyStore {
    pub fn new(data_dir: &Path) -> Self {
        Self { data_dir: data_dir.to_path_buf() }
    }

    fn config_path(&self, profile_id: &str) -> PathBuf {
        self.data_dir
            .join("strategies")
            .join(profile_id)
            .join("strategies.json")
    }

    /// 앱 초기화 시 동기적으로 전략 설정 로드 (없으면 빈 벡터 반환)
    pub fn load_sync(&self, profile_id: &str) -> Vec<StrategyConfig> {
        let path = self.config_path(profile_id);
        if !path.exists() {
            return vec![];
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                serde_json::from_str::<Vec<StrategyConfig>>(&content)
                    .unwrap_or_default()
            }
            Err(e) => {
                tracing::warn!(
                    "전략 설정 로드 실패 — {:?}: {} (기본값 사용)",
                    path, e
                );
                vec![]
            }
        }
    }

    /// 전략 설정을 비동기적으로 저장
    pub async fn save(&self, profile_id: &str, configs: &[StrategyConfig]) -> anyhow::Result<()> {
        let path = self.config_path(profile_id);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let content = serde_json::to_string_pretty(configs)?;
        tokio::fs::write(&path, content).await?;
        tracing::debug!(
            "전략 설정 저장 완료 — 프로파일: {}, 전략 수: {}",
            profile_id,
            configs.len()
        );
        Ok(())
    }
}
