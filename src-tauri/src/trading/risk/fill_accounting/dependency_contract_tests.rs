//! Receipt literals protect already persisted identities across dependency versions.
use super::*;
#[test]
fn risk_receipts_preserve_fixed_keys_payloads_and_restored_deduplication() {
    let mut risk = RiskManager::new(10_000, 0.2);
    let date = chrono::Local::now().date_naive();
    let cases = [
        (
            BrokerId::Kis,
            Some("fixture-account"),
            -50,
            r#"[{"brokerId":"kis","accountId":"fixture-account"},"synthetic-event"]"#,
            r#"["fixture-strategy","SOXQ",-50]"#,
            "2e904133114c1a1a77d89c391914a4df067dbb6cd9eb5e2a8dadd4bb3ebf99ad",
            "8002c1a619d71a3cc047f29e5680713d0a562532fffd6742a1c12248447bd261",
        ),
        (
            BrokerId::Kis,
            None,
            0,
            r#"[{"brokerId":"kis","accountId":null},"synthetic-event"]"#,
            r#"["fixture-strategy","SOXQ",0]"#,
            "d5b5f33efbb1407c58893173d1a4e8ddd4bbf20d4b7e79720496f37a6abaa367",
            "b62aec8fcc6323e13c7cc29bac92ac32133bbad336d1719d6a4daf5a15bd9a70",
        ),
        (
            BrokerId::Toss,
            Some("fixture-account"),
            75,
            r#"[{"brokerId":"toss","accountId":"fixture-account"},"synthetic-event"]"#,
            r#"["fixture-strategy","SOXQ",75]"#,
            "9c5aca4c8fb401e3cd6031e89196c2cbd2588c457f4341d3e0b64dbc98c06e5c",
            "07af1844ae72a4f47cab0c4781dfa3ed34c65c58f22daa7cba17cb662a6be10c",
        ),
    ];
    for (broker, account, pnl, key_bytes, payload_bytes, key, payload) in cases {
        let scope = BrokerScope::new(broker, account.map(|v| BrokerAccountId(v.into())));
        assert_eq!(
            serde_json::to_string(&(&scope, "synthetic-event")).unwrap(),
            key_bytes
        );
        assert_eq!(
            serde_json::to_string(&("fixture-strategy", "SOXQ", pnl)).unwrap(),
            payload_bytes
        );
        assert!(risk
            .record_fill_pnl_once(
                date,
                "synthetic-event",
                &scope,
                "fixture-strategy",
                "SOXQ",
                pnl
            )
            .unwrap());
        assert_eq!(
            risk.runtime_state()
                .processed_fill_pnl
                .get(key)
                .map(String::as_str),
            Some(payload)
        );
        let mut restored = roundtrip(&risk);
        assert!(!restored
            .record_fill_pnl_once(
                date,
                "synthetic-event",
                &scope,
                "fixture-strategy",
                "SOXQ",
                pnl
            )
            .unwrap());
        assert!(restored
            .record_fill_pnl_once(
                date,
                "synthetic-event",
                &scope,
                "fixture-strategy",
                "SOXQ",
                pnl + 1
            )
            .is_err());
    }
    assert_eq!(risk.runtime_state().processed_fill_pnl.len(), 3);
}
