//! Step 2: hash algorithms and hash types.
mod common;

use bee_rpc::hash::HashAlgorithm;

#[test]
fn hash_types_match_the_reference() {
    let m = common::manifest();
    for alg in HashAlgorithm::ALL {
        let v = &m["hash_types"][alg.name()];
        assert_eq!(
            hex::encode(alg.hash_type()),
            v["hash_type"].as_str().unwrap(),
            "{}",
            alg.name()
        );
    }
}

#[test]
fn digests_match_the_reference() {
    let m = common::manifest();
    for alg in HashAlgorithm::ALL {
        let v = &m["hash_types"][alg.name()];
        assert_eq!(
            hex::encode(alg.digest(b"abc")),
            v["digest_of_abc"].as_str().unwrap(),
            "{}",
            alg.name()
        );
    }
}

#[test]
fn hash_type_is_digest_of_empty_input() {
    for alg in HashAlgorithm::ALL {
        assert_eq!(alg.hash_type(), alg.digest(b""), "{}", alg.name());
    }
}

#[test]
fn from_hash_type_round_trips() {
    for alg in HashAlgorithm::ALL {
        assert_eq!(HashAlgorithm::from_hash_type(&alg.hash_type()), Some(alg));
    }
    assert_eq!(HashAlgorithm::from_hash_type(b"not a hash type"), None);
    assert_eq!(HashAlgorithm::from_hash_type(b""), None);
}

#[test]
fn default_is_the_reference_default() {
    let m = common::manifest();
    assert_eq!(
        hex::encode(HashAlgorithm::default().hash_type()),
        m["default_hash_type"].as_str().unwrap()
    );
}

#[test]
fn incremental_equals_one_shot() {
    let data: Vec<u8> = (0..10_000u32).map(|i| (i * 7 % 251) as u8).collect();
    for alg in HashAlgorithm::ALL {
        let mut h = alg.hasher();
        for piece in data.chunks(333) {
            h.update(piece);
        }
        assert_eq!(h.finalize(), alg.digest(&data), "{}", alg.name());
    }
}
