use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

/// USD/KRW 현재 환율 조회
///
/// `open.er-api.com` 무료 API (API 키 불필요, 1일 1회 업데이트)를 사용합니다.
/// 네트워크 오류 발생 시 Err를 반환하며, 호출부에서 캐시 값을 유지합니다.
pub async fn fetch_usd_krw_rate() -> Result<f64> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("KISAutoTrade/1.0")
        .build()?;

    #[derive(Deserialize)]
    struct RateResp {
        result: String,
        rates: HashMap<String, f64>,
    }

    let resp: RateResp = client
        .get("https://open.er-api.com/v6/latest/USD")
        .send()
        .await?
        .json()
        .await?;

    if resp.result != "success" {
        anyhow::bail!("환율 API 오류: result = {}", resp.result);
    }

    resp.rates
        .get("KRW")
        .copied()
        .ok_or_else(|| anyhow::anyhow!("환율 응답에 KRW 없음"))
}
