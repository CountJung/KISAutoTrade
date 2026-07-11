/// 전략 설정 영구 저장소 — 프로파일별 JSON 파일
///
/// 저장 경로: `{data_dir}/strategies/{profile_id}/strategies.json`
use std::path::{Path, PathBuf};

use crate::{
    storage::{read_json_or_default, write_json},
    trading::strategy::StrategyConfig,
};

pub struct StrategyStore {
    data_dir: PathBuf,
}

impl StrategyStore {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }

    fn config_path(&self, profile_id: &str) -> PathBuf {
        self.data_dir
            .join("strategies")
            .join(profile_id)
            .join("strategies.json")
    }

    /// 전략 설정 로드. 활성 storage backend(JSON 또는 DB)를 따른다.
    pub async fn load(&self, profile_id: &str) -> anyhow::Result<Vec<StrategyConfig>> {
        let path = self.config_path(profile_id);
        read_json_or_default(&path).await
    }

    /// 전략 설정을 비동기적으로 저장
    pub async fn save(&self, profile_id: &str, configs: &[StrategyConfig]) -> anyhow::Result<()> {
        let path = self.config_path(profile_id);
        write_json(&path, &configs).await?;
        tracing::debug!(
            "전략 설정 저장 완료 — 프로파일: {}, 전략 수: {}",
            profile_id,
            configs.len()
        );
        Ok(())
    }
}
