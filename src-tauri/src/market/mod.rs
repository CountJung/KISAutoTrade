/// KRX 전체 종목 목록 관리
///
/// 한국거래소 데이터시스템(data.krx.co.kr)에서 KOSPI + KOSDAQ 종목 목록을 다운로드하고
/// 로컬 JSON 파일로 캐시합니다. 24시간마다 자동 갱신.
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::api::rest::StockSearchItem;

const CACHE_FILE: &str = "stock_list.json";
const CACHE_TTL_SECS: u64 = 86_400; // 24시간

/// KRX JSON API 응답 아이템
#[derive(Deserialize)]
struct KrxItem {
    #[serde(rename = "ISU_SRT_CD")]
    code: String,
    #[serde(rename = "ISU_ABBRV")]
    name: String,
}

#[derive(Deserialize)]
struct KrxResponse {
    #[serde(rename = "OutBlock_1")]
    items: Vec<KrxItem>,
}

pub struct StockList;

impl StockList {
    /// KRX 데이터시스템에서 KOSPI + KOSDAQ 전체 목록 다운로드
    /// POST https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd
    pub async fn fetch_from_krx() -> Result<Vec<StockSearchItem>> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let mut all: Vec<StockSearchItem> = Vec::with_capacity(3500);

        for mkt_id in &["STK", "KSQ"] {
            let resp = client
                .post("https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd")
                .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
                .header("Referer", "https://data.krx.co.kr/")
                .body(format!(
                    "bld=dbms%2FMDC%2FSTAT%2Fstandard%2FMDCSTAT01901&mktId={}&share=1&money=1&csvxls_isNo=false",
                    mkt_id
                ))
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    match r.json::<KrxResponse>().await {
                        Ok(data) => {
                            for item in data.items {
                                if !item.code.is_empty() && !item.name.is_empty() {
                                    all.push(StockSearchItem {
                                        pdno: item.code,
                                        prdt_name: item.name,
                                    });
                                }
                            }
                        }
                        Err(e) => tracing::warn!("KRX {} JSON 파싱 실패: {}", mkt_id, e),
                    }
                }
                Ok(r) => tracing::warn!("KRX {} HTTP {}", mkt_id, r.status()),
                Err(e) => tracing::warn!("KRX {} 요청 실패: {}", mkt_id, e),
            }
        }

        tracing::info!("KRX 종목 목록 다운로드: {}개", all.len());
        Ok(all)
    }

    /// 캐시 로드 또는 KRX 다운로드 (캐시가 24시간 미만이면 재사용)
    pub async fn load_or_fetch(data_dir: &Path) -> Vec<StockSearchItem> {
        let cache_path = data_dir.join(CACHE_FILE);

        // 유효한 캐시가 있으면 즉시 반환
        if let Some(items) = try_load_cache(&cache_path) {
            return items;
        }

        match Self::fetch_from_krx().await {
            Ok(items) if !items.is_empty() => {
                save_cache(&cache_path, &items);
                items
            }
            Ok(_) => {
                tracing::warn!("KRX 종목 목록이 비어있습니다");
                try_load_cache_any(&cache_path).unwrap_or_default()
            }
            Err(e) => {
                tracing::warn!("KRX 종목 목록 다운로드 실패: {} — 캐시 사용 시도", e);
                try_load_cache_any(&cache_path).unwrap_or_default()
            }
        }
    }
}

fn try_load_cache(path: &std::path::PathBuf) -> Option<Vec<StockSearchItem>> {
    let meta = std::fs::metadata(path).ok()?;
    let age = std::time::SystemTime::now()
        .duration_since(meta.modified().ok()?)
        .ok()?;
    if age.as_secs() > CACHE_TTL_SECS {
        return None;
    }
    let data = std::fs::read_to_string(path).ok()?;
    let items: Vec<StockSearchItem> = serde_json::from_str(&data).ok()?;
    tracing::info!("종목 목록 캐시 로드: {}개", items.len());
    Some(items)
}

fn try_load_cache_any(path: &std::path::PathBuf) -> Option<Vec<StockSearchItem>> {
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_cache(path: &std::path::PathBuf, items: &[StockSearchItem]) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string(items) {
        let _ = std::fs::write(path, json);
    }
}

/// 이름 또는 코드로 로컬 검색
pub fn search_local(items: &[StockSearchItem], query: &str, limit: usize) -> Vec<StockSearchItem> {
    let q = query.to_lowercase();
    items
        .iter()
        .filter(|i| {
            i.prdt_name.to_lowercase().contains(&q) || i.pdno.contains(query)
        })
        .take(limit)
        .cloned()
        .collect()
}
