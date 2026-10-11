//! Step 5: parser errors and edge cases.
mod common;

use bee_rpc::buffer::buffer::Head;
use bee_rpc::buffer::Buffer;
use bee_rpc::frame::{parse_all, ParseOptions, Parser, Part};
use bee_rpc::hash::HashAlgorithm;
use bee_rpc::pointer::block_pointer;
use bee_rpc::skip::{skip_buffer, StreamControl};
use bee_rpc::Error;

fn expected_error(name: &str, e: &Error) -> bool {
    match name {
        "unknown_head_index" => matches!(e, Error::UnknownIndex(9)),
        "intersecting_blocks" => matches!(e, Error::IntersectingBlocks),
        "nesting_over_limit" => matches!(e, Error::NestingTooDeep(32)),
        "block_id_mismatch" => matches!(e, Error::BlockIdMismatch { .. }),
        "block_without_hash_type" => matches!(e, Error::UnresolvableBlock),
        other => panic!("no expected error for case {other}: add it here"),
    }
}

#[test]
fn error_vectors() {
    let m = common::manifest();
    for case in common::cases(&m, "parse_errors") {
        let name = case["name"].as_str().unwrap();
        let got = parse_all(
            common::stream(case["stream"].as_str().unwrap()),
            ParseOptions::accepting(common::indices(case)),
        );
        match case["rust_expects"].as_str().unwrap() {
            "error" => match got {
                Err(e) => assert!(expected_error(name, &e), "{name}: wrong error {e:?}"),
                Ok(p) => panic!("{name}: expected an error, got {} parts", p.len()),
            },
            "ok" => {
                let parts = got.unwrap_or_else(|e| panic!("{name}: {e}"));
                let want: Vec<_> = case["python_parse"]["parts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(common::summary_of)
                    .collect();
                let got: Vec<_> = parts.iter().map(|p| common::summary(&p.data)).collect();
                assert_eq!(got, want, "{name}");
            }
            other => panic!("{name}: unknown expectation {other}"),
        }
    }
}

fn head(index: i32) -> Buffer {
    Buffer {
        head: Some(Head {
            index,
            partitions: vec![],
        }),
        ..Default::default()
    }
}
fn chunk(data: &[u8]) -> Buffer {
    Buffer {
        chunk: Some(data.to_vec()),
        ..Default::default()
    }
}
fn sep() -> Buffer {
    Buffer {
        separator: Some(true),
        ..Default::default()
    }
}
fn marker(content: &[u8]) -> Buffer {
    let id = HashAlgorithm::Sha3_256.digest(content);
    Buffer {
        block: Some(block_pointer(&id, HashAlgorithm::Sha3_256, false)),
        ..Default::default()
    }
}

#[test]
fn stream_ending_inside_a_block_is_an_error() {
    let got = parse_all(
        vec![head(1), marker(b"abc"), chunk(b"abc")],
        ParseOptions::default(),
    );
    assert!(matches!(got, Err(Error::UnclosedBlock(_))), "got {got:?}");
}

#[test]
fn index_zero_is_always_accepted() {
    let got = parse_all(
        vec![head(0), chunk(b"x"), sep()],
        ParseOptions::accepting([5]),
    )
    .unwrap();
    assert_eq!(got, vec![Part::new(0, b"x".to_vec())]);
}

#[test]
fn headless_index_is_configurable() {
    let opts = ParseOptions {
        headless_index: 3,
        ..ParseOptions::accepting([3])
    };
    let got = parse_all(vec![chunk(b"x"), sep()], opts).unwrap();
    assert_eq!(got, vec![Part::new(3, b"x".to_vec())]);
}

#[test]
fn empty_message_is_emitted() {
    let got = parse_all(
        vec![Buffer {
            chunk: Some(vec![]),
            separator: Some(true),
            ..head(1)
        }],
        ParseOptions::default(),
    )
    .unwrap();
    assert_eq!(got, vec![Part::new(1, vec![])]);
}

#[test]
fn a_block_inside_a_message_is_expanded_in_place() {
    let got = parse_all(
        vec![
            head(1),
            chunk(b"<"),
            marker(b"abc"),
            chunk(b"abc"),
            marker(b"abc"),
            chunk(b">"),
            sep(),
        ],
        ParseOptions::default(),
    )
    .unwrap();
    assert_eq!(got, vec![Part::new(1, b"<abc>".to_vec())]);
}

#[test]
fn chunk_on_a_start_marker_goes_inside_the_block() {
    let mut start = marker(b"abc");
    start.chunk = Some(b"a".to_vec());
    let got = parse_all(
        vec![head(1), start, chunk(b"bc"), marker(b"abc"), sep()],
        ParseOptions::default(),
    )
    .unwrap();
    assert_eq!(got, vec![Part::new(1, b"abc".to_vec())]);
}

#[test]
fn skip_buffers_are_never_message_bytes() {
    let sb = skip_buffer(&[9u8; 32], HashAlgorithm::Sha3_256);
    let got = parse_all(
        vec![head(1), chunk(b"ab"), sb.clone(), chunk(b"cd"), sep()],
        ParseOptions::default(),
    )
    .unwrap();
    assert_eq!(got, vec![Part::new(1, b"abcd".to_vec())]);
    // Also between messages: an inert skip Buffer starts no message.
    let got = parse_all(
        vec![sb, head(1), chunk(b"x"), sep()],
        ParseOptions::default(),
    )
    .unwrap();
    assert_eq!(got, vec![Part::new(1, b"x".to_vec())]);
}

#[test]
fn skip_requests_from_the_peer_are_recorded() {
    let control = StreamControl::new(HashAlgorithm::Sha3_256);
    let mut p = Parser::new(ParseOptions::default()).with_control(control.clone());
    p.push(skip_buffer(&[7u8; 32], HashAlgorithm::Sha3_256))
        .unwrap();
    assert!(control.should_skip(&[7u8; 32]));
}

#[test]
fn errors_stop_the_parser_state_clean() {
    // After an error, a new parser works: no global state.
    let _ = parse_all(
        vec![head(9), chunk(b"x"), sep()],
        ParseOptions::accepting([1]),
    );
    let got = parse_all(
        vec![head(1), chunk(b"x"), sep()],
        ParseOptions::accepting([1]),
    )
    .unwrap();
    assert_eq!(got, vec![Part::new(1, b"x".to_vec())]);
}
