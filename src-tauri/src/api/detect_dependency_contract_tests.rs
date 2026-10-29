use super::*;
use std::error::Error;
#[test]
fn detection_errors_keep_fixed_display_from_and_source() {
    let unknown = DetectTradingTypeError::Unknown;
    assert_eq!(unknown.to_string(),"실전/모의 키를 자동 감지하지 못했습니다. 네트워크 또는 API 키를 확인하거나 직접 선택해 주세요.");
    assert!(unknown.source().is_none());
    // Invalid header fails locally while building; no request is ever sent.
    let source = crate::http::client_builder()
        .user_agent("invalid\nagent")
        .build()
        .unwrap_err();
    assert!(source.is_builder());
    let error = DetectTradingTypeError::from(source);
    assert_eq!(error.to_string(), "HTTP 클라이언트 오류: builder error");
    assert!(error
        .source()
        .unwrap()
        .downcast_ref::<reqwest::Error>()
        .unwrap()
        .is_builder());
    assert!(matches!(error, DetectTradingTypeError::Request(_)));
}
