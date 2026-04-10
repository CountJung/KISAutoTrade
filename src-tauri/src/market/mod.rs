/// KRX 전체 종목 목록 관리
///
/// 한국거래소 데이터시스템(data.krx.co.kr)에서 KOSPI + KOSDAQ 종목 목록을 다운로드하고
/// 로컬 JSON 파일로 캐시합니다. 24시간마다 자동 갱신.
/// KRX 접근 불가 시 NAVER Finance 자동완성 API로 실시간 검색 폴백.
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;

use crate::api::rest::StockSearchItem;

const CACHE_FILE: &str = "stock_list.json";
const CACHE_TTL_SECS: u64 = 86_400; // 24시간

/// KRX JSON API 응답 아이템
#[derive(Deserialize)]
struct KrxItem {
    #[serde(rename = "ISU_SRT_CD")]
    code: String,
    /// 정식 종목명 — ISU_ABBRV 대비 "미국" 등 ETF 키워드 검색 커버리지 향상
    #[serde(rename = "ISU_NM")]
    full_name: String,
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
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            // KRX는 JSESSIONID 쿠키 세션이 없으면 "LOGOUT" 응답을 반환함
            // cookie_store를 활성화하면 초기화 요청의 쿠키를 이후 AJAX 요청에서 재사용
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // ① 세션 초기화: KRX 루트 방문 → JSESSIONID 쿠키 획득
        //    WAF/봇 차단 방어를 위해 루트(/) 먼저 방문 후 특정 페이지 방문
        tracing::debug!("KRX 세션 초기화 중 (루트 페이지 방문)...");
        let _ = client
            .get("https://data.krx.co.kr/")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "ko-KR,ko;q=0.9,en-US;q=0.8,en;q=0.7")
            .send()
            .await;
        // 특정 페이지도 방문해서 Referer 체인 구성
        let init_result = client
            .get("https://data.krx.co.kr/contents/MDC/STAT/standard/MDCSTAT01901.cmd")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Referer", "https://data.krx.co.kr/")
            .send()
            .await;
        match &init_result {
            Ok(r) => tracing::debug!("KRX 세션 초기화: HTTP {}", r.status()),
            Err(e) => tracing::warn!("KRX 세션 초기화 실패 (계속 시도): {}", e),
        }

        let mut all: Vec<StockSearchItem> = Vec::with_capacity(3500);

        for mkt_id in &["STK", "KSQ"] {
            let resp = client
                .post("https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd")
                .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
                .header("Referer", "https://data.krx.co.kr/contents/MDC/STAT/standard/MDCSTAT01901.cmd")
                // KRX AJAX 엔드포인트는 XMLHttpRequest 헤더 필요
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Accept", "application/json, text/javascript, */*; q=0.01")
                .body(format!(
                    "bld=dbms%2FMDC%2FSTAT%2Fstandard%2FMDCSTAT01901&mktId={}&share=1&money=1&csvxls_isNo=false",
                    mkt_id
                ))
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    // 파싱 전 원본 텍스트 확인 (LOGOUT/에러 응답 감지)
                    match r.text().await {
                        Ok(text) if text.starts_with("LOGOUT") || text.contains("\"error\"") => {
                            tracing::warn!("KRX {} 세션 오류 응답: {:.80}", mkt_id, text);
                        }
                        Ok(text) => {
                            match serde_json::from_str::<KrxResponse>(&text) {
                                Ok(data) => {
                                    let prev = all.len();
                                    for item in data.items {
                                        if !item.code.is_empty() && !item.full_name.is_empty() {
                                            all.push(StockSearchItem {
                                                pdno: item.code,
                                                prdt_name: item.full_name,
                                                market: None,
                                            });
                                        }
                                    }
                                    tracing::info!("KRX {}: {}개 종목 추가", mkt_id, all.len() - prev);
                                }
                                Err(e) => tracing::warn!(
                                    "KRX {} JSON 파싱 실패: {} (응답 앞 200자: {:.200})",
                                    mkt_id, e, text
                                ),
                            }
                        }
                        Err(e) => tracing::warn!("KRX {} 응답 텍스트 읽기 실패: {}", mkt_id, e),
                    }
                }
                Ok(r) => tracing::warn!("KRX {} HTTP {}", mkt_id, r.status()),
                Err(e) => tracing::warn!("KRX {} 요청 실패: {}", mkt_id, e),
            }
        }

        tracing::info!("KRX 종목 목록 다운로드 완료: {}개", all.len());
        Ok(all)
    }

    /// 캐시 로드 또는 KRX 다운로드 (캐시가 24시간 미만이면 재사용)
    pub async fn load_or_fetch(data_dir: &Path) -> Vec<StockSearchItem> {
        let cache_path = data_dir.join(CACHE_FILE);
        tracing::info!("종목 목록 로드 시작 — 캐시 경로: {:?}", cache_path);

        // 유효한 캐시가 있으면 즉시 반환
        if let Some(items) = try_load_cache(&cache_path) {
            return items;
        }

        tracing::info!("유효한 캐시 없음 — KRX 서버에서 다운로드 시도");
        match Self::fetch_from_krx().await {
            Ok(items) if !items.is_empty() => {
                tracing::info!("KRX 다운로드 성공: {}개 종목 → 캐시 저장", items.len());
                save_cache(&cache_path, &items);
                items
            }
            Ok(_) => {
                tracing::warn!("KRX 종목 목록이 비어있습니다 — 만료된 캐시로 폴백");
                try_load_cache_any(&cache_path).unwrap_or_else(|| {
                    tracing::error!("종목 목록 로드 완전 실패: KRX 응답 비어있고 캐시도 없음. \
                        앱에서 '종목 목록 새로고침' 버튼을 눌러주세요.");
                    vec![]
                })
            }
            Err(e) => {
                tracing::warn!("KRX 다운로드 실패: {} — 만료된 캐시로 폴백", e);
                try_load_cache_any(&cache_path).unwrap_or_else(|| {
                    tracing::error!("종목 목록 로드 완전 실패: KRX 실패({})이고 캐시도 없음. \
                        네트워크 연결 및 방화벽을 확인하거나 앱에서 '종목 목록 새로고침'을 눌러주세요.", e);
                    vec![]
                })
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
    if let Ok(json) = serde_json::to_string_pretty(items) {
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

// ── NAVER Finance 실시간 검색 (KRX 캐시 없을 때 폴백) ─────────────

#[derive(Deserialize)]
struct NaverAcItem {
    code: String,
    name: String,
}

#[derive(Deserialize)]
struct NaverAcResponse {
    #[serde(default)]
    items: Vec<NaverAcItem>,
}

// ── Yahoo Finance 종목코드 → 이름 조회 ────────────────────────────

#[derive(Deserialize)]
struct YahooSearchQuote {
    symbol: String,
    #[serde(default)]
    longname: String,
    #[serde(default)]
    shortname: String,
}

// 직접 파싱을 위해 래퍼 없이 처리
#[derive(Deserialize)]
struct YahooSearchRaw {
    #[serde(default)]
    quotes: Vec<YahooSearchQuote>,
}

/// Yahoo Finance로 6자리 KRX 종목코드 → 한글 종목명 조회
/// ex) "005930" → "삼성전자(주)"
/// URL: https://query1.finance.yahoo.com/v1/finance/search?q=005930.KS&lang=ko&region=KR
pub async fn lookup_name_by_code(code: &str) -> Result<String> {
    let symbol = format!("{}.KS", code);
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let resp = client
        .get("https://query1.finance.yahoo.com/v1/finance/search")
        .query(&[
            ("q", symbol.as_str()),
            ("lang", "ko"),
            ("region", "KR"),
            ("quotesCount", "1"),
            ("newsCount", "0"),
            ("listsCount", "0"),
        ])
        .header("Accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("Yahoo Finance HTTP 오류: {}", resp.status()));
    }

    let text = resp.text().await?;
    tracing::debug!("Yahoo Finance 응답: {:.200}", text);

    let data: YahooSearchRaw = serde_json::from_str(&text)
        .map_err(|e| anyhow!("Yahoo Finance 응답 파싱 실패: {} (본문: {:.200})", e, text))?;

    let q = data.quotes.into_iter()
        .find(|q| q.symbol == symbol)
        .ok_or_else(|| anyhow!("Yahoo Finance: {} 에 대한 결과 없음", symbol))?;

    // longname이 있으면 우선 사용, 없으면 shortname
    let name = if !q.longname.is_empty() { q.longname } else { q.shortname };
    if name.is_empty() {
        return Err(anyhow!("Yahoo Finance: {} 이름 필드 비어있음", symbol));
    }

    Ok(name)
}

/// NAVER Finance 자동완성 API를 이용한 실시간 종목 검색
/// URL: https://ac.stock.naver.com/ac?query={q}&target=stock,etf&source=domestic
/// - KRX 캐시가 비어있을 때 폴백으로 사용
/// - 브라우저 없이도 접근 가능한 공개 API
pub async fn search_naver_live(query: &str) -> Result<Vec<StockSearchItem>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let resp = client
        .get("https://ac.stock.naver.com/ac")
        .query(&[("query", query), ("target", "stock,etf"), ("source", "domestic")])
        .header("Accept", "application/json, text/plain, */*")
        .header("Referer", "https://finance.naver.com/")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("NAVER 검색 HTTP 오류: {}", resp.status()));
    }

    let text = resp.text().await?;
    tracing::debug!("NAVER autocomplete 응답: {:.200}", text);

    let data: NaverAcResponse = serde_json::from_str(&text)
        .map_err(|e| anyhow!("NAVER 응답 파싱 실패: {} (본문: {:.200})", e, text))?;

    if data.items.is_empty() {
        tracing::debug!("NAVER 검색 결과 없음: query={:?}", query);
    }

    Ok(data
        .items
        .into_iter()
        .filter(|i| !i.code.is_empty())
        .map(|i| StockSearchItem {
            pdno: i.code,
            prdt_name: i.name,
            market: None,
        })
        .collect())
}

// ── KRX 프록시 검색 (k-skill-proxy) ─────────────────────────────────
//
// k-skill-proxy는 KRX Open API를 API 키 없이 래핑하는 공개 프록시다.
// 공식 KRX 데이터(KOSPI/KOSDAQ/KONEX 시장 구분 포함)를 반환하며,
// data.krx.co.kr WAF 차단 문제를 우회한다.
// 출처: https://github.com/CountJung/k-skill/blob/main/korean-stock-search/SKILL.md
const KRX_PROXY_BASE: &str = "https://k-skill-proxy.nomadamas.org";

/// KRX 프록시 검색 결과 아이템 (k-skill-proxy 응답 형태)
#[derive(Deserialize)]
struct KrxProxyItem {
    market: String,
    code: String,
    name: String,
}

#[derive(Deserialize)]
struct KrxProxySearchResponse {
    #[serde(default)]
    items: Vec<KrxProxyItem>,
}

/// k-skill-proxy를 통한 KRX 공식 종목 이름 검색
///
/// - 이름 또는 종목코드 검색 가능
/// - API 키 불필요 (프록시 서버가 KRX_API_KEY 관리)
/// - 시장 구분(KOSPI/KOSDAQ/KONEX) 포함 반환
/// - KRX data.krx.co.kr WAF 차단 우회 가능
pub async fn search_krx_proxy(query: &str, limit: usize) -> Result<Vec<StockSearchItem>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; KISAutoTrade)")
        .timeout(std::time::Duration::from_secs(8))
        .build()?;

    let base = std::env::var("KSKILL_PROXY_BASE_URL")
        .unwrap_or_else(|_| KRX_PROXY_BASE.to_string());

    let resp = client
        .get(format!("{}/v1/korean-stock/search", base))
        .query(&[("q", query), ("limit", &limit.to_string())])
        .header("Accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("KRX 프록시 HTTP 오류: {}", resp.status()));
    }

    let text = resp.text().await?;
    tracing::debug!("KRX 프록시 검색 응답: {:.300}", text);

    let data: KrxProxySearchResponse = serde_json::from_str(&text)
        .map_err(|e| anyhow!("KRX 프록시 응답 파싱 실패: {} (본문: {:.200})", e, text))?;

    if data.items.is_empty() {
        tracing::debug!("KRX 프록시 검색 결과 없음: query={:?}", query);
    }

    Ok(data.items
        .into_iter()
        .filter(|i| !i.code.is_empty())
        .map(|i| StockSearchItem {
            pdno: i.code,
            prdt_name: i.name,
            market: Some(i.market),
        })
        .collect())
}
