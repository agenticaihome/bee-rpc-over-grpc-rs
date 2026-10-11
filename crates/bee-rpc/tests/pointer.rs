//! Step 2: block pointers and hash-type inheritance.
mod common;

use bee_rpc::buffer::buffer::Block;
use bee_rpc::hash::HashAlgorithm;
use bee_rpc::pointer::{
    block_id_from_pointer, block_pointer, inherit_hash_types, resolve_hash_types,
};
use bee_rpc::Message;

#[test]
fn pointers_match_the_reference() {
    let m = common::manifest();
    for case in common::cases(&m, "pointers") {
        let id = common::unhex(&case["block_id"]);
        let omit = case["omit_types"].as_bool().unwrap();
        let p = block_pointer(&id, HashAlgorithm::Sha3_256, omit);
        assert_eq!(
            hex::encode(p.encode_to_vec()),
            case["pointer"].as_str().unwrap(),
            "omit={omit}"
        );
        assert_eq!(p.encoded_len() as u64, case["length"].as_u64().unwrap());
    }
}

fn inherited_of(case: &serde_json::Value) -> Option<Vec<Vec<u8>>> {
    case["inherited"]
        .as_array()
        .map(|a| a.iter().map(common::unhex).collect())
}

#[test]
fn resolve_matches_the_reference() {
    let m = common::manifest();
    for case in common::cases(&m, "resolve") {
        let name = case["name"].as_str().unwrap();
        let block = Block::decode(common::unhex(&case["pointer"]).as_slice()).unwrap();
        let inherited = inherited_of(case);
        let got = resolve_hash_types(&block, inherited.as_deref());
        match case["resolved"].as_array() {
            None => assert!(
                matches!(got, Err(bee_rpc::Error::HashType(_))),
                "{name}: expected a HashType error, got {got:?}"
            ),
            Some(want) => {
                let want: Vec<Vec<u8>> = want.iter().map(common::unhex).collect();
                let got = got.unwrap_or_else(|e| panic!("{name}: {e}"));
                assert_eq!(got, want, "{name}: resolved");
                let next: Vec<Vec<u8>> = case["inherit_next"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(common::unhex)
                    .collect();
                assert_eq!(
                    inherit_hash_types(&got, inherited.as_deref()),
                    next,
                    "{name}: inherit"
                );
            }
        }
    }
}

#[test]
fn block_id_from_pointer_matches_the_reference() {
    let m = common::manifest();
    for case in common::cases(&m, "resolve") {
        let name = case["name"].as_str().unwrap();
        let block = Block::decode(common::unhex(&case["pointer"]).as_slice()).unwrap();
        let inherited = inherited_of(case);
        let got = block_id_from_pointer(&block, inherited.as_deref(), HashAlgorithm::Sha3_256);
        let want = case["block_id"].as_str().map(|s| hex::decode(s).unwrap());
        assert_eq!(got, want, "{name}");
    }
}

#[test]
fn pointer_round_trips_through_block_id() {
    let id = HashAlgorithm::Sha256.digest(b"content");
    let p = block_pointer(&id, HashAlgorithm::Sha256, false);
    assert_eq!(
        block_id_from_pointer(&p, None, HashAlgorithm::Sha256),
        Some(id.clone())
    );
    assert_eq!(
        block_id_from_pointer(&p, None, HashAlgorithm::Sha3_256),
        None
    );
    let omitted = block_pointer(&id, HashAlgorithm::Sha256, true);
    assert_eq!(
        block_id_from_pointer(&omitted, None, HashAlgorithm::Sha256),
        None
    );
    let ctx = vec![HashAlgorithm::Sha256.hash_type()];
    assert_eq!(
        block_id_from_pointer(&omitted, Some(&ctx), HashAlgorithm::Sha256),
        Some(id)
    );
}
