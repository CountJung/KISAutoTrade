//! Synthetic structs only; never opens profiles or constructs a provider client.
use super::*;
fn profile() -> AccountProfile {
    AccountProfile {
        id: "fixture-profile".into(),
        broker_id: BrokerId::Kis,
        name: "synthetic".into(),
        is_paper_trading: false,
        live_trading_consent: false,
        app_key: "synthetic-key".into(),
        app_secret: "synthetic-secret".into(),
        account_no: "fixture-account".into(),
    }
}
#[test]
fn execution_fingerprint_fixed_utf8_prefixes_and_array_output() {
    let cases=[
        ("fixture-profile", "fixture-account", "synthetic-key", "synthetic-secret", false, false, "0f00000000000000666978747572652d70726f66696c650f00000000000000666978747572652d6163636f756e740d0000000000000073796e7468657469632d6b6579100000000000000073796e7468657469632d7365637265740000", "c0d3d8b6f1ed2f41acbc512665e6e0db3b2b9e759f1482c0c11af4e347462d83"),
        ("ab", "c", "key", "secret", true, false, "0200000000000000616201000000000000006303000000000000006b657906000000000000007365637265740100", "dec7880d467974c7a1de0e7352e55b28c578b30fccf60fc9ef614c6299b53513"),
        ("a", "bc", "key", "secret", true, false, "0100000000000000610200000000000000626303000000000000006b657906000000000000007365637265740100", "c913d8ff1ac83b4d009caffeed2ecaad9b42009dd4f3c6008f7994bdac0d558b"),
        ("합성", "계좌", "키", "비밀", false, true, "0600000000000000ed95a9ec84b10600000000000000eab384eca28c0300000000000000ed82a40600000000000000ebb984ebb0800001", "3960e97eb0a745ac5b826b36b5f5fa28d0d318c27c9633b5f1ef71b7df7bc749"),
    ];
    for (id, account, key, secret, paper, consent, bytes, expected) in cases {
        let mut p = profile();
        p.id = id.into();
        p.account_no = account.into();
        p.app_key = key.into();
        p.app_secret = secret.into();
        p.is_paper_trading = paper;
        p.live_trading_consent = consent;
        let mut input = Vec::new();
        for v in [&p.id, &p.account_no, &p.app_key, &p.app_secret] {
            input.extend_from_slice(&(v.len() as u64).to_le_bytes());
            input.extend_from_slice(v.as_bytes());
        }
        input.extend([u8::from(paper), u8::from(consent)]);
        assert_eq!(
            input.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            bytes
        );
        let actual: [u8; 32] = profile_fingerprint(&p);
        assert_eq!(
            actual
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
            expected
        );
    }
}
#[test]
fn execution_fingerprint_preserves_the_existing_field_boundary() {
    let original = profile();
    let expected = profile_fingerprint(&original);
    for field in 0..6 {
        let mut p = original.clone();
        match field {
            0 => p.id.push('x'),
            1 => p.account_no.push('x'),
            2 => p.app_key.push('x'),
            3 => p.app_secret.push('x'),
            4 => p.is_paper_trading = true,
            _ => p.live_trading_consent = true,
        }
        assert_ne!(profile_fingerprint(&p), expected);
    }
    let mut p = original;
    p.name = "another synthetic label".into();
    p.broker_id = BrokerId::Toss;
    assert_eq!(profile_fingerprint(&p), expected); // broker is checked separately by run scope validation
}
