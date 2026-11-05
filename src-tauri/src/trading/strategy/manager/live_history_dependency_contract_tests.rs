//! This fingerprint identifies StrategyConfig, not history candle content.
use super::*;
#[test]
fn daily_history_configuration_keeps_exact_json_and_digest() {
    let mut config = StrategyConfig::new(
        "fixture-strategy",
        "합성 설정",
        true,
        vec!["SOXQ".into(), "SOXL".into()],
        2,
        serde_json::json!({"threshold":1.5}),
    );
    assert_eq!(
        serde_json::to_string(&config).unwrap(),
        r#"{"id":"fixture-strategy","name":"합성 설정","enabled":true,"broker_id":"kis","target_symbols":["SOXQ","SOXL"],"order_quantity":2,"params":{"threshold":1.5}}"#
    );
    assert_eq!(
        fingerprint(&config),
        "d53f73d148c1920729a289155849f66c13ef57fb115a337a0815f4a9759cea81"
    );
    config.set_scope(BrokerId::Toss, Some("fixture-account".into()));
    assert_eq!(
        serde_json::to_string(&config).unwrap(),
        r#"{"id":"fixture-strategy","name":"합성 설정","enabled":true,"broker_id":"toss","broker_account_id":"fixture-account","target_symbols":["SOXQ","SOXL"],"order_quantity":2,"params":{"threshold":1.5}}"#
    );
    assert_eq!(
        fingerprint(&config),
        "e4d84468196d8cb39dbd94125e393a8a3ddb0aaf1a8cbcd19fd3105062cc26d5"
    );
    let expected = fingerprint(&config);
    config.order_quantity = 3;
    assert_ne!(fingerprint(&config), expected);
}
