//! Step 7: a real gRPC call to a peer that runs the Python reference.
//!
//! Set `BEE_RPC_PYTHON` to a Python 3.11 with tools/vectors/requirements.txt.
//! Without it the tests stop with a message, except in CI (`CI` is set), where a
//! missing interpreter is a failure: the interop tests must not pass by skipping.
#![cfg(feature = "grpc")]

mod common;

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use bee_rpc::frame::{BlockStore, Part};
use bee_rpc::grpc::{call, CallOptions};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

struct Peer {
    _child: Child,
    channel: tonic::transport::Channel,
}

async fn peer() -> Option<Peer> {
    let python = match std::env::var("BEE_RPC_PYTHON") {
        Ok(p) => p,
        Err(_) if std::env::var_os("CI").is_some() => panic!("BEE_RPC_PYTHON is not set in CI"),
        Err(_) => {
            eprintln!("interop: BEE_RPC_PYTHON is not set; test not run");
            return None;
        }
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut child = Command::new(python)
        .arg("-I")
        .arg(root.join("tools/interop/py_peer.py"))
        .arg("--vectors")
        .arg(common::vectors_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("start the Python peer");
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let line = tokio::time::timeout(Duration::from_secs(30), lines.next_line())
        .await
        .expect("the Python peer did not start")
        .unwrap()
        .expect("the Python peer closed stdout");
    let port: u16 = line
        .strip_prefix("PORT ")
        .expect("PORT line")
        .parse()
        .unwrap();
    let channel = tonic::transport::Endpoint::from_shared(format!("http://127.0.0.1:{port}"))
        .unwrap()
        .connect()
        .await
        .expect("connect to the Python peer");
    Some(Peer {
        _child: child,
        channel,
    })
}

fn opts() -> CallOptions {
    CallOptions {
        timeout: Some(Duration::from_secs(60)),
        ..Default::default()
    }
}

#[tokio::test]
async fn exchange_small_and_large_parts() {
    let Some(p) = peer().await else { return };
    let big: Vec<u8> = (0..(2 * bee_rpc::CHUNK_SIZE + 3))
        .map(|i| (i % 251) as u8)
        .collect();
    let sent = vec![
        Part::new(1, b"one".to_vec()),
        Part::new(2, big),
        Part::new(0, b"raw".to_vec()),
    ];
    let got = call(p.channel, "/bee.WireHarness/Exchange", sent.clone(), opts())
        .await
        .unwrap();

    assert_eq!(got.len(), 1 + sent.len());
    let summary: Vec<(i64, u64, String)> = serde_json::from_slice(&got[0].data).unwrap();
    let want: Vec<(i64, u64, String)> = sent
        .iter()
        .map(|s| {
            (
                s.index as i64,
                s.data.len() as u64,
                common::sha256_hex(&s.data),
            )
        })
        .collect();
    assert_eq!(summary, want, "what the Python peer received");
    for (g, s) in got[1..].iter().zip(&sent) {
        assert_eq!(g.index, 0);
        assert!(g.data == s.data, "echo differs");
    }
}

#[tokio::test]
async fn receive_block_streams_from_python() {
    let Some(p) = peer().await else { return };
    let m = common::manifest();
    for case in common::cases(&m, "parse") {
        let name = case["stream"].as_str().unwrap();
        let got = call(
            p.channel.clone(),
            "/bee.WireHarness/Stream",
            vec![Part::new(1, name.as_bytes().to_vec())],
            opts(),
        )
        .await
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        let want: Vec<_> = case["python_parse"]["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(common::summary_of)
            .collect();
        let got: Vec<_> = got.iter().map(|p| common::summary(&p.data)).collect();
        assert_eq!(got, want, "{name}");
    }
}

struct MemStore(HashMap<Vec<u8>, Vec<u8>>);

impl BlockStore for MemStore {
    fn get(&self, id: &[u8]) -> Option<Vec<u8>> {
        self.0.get(id).cloned()
    }
}

#[tokio::test]
async fn block_skip_saves_bandwidth_with_python() {
    let Some(p) = peer().await else { return };
    let m = common::manifest();
    let case = common::cases(&m, "parse")
        .iter()
        .find(|c| c["name"] == "big_block")
        .unwrap();
    let store = MemStore(
        case["blocks"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (hex::decode(k).unwrap(), common::unhex(v)))
            .collect(),
    );
    let mut o = opts();
    o.block_skip = true;
    let got = bee_rpc::grpc::call_with_store(
        p.channel.clone(),
        "/bee.WireHarness/Skip",
        vec![],
        o,
        Arc::new(store),
    )
    .await
    .unwrap();
    assert_eq!(
        common::summary(&got[0].data),
        common::summary_of(&case["python_parse"]["parts"][0])
    );

    let stats = call(p.channel, "/bee.WireHarness/Stats", vec![], opts())
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&stats[0].data).unwrap();
    let sent = v["skip_chunks_sent"].as_i64().unwrap();
    // Without a skip the peer sends 4 chunks: 1 before the block and 3 in it.
    assert!(
        (1..=2).contains(&sent),
        "the Python peer sent {sent} of 4 chunks: the skip request did not work"
    );
}
