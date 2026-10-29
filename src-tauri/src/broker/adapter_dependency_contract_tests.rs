use super::*;
use std::error::Error;
#[test]
fn adapter_errors_keep_named_tuple_and_transparent_source_contracts() {
    let unsupported = BrokerAdapterError::Unsupported {
        broker: BrokerId::Toss,
        operation: "place_order",
    };
    assert_eq!(
        unsupported.to_string(),
        "Toss adapter does not support place_order"
    );
    assert!(unsupported.source().is_none());
    assert_eq!(
        BrokerAdapterError::InvalidRequest("합성 입력".into()).to_string(),
        "invalid broker request: 합성 입력"
    );
    let source =
        anyhow::Error::new(std::io::Error::other("synthetic leaf")).context("synthetic provider");
    let error = BrokerAdapterError::from(source);
    assert_eq!(error.to_string(), "synthetic provider");
    assert_eq!(error.source().unwrap().to_string(), "synthetic leaf");
    assert!(error
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .is_some());
    let wrapped = anyhow::Error::new(error).context("synthetic adapter");
    assert_eq!(
        wrapped.chain().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["synthetic adapter", "synthetic provider", "synthetic leaf"]
    );
    assert!(wrapped.downcast_ref::<BrokerAdapterError>().is_some());
}
