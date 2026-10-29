use super::*;
use std::error::Error;
#[test]
fn configuration_error_keeps_code_message_and_anyhow_identity() {
    let error =
        ConfigurationUpdateError::new("INVALID_STRATEGY_PARAMS", "합성 설정이 올바르지 않습니다.");
    assert_eq!(error.code, "INVALID_STRATEGY_PARAMS");
    assert_eq!(error.message, "합성 설정이 올바르지 않습니다.");
    assert_eq!(error.to_string(), "합성 설정이 올바르지 않습니다.");
    assert!(error.source().is_none());
    let wrapped = anyhow::Error::new(error).context("synthetic update");
    let typed = wrapped.downcast_ref::<ConfigurationUpdateError>().unwrap();
    assert_eq!(typed.code, "INVALID_STRATEGY_PARAMS");
    assert_eq!(
        wrapped.chain().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["synthetic update", "합성 설정이 올바르지 않습니다."]
    );
}
