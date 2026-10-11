//! Step 3: the .bee file format.
mod common;

use bee_rpc::bee_file::{read_bee, write_bee};
use bee_rpc::buffer::Buffer;
use bee_rpc::Error;

fn all_bee_files() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(common::vectors_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".bee"))
        .collect();
    names.sort();
    assert!(names.len() >= 20, "vectors are missing");
    names
}

#[test]
fn reads_the_file_of_the_reference_writer() {
    let m = common::manifest();
    let case = &common::cases(&m, "bee_file")[0];
    let name = case["file"].as_str().unwrap();
    let file = std::fs::File::open(common::vectors_dir().join(name)).unwrap();
    let got: Vec<Buffer> = read_bee(file).collect::<Result<_, _>>().unwrap();
    assert_eq!(got, common::stream(name));
}

#[test]
fn reads_every_vector_file() {
    for name in all_bee_files() {
        let raw = std::fs::read(common::vectors_dir().join(&name)).unwrap();
        let got: Vec<Buffer> = read_bee(raw.as_slice()).collect::<Result<_, _>>().unwrap();
        assert_eq!(got, common::stream(&name), "{name}");
    }
}

#[test]
fn writes_the_same_bytes_as_the_reference() {
    for name in all_bee_files() {
        let raw = std::fs::read(common::vectors_dir().join(&name)).unwrap();
        let mut out = Vec::new();
        write_bee(&mut out, common::stream(&name)).unwrap();
        assert!(out == raw, "{name}: written bytes differ");
    }
}

#[test]
fn empty_file_has_no_buffers() {
    assert_eq!(read_bee(&b""[..]).count(), 0);
}

#[test]
fn cut_length_is_an_error() {
    let mut it = read_bee(&[0u8, 0, 1][..]);
    assert!(matches!(it.next(), Some(Err(Error::Decode(_)))));
}

#[test]
fn cut_record_is_an_error() {
    let mut it = read_bee(&[0u8, 0, 0, 5, 0x0a, 0x01][..]);
    assert!(matches!(it.next(), Some(Err(Error::Decode(_)))));
}

#[test]
fn record_that_is_not_a_buffer_is_an_error() {
    // Field 1 (chunk, length-delimited) with a length past the record end.
    let mut it = read_bee(&[0u8, 0, 0, 2, 0x0a, 0x7f][..]);
    assert!(matches!(it.next(), Some(Err(Error::Decode(_)))));
}

#[test]
fn second_record_error_comes_after_first_buffer() {
    let mut raw = Vec::new();
    write_bee(
        &mut raw,
        vec![Buffer {
            separator: Some(true),
            ..Default::default()
        }],
    )
    .unwrap();
    raw.extend_from_slice(&[0, 0]);
    let mut it = read_bee(raw.as_slice());
    assert_eq!(it.next().unwrap().unwrap().separator, Some(true));
    assert!(matches!(it.next(), Some(Err(Error::Decode(_)))));
}
