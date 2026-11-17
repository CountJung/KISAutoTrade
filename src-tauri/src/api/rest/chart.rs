//! KIS 국내·해외 기간별 차트 요청과 응답 매핑.

use super::{read_kis_response_text, ChartCandle, KisRestClient, KIS_RATE_GROUP_QUOTE};
use anyhow::Result;
use reqwest::{header::HeaderMap, Client, RequestBuilder, Response};
use serde::Deserialize;

/// 인증·pacing 정책과 분리된 차트 wire 계약. 테스트도 이 요청 빌더를 사용한다.
struct ChartRequest<'a> {
    path: &'static str,
    tr_id: &'static str,
    query: [(&'static str, &'a str); 6],
}

impl<'a> ChartRequest<'a> {
    fn domestic(symbol: &'a str, period: &'a str, from: &'a str, to: &'a str) -> Self {
        Self {
            path: "/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice",
            tr_id: "FHKST03010100",
            query: [
                ("FID_COND_MRKT_DIV_CODE", "J"),
                ("FID_INPUT_ISCD", symbol),
                ("FID_INPUT_DATE_1", from),
                ("FID_INPUT_DATE_2", to),
                ("FID_PERIOD_DIV_CODE", period),
                ("FID_ORG_ADJ_PRC", "0"),
            ],
        }
    }

    fn overseas(symbol: &'a str, exchange: &'a str, period: &str, base_date: &'a str) -> Self {
        let gubn = match period {
            "W" => "1",
            "M" => "2",
            _ => "0",
        };
        Self {
            path: "/uapi/overseas-price/v1/quotations/dailyprice",
            // Official examples_llm/overseas_stock/dailyprice and examples_user agree.
            tr_id: "HHDFS76240000",
            query: [
                ("AUTH", ""),
                ("EXCD", exchange),
                ("SYMB", symbol),
                ("GUBN", gubn),
                ("BYMD", base_date),
                ("MODP", "1"),
            ],
        }
    }

    fn overseas_daily_mode(
        symbol: &'a str,
        exchange: &'a str,
        base_date: &'a str,
        adjusted: bool,
    ) -> Self {
        let mut request = Self::overseas(symbol, exchange, "D", base_date);
        request.query[5].1 = if adjusted { "1" } else { "0" };
        request
    }

    fn build(&self, client: &Client, base_url: &str, mut headers: HeaderMap) -> RequestBuilder {
        headers.insert(
            "tr_id",
            reqwest::header::HeaderValue::from_static(self.tr_id),
        );
        client
            .get(format!("{base_url}{}", self.path))
            .headers(headers)
            .query(&self.query)
    }
}

impl KisRestClient {
    /// Research-only unadjusted daily OHLCV. Keeps provider strings and row order;
    /// date/price validation belongs to the collector. Never exposes response bodies.
    pub async fn get_broker_chart_daily_quotes(
        &self,
        symbol: &str,
        exchange: &str,
        base_date: &str,
    ) -> Result<Vec<ChartCandle>> {
        let recipe = ChartRequest::overseas_daily_mode(symbol, exchange, base_date, false);
        let headers = self
            .auth_headers(recipe.tr_id)
            .await
            .map_err(|_| anyhow::anyhow!("BROKER_CHART_AUTH_FAILED"))?;
        let response = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_QUOTE,
                recipe.build(&self.client, &self.base_url, headers),
            )
            .await
            .map_err(|_| anyhow::anyhow!("BROKER_CHART_TRANSPORT_FAILED"))?;
        parse_broker_chart_daily_response(response).await
    }

    /// Quote-only sector preparation. Preserves the shared HTTP timeout/rate limiter,
    /// but never echoes provider response bodies or auth/transport errors.
    pub async fn get_sector_daily_quotes(
        &self,
        symbol: &str,
        exchange: &str,
        base_date: &str,
        adjusted: bool,
    ) -> Result<Vec<crate::trading::sector_data::SectorQuoteRow>> {
        let recipe = ChartRequest::overseas_daily_mode(symbol, exchange, base_date, adjusted);
        let headers = self
            .auth_headers(recipe.tr_id)
            .await
            .map_err(|_| anyhow::anyhow!("SECTOR_QUOTE_AUTH_FAILED"))?;
        let response = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_QUOTE,
                recipe.build(&self.client, &self.base_url, headers),
            )
            .await
            .map_err(|_| anyhow::anyhow!("SECTOR_QUOTE_TRANSPORT_FAILED"))?;
        let success = response.status().is_success();
        let body = read_kis_response_text(response)
            .await
            .map_err(|_| anyhow::anyhow!("SECTOR_QUOTE_RESPONSE_LIMIT_OR_READ_FAILED"))?;
        parse_sector_daily_body(success, &body)
    }
    /// 국내 기간별 차트 (일/주/월, 실전·모의 공통).
    pub async fn get_chart_data(
        &self,
        symbol: &str,
        period_code: &str, // "D"=일, "W"=주, "M"=월
        start_date: &str,  // YYYYMMDD
        end_date: &str,    // YYYYMMDD
    ) -> Result<Vec<ChartCandle>> {
        let recipe = ChartRequest::domestic(symbol, period_code, start_date, end_date);
        let headers = self.auth_headers(recipe.tr_id).await?;
        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_QUOTE,
                recipe.build(&self.client, &self.base_url, headers),
            )
            .await?;
        let candles = parse_domestic_response(resp).await?;
        tracing::debug!(
            "차트 데이터 조회 완료: {} ({}) {} 봉",
            symbol,
            period_code,
            candles.len()
        );
        Ok(candles)
    }

    /// 해외 기간별 차트. BYMD 공백은 당일 기준, MODP=1은 수정주가 반영.
    pub async fn get_overseas_chart_data(
        &self,
        symbol: &str,
        exchange: &str,
        period_code: &str, // "D"=일, "W"=주, "M"=월
        base_date: &str,   // YYYYMMDD — 비워두면 오늘 기준
    ) -> Result<Vec<ChartCandle>> {
        let recipe = ChartRequest::overseas(symbol, exchange, period_code, base_date);
        let headers = self.auth_headers(recipe.tr_id).await?;
        let resp = self
            .send_with_rate_limit(
                KIS_RATE_GROUP_QUOTE,
                recipe.build(&self.client, &self.base_url, headers),
            )
            .await?;
        let candles = parse_overseas_response(resp).await?;
        tracing::debug!(
            "해외 차트 조회 완료: {} {} ({}) {} 봉",
            exchange,
            symbol,
            period_code,
            candles.len()
        );
        Ok(candles)
    }
}

async fn parse_broker_chart_daily_response(response: Response) -> Result<Vec<ChartCandle>> {
    let success = response.status().is_success();
    let body = read_kis_response_text(response)
        .await
        .map_err(|_| anyhow::anyhow!("BROKER_CHART_RESPONSE_LIMIT_OR_READ_FAILED"))?;
    parse_broker_chart_daily_body(success, &body)
}

fn parse_broker_chart_daily_body(success: bool, body: &str) -> Result<Vec<ChartCandle>> {
    if !success {
        anyhow::bail!("BROKER_CHART_HTTP_FAILED");
    }
    #[derive(Deserialize)]
    struct Raw {
        rt_cd: String,
        msg_cd: Option<String>,
        output2: Option<Vec<Row>>,
    }
    #[derive(Deserialize)]
    struct Row {
        xymd: String,
        open: String,
        high: String,
        low: String,
        clos: String,
        tvol: String,
    }
    let raw: Raw =
        serde_json::from_str(body).map_err(|_| anyhow::anyhow!("BROKER_CHART_INVALID_RESPONSE"))?;
    if raw.rt_cd != "0" {
        if matches!(raw.msg_cd.as_deref(), Some("EGW00133" | "EGW00201")) {
            anyhow::bail!("BROKER_CHART_RATE_LIMITED");
        }
        anyhow::bail!("BROKER_CHART_PROVIDER_REJECTED");
    }
    let rows = raw
        .output2
        .ok_or_else(|| anyhow::anyhow!("BROKER_CHART_MISSING_ROWS"))?;
    if rows.is_empty() {
        anyhow::bail!("BROKER_CHART_EMPTY_ROWS");
    }
    if rows.len() > 500 {
        anyhow::bail!("BROKER_CHART_TOO_MANY_ROWS");
    }
    if rows.iter().any(|row| {
        [
            &row.xymd, &row.open, &row.high, &row.low, &row.clos, &row.tvol,
        ]
        .iter()
        .any(|value| value.trim().is_empty())
    }) {
        anyhow::bail!("BROKER_CHART_EMPTY_FIELD");
    }
    Ok(rows
        .into_iter()
        .map(|row| ChartCandle {
            date: row.xymd,
            open: row.open,
            high: row.high,
            low: row.low,
            close: row.clos,
            volume: row.tvol,
        })
        .collect())
}

fn parse_sector_daily_body(
    success: bool,
    body: &str,
) -> Result<Vec<crate::trading::sector_data::SectorQuoteRow>> {
    if !success {
        anyhow::bail!("SECTOR_QUOTE_HTTP_FAILED");
    }
    #[derive(Deserialize)]
    struct Raw {
        rt_cd: String,
        output2: Option<Vec<Row>>,
    }
    #[derive(Deserialize)]
    struct Row {
        xymd: String,
        open: String,
        clos: String,
    }
    let raw: Raw =
        serde_json::from_str(body).map_err(|_| anyhow::anyhow!("SECTOR_QUOTE_INVALID_RESPONSE"))?;
    if raw.rt_cd != "0" {
        anyhow::bail!("SECTOR_QUOTE_PROVIDER_REJECTED");
    }
    let rows = raw
        .output2
        .ok_or_else(|| anyhow::anyhow!("SECTOR_QUOTE_MISSING_ROWS"))?;
    if rows.len() > 500 {
        anyhow::bail!("SECTOR_QUOTE_TOO_MANY_ROWS");
    }
    Ok(rows
        .into_iter()
        .map(|row| crate::trading::sector_data::SectorQuoteRow {
            date: row.xymd,
            open: row.open,
            close: row.clos,
        })
        .collect())
}

async fn parse_domestic_response(resp: Response) -> Result<Vec<ChartCandle>> {
    let status = resp.status();
    let body = read_kis_response_text(resp).await?;
    if !status.is_success() {
        anyhow::bail!(
            "차트 조회 HTTP {}: {}",
            status,
            body.chars().take(300).collect::<String>()
        );
    }

    #[derive(Deserialize)]
    struct Raw {
        rt_cd: String,
        msg1: String,
        output2: Option<Vec<RawCandle>>,
    }

    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct RawCandle {
        stck_bsop_date: String,
        stck_oprc: String,
        stck_hgpr: String,
        stck_lwpr: String,
        stck_clpr: String,
        acml_vol: String,
    }

    let raw: Raw = serde_json::from_str(&body).map_err(|e| {
        tracing::error!(
            "차트 JSON 파싱 실패: {}\nbody preview: {}",
            e,
            body.chars().take(300).collect::<String>()
        );
        anyhow::anyhow!("차트 JSON 파싱 실패: {}", e)
    })?;
    if raw.rt_cd != "0" {
        anyhow::bail!("차트 조회 오류: {}", raw.msg1);
    }

    let mut candles: Vec<ChartCandle> = raw
        .output2
        .unwrap_or_default()
        .into_iter()
        .filter(|c| !c.stck_bsop_date.is_empty())
        .map(|c| ChartCandle {
            date: c.stck_bsop_date,
            open: c.stck_oprc,
            high: c.stck_hgpr,
            low: c.stck_lwpr,
            close: c.stck_clpr,
            volume: c.acml_vol,
        })
        .collect();
    // KIS 최신순 반환 → 기존 공개 메서드의 오름차순 계약 유지.
    candles.reverse();
    Ok(candles)
}

async fn parse_overseas_response(resp: Response) -> Result<Vec<ChartCandle>> {
    let status = resp.status();
    let body = read_kis_response_text(resp).await?;
    if !status.is_success() {
        anyhow::bail!(
            "해외 차트 조회 HTTP {}: {}",
            status,
            body.chars().take(300).collect::<String>()
        );
    }

    #[derive(Deserialize)]
    struct Raw {
        rt_cd: String,
        msg1: String,
        output2: Option<Vec<RawOverseasCandle>>,
    }

    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct RawOverseasCandle {
        xymd: String,
        open: String,
        high: String,
        low: String,
        clos: String,
        tvol: String,
    }

    let raw: Raw = serde_json::from_str(&body).map_err(|e| {
        tracing::error!(
            "해외 차트 JSON 파싱 실패: {}\nbody: {}",
            e,
            body.chars().take(300).collect::<String>()
        );
        anyhow::anyhow!("해외 차트 JSON 파싱 실패: {}", e)
    })?;
    if raw.rt_cd != "0" {
        anyhow::bail!("해외 차트 조회 오류: {}", raw.msg1);
    }

    let mut candles: Vec<ChartCandle> = raw
        .output2
        .unwrap_or_default()
        .into_iter()
        .filter(|c| !c.xymd.is_empty())
        .map(|c| ChartCandle {
            date: c.xymd,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.clos,
            volume: c.tvol,
        })
        .collect();
    candles.reverse();
    Ok(candles)
}

#[cfg(test)]
#[path = "chart_tests.rs"]
mod tests;
