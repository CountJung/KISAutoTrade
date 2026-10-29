use super::*;
use std::error::Error;
#[test]
fn kis_rejection_keeps_fixed_message_and_typed_anyhow_identity() {
    for message in [
        "주문 오류: 합성 거절",
        "해외 주문 오류: synthetic rejection",
    ] {
        let rejection = KisOrderRejected(message.into());
        assert_eq!(rejection.to_string(), message);
        assert!(rejection.source().is_none());
        let wrapped = anyhow::Error::new(rejection).context("synthetic submit");
        assert_eq!(
            wrapped.downcast_ref::<KisOrderRejected>().unwrap().0,
            message
        );
        assert_eq!(
            wrapped.chain().map(ToString::to_string).collect::<Vec<_>>(),
            vec!["synthetic submit", message]
        );
        assert!(anyhow::anyhow!(message.to_string())
            .downcast_ref::<KisOrderRejected>()
            .is_none());
    }
}
