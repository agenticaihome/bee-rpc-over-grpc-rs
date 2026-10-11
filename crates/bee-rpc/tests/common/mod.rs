//! Helpers for the tests. They read the vectors with their own code, so a bug in
//! the crate's .bee reader cannot hide a bug elsewhere.
#![allow(dead_code)]

use std::path::PathBuf;

use bee_rpc::buffer::Buffer;
use bee_rpc::Message;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors")
}

pub fn manifest() -> Value {
    let raw = std::fs::read(vectors_dir().join("manifest.json")).expect("read manifest.json");
    serde_json::from_slice(&raw).expect("parse manifest.json")
}

/// The raw records of a .bee file: the bytes of each Buffer, in order.
pub fn records(name: &str) -> Vec<Vec<u8>> {
    let raw =
        std::fs::read(vectors_dir().join(name)).unwrap_or_else(|e| panic!("read {name}: {e}"));
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < raw.len() {
        let len = u32::from_be_bytes(raw[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        out.push(raw[pos..pos + len].to_vec());
        pos += len;
    }
    assert_eq!(pos, raw.len(), "{name}: trailing bytes");
    out
}

/// The Buffers of a .bee vector file.
pub fn stream(name: &str) -> Vec<Buffer> {
    records(name)
        .iter()
        .map(|r| Buffer::decode(r.as_slice()).expect("decode Buffer"))
        .collect()
}

pub fn encode(buffers: &[Buffer]) -> Vec<Vec<u8>> {
    buffers.iter().map(|b| b.encode_to_vec()).collect()
}

pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

pub fn unhex(v: &Value) -> Vec<u8> {
    hex::decode(v.as_str().expect("hex string")).expect("valid hex")
}

pub fn cases<'a>(m: &'a Value, key: &str) -> &'a Vec<Value> {
    m[key]
        .as_array()
        .unwrap_or_else(|| panic!("manifest has no list {key}"))
}

pub fn indices(case: &Value) -> Vec<i32> {
    case["parse_indices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i32)
        .collect()
}

/// A summary of a data blob that is easy to compare: (len, sha256).
pub fn summary(data: &[u8]) -> (u64, String) {
    (data.len() as u64, sha256_hex(data))
}

pub fn summary_of(v: &Value) -> (u64, String) {
    (
        v["len"].as_u64().unwrap(),
        v["sha256"].as_str().unwrap().to_string(),
    )
}

/// The messages of a stream without blocks, cut by the test itself:
/// (index, bytes) per message. Used to get the input of the serialize vectors.
pub fn naive_messages(buffers: &[Buffer]) -> Vec<(i32, Vec<u8>)> {
    let mut out = Vec::new();
    let mut open: Option<(i32, Vec<u8>)> = None;
    for b in buffers {
        assert!(b.block.is_none(), "naive_messages: a stream with blocks");
        if let Some(h) = &b.head {
            assert!(open.is_none(), "head inside an open message");
            open = Some((h.index, Vec::new()));
        }
        if let Some(c) = &b.chunk {
            open.as_mut()
                .expect("chunk outside a message")
                .1
                .extend_from_slice(c);
        }
        if b.separator == Some(true) {
            out.push(open.take().expect("separator outside a message"));
        }
    }
    assert!(open.is_none(), "stream ends inside a message");
    out
}
