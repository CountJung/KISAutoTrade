/// 국내 주식 종목코드 → 종목명 영구 캐시
///
/// - 파일 경로: `{data_dir}/stocklist/stocklist.json`
/// - 종목코드를 키, 종목명+갱신시각을 값으로 저장
/// - 잔고·현재가·주문 응답 등 KIS API에서 name 을 받을 때마다 자동 upsert
/// - `search()` 로 이름/코드 부분 일치 검색
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::api::rest::StockSearchItem;

const STOCKLIST_DIR: &str = "stocklist";
const STOCKLIST_FILE: &str = "stocklist.json";

// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockEntry {
    pub name: String,
    pub updated_at: String, // RFC3339
}

/// 종목 목록 통계 (Settings 화면 표시용)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockListStats {
    /// 저장된 종목 수
    pub count: usize,
    /// 마지막 upsert 시각 (RFC3339), 없으면 None
    pub last_updated_at: Option<String>,
    /// 종목 목록 파일 절대 경로 (디버그용)
    pub file_path: String,
    /// 자동 갱신 간격 (시간 단위, 0 = 수동 전용)
    pub update_interval_hours: u32,
}

// ────────────────────────────────────────────────────────────────────

pub struct StockStore {
    path: PathBuf,
    data: Arc<RwLock<HashMap<String, StockEntry>>>,
    /// 자동 갱신 간격 (시간, 0 = 수동)
    update_interval_hours: Arc<RwLock<u32>>,
}

impl StockStore {
    /// data_dir: AppState.data_dir 경로
    pub fn new(data_dir: &Path) -> Self {
        let path = data_dir.join(STOCKLIST_DIR).join(STOCKLIST_FILE);
        let data = load_from_file(&path);
        tracing::info!(
            "StockStore 초기화: {}개 종목 로드 ({:?})",
            data.len(), path
        );
        Self {
            path,
            data: Arc::new(RwLock::new(data)),
            update_interval_hours: Arc::new(RwLock::new(24)),
        }
    }

    // ── 조회 ──────────────────────────────────────────────────────

    /// 이름 또는 코드 부분 일치 검색
    pub async fn search(&self, query: &str, limit: usize) -> Vec<StockSearchItem> {
        if query.len() < 2 {
            return vec![];
        }
        let q = query.to_lowercase();
        let data = self.data.read().await;
        let mut results: Vec<StockSearchItem> = data
            .iter()
            .filter(|(code, entry)| {
                entry.name.to_lowercase().contains(&q) || code.contains(query)
            })
            .map(|(code, entry)| StockSearchItem {
                pdno: code.clone(),
                prdt_name: entry.name.clone(),
                market: None,
            })
            .collect();
        // 코드 완전 일치 우선 정렬
        results.sort_by(|a, b| {
            let a_exact = a.pdno == query;
            let b_exact = b.pdno == query;
            b_exact.cmp(&a_exact).then(a.pdno.cmp(&b.pdno))
        });
        results.truncate(limit);
        results
    }

    /// 종목명 조회 (코드 → 이름)
    pub async fn get_name(&self, code: &str) -> Option<String> {
        self.data.read().await.get(code).map(|e| e.name.clone())
    }

    /// 저장된 종목 수
    pub async fn size(&self) -> usize {
        self.data.read().await.len()
    }

    /// 마지막 갱신 시각 (항목이 하나라도 있으면 최신 항목의 시각 반환)
    pub async fn last_updated_at(&self) -> Option<String> {
        let data = self.data.read().await;
        data.values()
            .max_by(|a, b| a.updated_at.cmp(&b.updated_at))
            .map(|e| e.updated_at.clone())
    }

    pub async fn get_interval_hours(&self) -> u32 {
        *self.update_interval_hours.read().await
    }

    // ── 쓰기 ──────────────────────────────────────────────────────

    /// 종목 1건 upsert (기존보다 새 이름이 들어오면 덮어씀)
    pub async fn upsert(&self, code: &str, name: &str) {
        if code.is_empty() || name.is_empty() {
            return;
        }
        let now = chrono::Utc::now().to_rfc3339();
        let entry = StockEntry { name: name.to_string(), updated_at: now };
        let mut data = self.data.write().await;
        data.insert(code.to_string(), entry);
        self.persist(&data);
    }

    /// 여러 건 일괄 upsert (KIS 잔고·체결 등 배치 처리용)
    pub async fn upsert_many<I, S1, S2>(&self, items: I)
    where
        I: IntoIterator<Item = (S1, S2)>,
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let now = chrono::Utc::now().to_rfc3339();
        let mut data = self.data.write().await;
        let mut changed = false;
        for (code, name) in items {
            let c = code.as_ref();
            let n = name.as_ref();
            if c.is_empty() || n.is_empty() {
                continue;
            }
            data.insert(c.to_string(), StockEntry { name: n.to_string(), updated_at: now.clone() });
            changed = true;
        }
        if changed {
            self.persist(&data);
        }
    }

    /// 자동 갱신 간격 설정 (0 = 수동 전용)
    pub async fn set_interval_hours(&self, hours: u32) -> Result<()> {
        *self.update_interval_hours.write().await = hours;
        Ok(())
    }

    // ── 내부 유틸 ─────────────────────────────────────────────────

    fn persist(&self, data: &HashMap<String, StockEntry>) {
        let path = self.path.clone();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match serde_json::to_string_pretty(data) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    tracing::warn!("StockStore 저장 실패: {}", e);
                }
            }
            Err(e) => tracing::warn!("StockStore 직렬화 실패: {}", e),
        }
    }
}

fn load_from_file(path: &PathBuf) -> HashMap<String, StockEntry> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            tracing::warn!("stocklist.json 파싱 실패 (초기화): {}", e);
            HashMap::new()
        }),
        Err(_) => HashMap::new(),
    }
}
