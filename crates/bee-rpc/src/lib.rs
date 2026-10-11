//! Rust port of [bee-rpc-over-grpc-py](https://github.com/bee-rpc-protocol/bee-rpc-over-grpc-py).
//!
//! The wire format is `buffer.Buffer` (see `protos/buffer.proto`). The reference
//! is the Python library at v0.0.2. `PORTING.md` lists what to port, in order.

#![allow(clippy::all, missing_docs)]

pub mod buffer {
    include!("gen/buffer.rs");
}

pub use prost::Message;

#[cfg(test)]
mod tests {
    use super::buffer::Buffer;
    use super::Message;

    #[test]
    fn empty_buffer_round_trips() {
        let b = Buffer::default();
        assert_eq!(Buffer::decode(b.encode_to_vec().as_slice()).unwrap(), b);
    }
}
