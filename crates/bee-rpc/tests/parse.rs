//! Step 5: the parser, on streams from the reference.
mod common;

use std::collections::HashMap;
use std::sync::Arc;

use bee_rpc::buffer::Buffer;
use bee_rpc::frame::{parse_all, BlockStore, ParseOptions, Parser, Part};
use bee_rpc::hash::HashAlgorithm;
use bee_rpc::signal::Signal;
use bee_rpc::skip::{skip_buffer, StreamControl};
use bee_rpc::Error;

struct MemStore(HashMap<Vec<u8>, Vec<u8>>);

impl BlockStore for MemStore {
    fn get(&self, id: &[u8]) -> Option<Vec<u8>> {
        self.0.get(id).cloned()
    }
}

fn store_of(case: &serde_json::Value) -> MemStore {
    MemStore(
        case["blocks"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (hex::decode(k).unwrap(), common::unhex(v)))
            .collect(),
    )
}

fn run(parser: &mut Parser, buffers: Vec<Buffer>) -> Result<Vec<Part>, Error> {
    let mut out = Vec::new();
    for b in buffers {
        out.extend(parser.push(b)?);
    }
    out.extend(parser.finish()?);
    Ok(out)
}

#[test]
fn parses_the_serialize_vectors_back_to_their_inputs() {
    let m = common::manifest();
    for case in common::cases(&m, "serialize") {
        let name = case["name"].as_str().unwrap();
        let python = common::stream(case["stream"].as_str().unwrap());
        let parts = parse_all(python, ParseOptions::accepting(common::indices(case)))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let want = case["inputs"].as_array().unwrap();
        assert_eq!(
            parts.len(),
            want.len(),
            "{name}: part count (empty parts included, D4)"
        );
        for (p, w) in parts.iter().zip(want) {
            assert_eq!(p.index as i64, w["index"].as_i64().unwrap(), "{name}");
            assert_eq!(common::summary(&p.data), common::summary_of(w), "{name}");
        }
        // Cross-check with what the reference parser yields (it drops empty parts).
        let nonempty: Vec<_> = parts
            .iter()
            .filter(|p| !p.data.is_empty())
            .map(|p| common::summary(&p.data))
            .collect();
        let python: Vec<_> = case["python_parse"]["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(common::summary_of)
            .collect();
        assert_eq!(
            nonempty, python,
            "{name}: differs from the reference parser"
        );
    }
}

#[test]
fn rebuilds_messages_with_blocks() {
    let m = common::manifest();
    for case in common::cases(&m, "parse") {
        let name = case["name"].as_str().unwrap();
        let mut parser = Parser::new(ParseOptions::accepting(common::indices(case)));
        let parts = run(
            &mut parser,
            common::stream(case["stream"].as_str().unwrap()),
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        let want: Vec<_> = case["python_parse"]["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(common::summary_of)
            .collect();
        let got: Vec<_> = parts.iter().map(|p| common::summary(&p.data)).collect();
        assert_eq!(got, want, "{name}: message bytes");
        assert!(parts.iter().all(|p| p.index == 1), "{name}: index");

        let starts: Vec<(String, u64, Vec<u64>)> = parser
            .block_starts()
            .iter()
            .map(|s| {
                (
                    hex::encode(&s.id),
                    s.depth as u64,
                    s.previous_lengths_position.clone(),
                )
            })
            .collect();
        let want_starts: Vec<(String, u64, Vec<u64>)> = case["block_starts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                (
                    s["id"].as_str().unwrap().to_string(),
                    s["depth"].as_u64().unwrap(),
                    s["previous_lengths_position"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_u64().unwrap())
                        .collect(),
                )
            })
            .collect();
        assert_eq!(starts, want_starts, "{name}: block starts");
    }
}

#[test]
fn held_blocks_come_from_the_store_and_are_skipped() {
    let m = common::manifest();
    for case in common::cases(&m, "parse") {
        let name = case["name"].as_str().unwrap();
        let store = store_of(case);
        let held: Vec<Vec<u8>> = store.0.keys().cloned().collect();
        let control = StreamControl::new(HashAlgorithm::Sha3_256);
        let mut parser = Parser::new(ParseOptions::accepting(common::indices(case)))
            .with_store(Arc::new(store))
            .with_control(control.clone());
        let parts = run(
            &mut parser,
            common::stream(case["stream"].as_str().unwrap()),
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        let want: Vec<_> = case["python_parse"]["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(common::summary_of)
            .collect();
        let got: Vec<_> = parts.iter().map(|p| common::summary(&p.data)).collect();
        assert_eq!(got, want, "{name}: message bytes with a store");

        // Every depth-1 block is held, so each one gets a skip request; nothing
        // that is not held gets one.
        let outbound = control.take_outbound();
        let requested: Vec<Vec<u8>> = outbound
            .iter()
            .map(|b| {
                bee_rpc::skip::skipped_block_id(b, HashAlgorithm::Sha3_256).expect("a skip Buffer")
            })
            .collect();
        for s in case["block_starts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["depth"] == 1)
        {
            let id = common::unhex(&s["id"]);
            assert!(
                requested.contains(&id),
                "{name}: no skip request for a held block"
            );
            assert_eq!(
                outbound
                    .iter()
                    .filter(|b| **b == skip_buffer(&id, HashAlgorithm::Sha3_256))
                    .count(),
                1,
                "{name}: one skip request per block"
            );
        }
        assert!(
            requested.iter().all(|r| held.contains(r)),
            "{name}: skip request for a block not held"
        );
    }
}

fn one_block_case() -> (serde_json::Value, Vec<Buffer>) {
    let m = common::manifest();
    let case = common::cases(&m, "parse")
        .iter()
        .find(|c| c["name"] == "one_block")
        .unwrap()
        .clone();
    let s = common::stream(case["stream"].as_str().unwrap());
    (case, s)
}

/// The one_block stream with one byte of the block content changed.
fn corrupted() -> (serde_json::Value, Vec<Buffer>) {
    let (case, mut s) = one_block_case();
    let open = s.iter().position(|b| b.block.is_some()).unwrap();
    let c = s[open + 1]
        .chunk
        .as_mut()
        .expect("chunk after the start marker");
    c[0] ^= 0xff;
    (case, s)
}

#[test]
fn corrupt_block_content_is_refused() {
    let (case, s) = corrupted();
    let got = parse_all(s, ParseOptions::accepting(common::indices(&case)));
    assert!(
        matches!(got, Err(Error::BlockIdMismatch { .. })),
        "got {got:?}"
    );
}

#[test]
fn corrupt_wire_content_of_a_held_block_is_ignored() {
    let (case, s) = corrupted();
    let mut parser = Parser::new(ParseOptions::accepting(common::indices(&case)))
        .with_store(Arc::new(store_of(&case)));
    let parts = run(&mut parser, s).unwrap();
    let want = common::summary_of(&case["python_parse"]["parts"][0]);
    assert_eq!(common::summary(&parts[0].data), want);
}

#[test]
fn chunk_boundaries_do_not_matter() {
    let m = common::manifest();
    for case in common::cases(&m, "parse") {
        let name = case["name"].as_str().unwrap();
        let mut split = Vec::new();
        for b in common::stream(case["stream"].as_str().unwrap()) {
            match (
                &b.chunk,
                b.head.is_none() && b.block.is_none() && b.separator.is_none(),
            ) {
                (Some(c), true) if c.len() > 1 => {
                    for piece in c.chunks(c.len() / 3 + 1) {
                        split.push(Buffer {
                            chunk: Some(piece.to_vec()),
                            ..Default::default()
                        });
                    }
                }
                _ => split.push(b),
            }
        }
        let parts = parse_all(split, ParseOptions::accepting(common::indices(case)))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            common::summary(&parts[0].data),
            common::summary_of(&case["python_parse"]["parts"][0]),
            "{name}"
        );
    }
}

#[test]
fn messages_come_out_as_soon_as_they_end() {
    let m = common::manifest();
    let case = common::cases(&m, "serialize")
        .iter()
        .find(|c| c["name"] == "three_messages")
        .unwrap();
    let mut parser = Parser::new(ParseOptions::accepting(common::indices(case)));
    let mut seen = 0;
    for b in common::stream(case["stream"].as_str().unwrap()) {
        let done = b.separator == Some(true);
        let out = parser.push(b).unwrap();
        assert_eq!(
            out.len(),
            usize::from(done),
            "a part must come out on its separator"
        );
        seen += out.len();
    }
    assert!(parser.finish().unwrap().is_empty());
    assert_eq!(seen, 3);
}

#[test]
fn signal_buffers_toggle_the_signal() {
    let signal = Signal::new();
    let mut parser = Parser::new(ParseOptions::default()).with_signal(signal.clone());
    let sig = Buffer {
        signal: Some(true),
        ..Default::default()
    };
    parser
        .push(Buffer {
            head: Some(Default::default()),
            ..Default::default()
        })
        .unwrap();
    parser.push(sig.clone()).unwrap();
    assert!(!signal.is_open());
    parser
        .push(Buffer {
            chunk: Some(b"ab".to_vec()),
            ..Default::default()
        })
        .unwrap();
    parser.push(sig).unwrap();
    assert!(signal.is_open());
    let out = parser
        .push(Buffer {
            separator: Some(true),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(out, vec![Part::new(0, b"ab".to_vec())]);
}
