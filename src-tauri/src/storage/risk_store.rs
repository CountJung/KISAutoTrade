use anyhow::Result;
use std::path::PathBuf;

use super::{read_json_or_default, write_json};
use crate::trading::risk::{RiskConfigState, RiskRuntimeState};

/// 리스크 설정과 일별 runtime 상태 저장소.
/// 설정(`risk/config.json`)과 runtime(`risk/runtime.json`)을 분리 저장해
/// 앱 재시작으로 손실 한도·주문 횟수·연속 손실 차단·비상정지가 우회되지 않게 한다.
pub struct RiskStore {
    config_path: PathBuf,
    runtime_path: PathBuf,
}

impl RiskStore {
    pub fn new(data_dir: PathBuf) -> Self {
        let base = data_dir.join("risk");
        Self {
            config_path: base.join("config.json"),
            runtime_path: base.join("runtime.json"),
        }
    }

    pub async fn load_config(&self) -> Result<Option<RiskConfigState>> {
        read_json_or_default(&self.config_path).await
    }

    pub async fn save_config(&self, state: &RiskConfigState) -> Result<()> {
        write_json(&self.config_path, &Some(state)).await
    }

    pub async fn load_runtime(&self) -> Result<Option<RiskRuntimeState>> {
        read_json_or_default(&self.runtime_path).await
    }

    pub async fn save_runtime(&self, state: &RiskRuntimeState) -> Result<()> {
        write_json(&self.runtime_path, &Some(state)).await
    }
}
