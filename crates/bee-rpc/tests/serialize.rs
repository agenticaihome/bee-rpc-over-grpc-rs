//! Step 4: the serializer. The output must be byte-identical to the reference.
mod common;

use bee_rpc::buffer::Buffer;
use bee_rpc::frame::{serialize_part, serialize_parts, Part};
use bee_rpc::CHUNK_SIZE;

#[test]
fn byte_identical_to_the_reference() {
    let m = common::manifest();
    for case in common::cases(&m, "serialize") {
        let name = case["name"].as_str().unwrap();
        let file = case["stream"].as_str().unwrap();
        let python = common::stream(file);

        // The input of the case, cut from the reference stream by the test itself,
        // and checked against the manifest.
        let inputs = common::naive_messages(&python);
        let want_inputs = case["inputs"].as_array().unwrap();
        assert_eq!(inputs.len(), want_inputs.len(), "{name}: inputs");
        for ((index, data), want) in inputs.iter().zip(want_inputs) {
            assert_eq!(
                *index as i64,
                want["index"].as_i64().unwrap(),
                "{name}: index"
            );
            assert_eq!(
                common::summary(data),
                common::summary_of(want),
                "{name}: data"
            );
        }

        let parts: Vec<Part> = inputs.into_iter().map(|(i, d)| Part::new(i, d)).collect();
        let got = serialize_parts(&parts);
        assert_eq!(
            got.len() as u64,
            case["buffers"].as_u64().unwrap(),
            "{name}: buffer count"
        );
        assert!(
            common::encode(&got) == common::records(file),
            "{name}: bytes differ from the reference"
        );
    }
}

fn count(data_len: usize) -> Vec<Buffer> {
    serialize_part(&Part::new(4, vec![7u8; data_len])).collect()
}

#[test]
fn small_part_is_one_buffer() {
    for len in [0, 1, 100, CHUNK_SIZE - 1] {
        let b = count(len);
        assert_eq!(b.len(), 1, "len {len}");
        assert_eq!(b[0].head.as_ref().unwrap().index, 4);
        assert_eq!(b[0].chunk.as_ref().unwrap().len(), len);
        assert_eq!(b[0].separator, Some(true));
        assert!(b[0].block.is_none() && b[0].skip.is_none() && b[0].signal.is_none());
    }
}

#[test]
fn large_part_is_head_chunks_separator() {
    for (len, chunks) in [
        (CHUNK_SIZE, 1),
        (CHUNK_SIZE + 1, 2),
        (3 * CHUNK_SIZE, 3),
        (3 * CHUNK_SIZE + 9, 4),
    ] {
        let b = count(len);
        assert_eq!(b.len(), chunks + 2, "len {len}");
        assert_eq!(
            b[0],
            Buffer {
                head: b[0].head.clone(),
                ..Default::default()
            }
        );
        assert_eq!(b[0].head.as_ref().unwrap().index, 4);
        let mut total = 0;
        for (k, c) in b[1..=chunks].iter().enumerate() {
            let n = c.chunk.as_ref().unwrap().len();
            assert!(n <= CHUNK_SIZE && n > 0);
            if k + 1 < chunks {
                assert_eq!(n, CHUNK_SIZE);
            }
            assert!(c.head.is_none() && c.separator.is_none());
            total += n;
        }
        assert_eq!(total, len);
        assert_eq!(
            b[chunks + 1],
            Buffer {
                separator: Some(true),
                ..Default::default()
            }
        );
    }
}

#[test]
fn index_zero_still_has_a_head() {
    let b: Vec<Buffer> = serialize_part(&Part::new(0, b"x".to_vec())).collect();
    assert_eq!(b[0].head.as_ref().map(|h| h.index), Some(0));
}

#[test]
fn chunks_carry_the_data_in_order() {
    let data: Vec<u8> = (0..(2 * CHUNK_SIZE + 77))
        .map(|i| (i % 253) as u8)
        .collect();
    let joined: Vec<u8> = serialize_part(&Part::new(1, data.clone()))
        .filter_map(|b| b.chunk)
        .flatten()
        .collect();
    assert!(joined == data);
}
