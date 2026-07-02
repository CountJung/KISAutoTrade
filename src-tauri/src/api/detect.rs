use serde::Serialize;

const REAL_TOKEN_URL: &str = "https://openapi.koreainvestment.com:9443/oauth2/tokenP";
const PAPER_TOKEN_URL: &str = "https://openapivts.koreainvestment.com:29443/oauth2/tokenP";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenDomain {
    Real,
    Paper,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenProbe {
    Issued,
    RejectedPaperKey,
    RejectedRealKey,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedTradingType {
    Real,
    Paper,
}

impl DetectedTradingType {
    pub fn is_paper(self) -> bool {
        matches!(self, Self::Paper)
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::Real => "실전투자 키로 확인되었습니다.",
            Self::Paper => "모의투자 키로 확인되었습니다.",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DetectTradingTypeError {
    #[error("HTTP 클라이언트 오류: {0}")]
    Request(#[from] reqwest::Error),
    #[error("실전/모의 키를 자동 감지하지 못했습니다. 네트워크 또는 API 키를 확인하거나 직접 선택해 주세요.")]
    Unknown,
}

#[derive(Serialize)]
struct DetectTokenReq<'a> {
    grant_type: &'static str,
    appkey: &'a str,
    appsecret: &'a str,
}

pub async fn detect_trading_type(
    client: &reqwest::Client,
    app_key: &str,
    app_secret: &str,
) -> Result<DetectedTradingType, DetectTradingTypeError> {
    let real_probe = probe_token_domain(client, TokenDomain::Real, app_key, app_secret).await?;
    match real_probe {
        TokenProbe::Issued | TokenProbe::RejectedRealKey => return Ok(DetectedTradingType::Real),
        TokenProbe::RejectedPaperKey => {
            let paper_probe =
                probe_token_domain(client, TokenDomain::Paper, app_key, app_secret).await?;
            return match paper_probe {
                TokenProbe::Issued | TokenProbe::RejectedPaperKey | TokenProbe::Rejected => {
                    Ok(DetectedTradingType::Paper)
                }
                TokenProbe::RejectedRealKey => Ok(DetectedTradingType::Real),
            };
        }
        TokenProbe::Rejected => {}
    }

    let paper_probe = probe_token_domain(client, TokenDomain::Paper, app_key, app_secret).await?;
    match paper_probe {
        TokenProbe::Issued | TokenProbe::RejectedPaperKey => Ok(DetectedTradingType::Paper),
        TokenProbe::RejectedRealKey => Ok(DetectedTradingType::Real),
        TokenProbe::Rejected => Err(DetectTradingTypeError::Unknown),
    }
}

async fn probe_token_domain(
    client: &reqwest::Client,
    domain: TokenDomain,
    app_key: &str,
    app_secret: &str,
) -> Result<TokenProbe, reqwest::Error> {
    let url = match domain {
        TokenDomain::Real => REAL_TOKEN_URL,
        TokenDomain::Paper => PAPER_TOKEN_URL,
    };
    let resp = client
        .post(url)
        .header("content-type", "application/json; charset=utf-8")
        .json(&DetectTokenReq {
            grant_type: "client_credentials",
            appkey: app_key,
            appsecret: app_secret,
        })
        .send()
        .await?;

    let status_success = resp.status().is_success();
    let body = resp.text().await.unwrap_or_default();
    Ok(classify_token_response(domain, status_success, &body))
}

fn classify_token_response(domain: TokenDomain, status_success: bool, body: &str) -> TokenProbe {
    if status_success && response_has_access_token(body) {
        return TokenProbe::Issued;
    }

    let text = response_message_text(body);
    match domain {
        TokenDomain::Real if is_paper_key_rejected_by_real_domain(&text) => {
            TokenProbe::RejectedPaperKey
        }
        TokenDomain::Paper if is_real_key_rejected_by_paper_domain(&text) => {
            TokenProbe::RejectedRealKey
        }
        _ => TokenProbe::Rejected,
    }
}

fn response_has_access_token(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("access_token")
                .and_then(|token| token.as_str())
                .map(|token| !token.is_empty())
        })
        .unwrap_or(false)
}

fn response_message_text(body: &str) -> String {
    let mut parts = Vec::new();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        for key in [
            "msg1",
            "msg_cd",
            "error",
            "error_description",
            "message",
            "detail",
        ] {
            if let Some(text) = value.get(key).and_then(|v| v.as_str()) {
                parts.push(text.to_string());
            }
        }
    }
    parts.push(body.to_string());
    parts.join(" ").to_ascii_lowercase()
}

fn mentions_app_key(text: &str) -> bool {
    text.contains("앱키")
        || text.contains("appkey")
        || text.contains("app_key")
        || text.contains("app key")
}

fn is_paper_key_rejected_by_real_domain(text: &str) -> bool {
    let mentions_paper_key = text.contains("모의투자") && mentions_app_key(text);
    let explicitly_cross_domain =
        text.contains("실전투자") || text.contains("도메인") || text.contains("domain");
    mentions_paper_key && explicitly_cross_domain
}

fn is_real_key_rejected_by_paper_domain(text: &str) -> bool {
    let mentions_real_key = text.contains("실전투자") && mentions_app_key(text);
    let explicitly_cross_domain =
        text.contains("모의투자") || text.contains("도메인") || text.contains("domain");
    mentions_real_key && explicitly_cross_domain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_successful_token_as_issued() {
        let body = r#"{"access_token":"abc","token_type":"Bearer"}"#;
        assert_eq!(
            classify_token_response(TokenDomain::Real, true, body),
            TokenProbe::Issued
        );
    }

    #[test]
    fn classifies_paper_key_rejected_by_real_domain() {
        let body = r#"{"rt_cd":"1","msg_cd":"OPSQ0000","msg1":"실전투자 도메인은 모의투자 앱키로 호출할 수 없습니다."}"#;
        assert_eq!(
            classify_token_response(TokenDomain::Real, true, body),
            TokenProbe::RejectedPaperKey
        );
    }

    #[test]
    fn classifies_real_key_rejected_by_paper_domain() {
        let body = r#"{"rt_cd":"1","msg_cd":"OPSQ0000","msg1":"모의투자 도메인은 실전투자 앱키로 호출할 수 없습니다."}"#;
        assert_eq!(
            classify_token_response(TokenDomain::Paper, true, body),
            TokenProbe::RejectedRealKey
        );
    }

    #[test]
    fn classifies_paper_appkey_rejected_by_real_domain_without_domain_word() {
        let body = r#"{"msg1":"실전투자에서는 모의투자 appkey로 호출할 수 없습니다."}"#;
        assert_eq!(
            classify_token_response(TokenDomain::Real, false, body),
            TokenProbe::RejectedPaperKey
        );
    }

    #[test]
    fn classifies_paper_key_from_plain_text_real_domain_error() {
        let body = "실전투자에서는 모의투자 앱키를 사용할 수 없습니다.";
        assert_eq!(
            classify_token_response(TokenDomain::Real, false, body),
            TokenProbe::RejectedPaperKey
        );
    }
}
