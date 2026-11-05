//! Scanner/byte-stream contract only; no DatabaseManager, pool, keychain or DB execution.
use super::super::*;
#[test]
fn database_scanner_preserves_original_payload_order_and_checksum_input() {
    let root = std::env::temp_dir().join(format!("kis-round9-hash-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(root.join("trades")).unwrap();
    std::fs::create_dir_all(root.join("orders")).unwrap();
    // Deliberately create the lexically later document first.
    std::fs::write(root.join("trades/b.json"), "[\n  {\"amount\":7}\n]\n").unwrap();
    std::fs::write(root.join("orders/a.json"), "{\"note\":\"합성\"}\n").unwrap();
    let documents = scan_json_documents_sync(&root).unwrap();
    assert_eq!(documents.len(), 2);
    assert_eq!(documents[0].key, "orders/a.json");
    assert_eq!(documents[1].key, "trades/b.json");
    assert_eq!(documents[0].payload, "{\"note\":\"합성\"}\n");
    assert_eq!(documents[1].payload, "[\n  {\"amount\":7}\n]\n");
    let mut checksum = Sha256::new();
    let mut bytes = Vec::new();
    // Mirrors only the unchanged production checksum loops; this is not a DB roundtrip.
    for document in &documents {
        checksum.update(document.key.as_bytes());
        checksum.update(document.payload.as_bytes());
        bytes.extend_from_slice(document.key.as_bytes());
        bytes.extend_from_slice(document.payload.as_bytes());
    }
    assert_eq!(
        bytes,
        "orders/a.json{\"note\":\"합성\"}\ntrades/b.json[\n  {\"amount\":7}\n]\n".as_bytes()
    );
    assert_eq!(
        hex::encode(checksum.finalize()),
        "50227291ef8be0ade6a81f7c9fc60f4af95c6b59fcae68ce85eb5f076579f302"
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn sha256_known_vectors_keep_hex_and_split_update_contracts() {
    let cases = [
        (
            0,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            1,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            2,
            "bb07fb7499eafc9729ad426730be4f8559fb03667bd137ed86e537b4eb8a5596",
        ),
        (
            3,
            "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
        ),
        (
            4,
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
        ),
        (
            5,
            "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
        ),
    ];
    for (case, expected) in cases {
        let bytes = match case {
            0 => Vec::new(),
            1 => b"abc".to_vec(),
            2 => "합성".as_bytes().to_vec(),
            3 => vec![b'a'; 63],
            4 => vec![b'a'; 64],
            _ => vec![b'a'; 65],
        };
        assert_eq!(hex::encode(Sha256::digest(&bytes)), expected);
        let mut hash = Sha256::new();
        for chunk in bytes.chunks(7) {
            hash.update(chunk);
        }
        assert_eq!(hex::encode(hash.finalize()), expected);
    }
}
