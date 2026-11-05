//! Fixed while sha2 0.10.9 / thiserror 1.0.69 are still selected.
use super::*;
fn profile() -> AccountProfile {
    AccountProfile {
        id: "fixture-profile".into(),
        broker_id: BrokerId::Toss,
        name: "synthetic".into(),
        is_paper_trading: false,
        live_trading_consent: false,
        app_key: "synthetic-key".into(),
        app_secret: "synthetic-secret".into(),
        account_no: "fixture-account".into(),
    }
}
#[test]
fn cursor_scope_preserves_fixed_input_bytes_and_digest() {
    let cases = [
        (
            0,
            r#"["fixture-profile","toss","fixture-account",false,"SOXQ",true,"1m",null,200]"#,
            "97d8c16fcdc65989a4e1dbf44f8d9b4f9079345b57fa4add8789d8bb0b242e50",
        ),
        (
            1,
            r#"["fixture-profile","toss","fixture-account",false,"SOXQ",true,"1m","NAS",200]"#,
            "3ff525bcd7ea15b62b739d23514a392ed10f721f8441bebd3555007966772550",
        ),
        (
            2,
            r#"["fixture-profile","toss","fixture-account",false,"SOXQ",true,"1m",null,50]"#,
            "b2697711ba7fee9f19e7db865addeaff5b568c2d3520d1f7e54dc3faf47e9697",
        ),
        (
            3,
            r#"["fixture-profile","kis","fixture-account",true,"005930",false,"D",null,200]"#,
            "1d2f9d64330a7de143f63b701a7a787bbd634a7199545b59558607ac1eb80cb5",
        ),
    ];
    for (case, bytes, expected) in cases {
        let mut p = profile();
        let mut input = input("1m");
        match case {
            1 => input.exchange = Some("NAS".into()),
            2 => input.count = Some(50),
            3 => {
                p.broker_id = BrokerId::Kis;
                p.is_paper_trading = true;
                input.symbol = "005930".into();
                input.is_overseas = false;
                input.interval = "D".into();
            }
            _ => {}
        }
        let contract = serde_json::json!([
            p.id,
            p.broker_id,
            p.account_no,
            p.is_paper_trading,
            input.symbol,
            input.is_overseas,
            input.interval,
            input.exchange,
            input.count.unwrap_or(200)
        ]);
        assert_eq!(contract.to_string(), bytes);
        assert_eq!(scope_key(&input, &p), expected);
    }
}
#[test]
fn cursor_default_count_and_unhashed_credentials_keep_the_fixed_identity() {
    let mut p = profile();
    let mut input = input("1m");
    let expected = "97d8c16fcdc65989a4e1dbf44f8d9b4f9079345b57fa4add8789d8bb0b242e50";
    assert_eq!(scope_key(&input, &p), expected);
    input.count = None;
    p.app_key = "different-synthetic-key".into();
    p.app_secret = "different-synthetic-secret".into();
    p.name = "different display name".into();
    p.live_trading_consent = true;
    assert_eq!(scope_key(&input, &p), expected);
    p.account_no.push_str("-other");
    assert_ne!(scope_key(&input, &p), expected);
    p = profile();
    p.id.push_str("-other");
    assert_ne!(scope_key(&input, &p), expected);
    p = profile();
    input.interval = "D".into();
    assert_ne!(scope_key(&input, &p), expected);
}
