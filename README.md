# bee-rpc-over-grpc-rs

Rust port of [bee-rpc-over-grpc-py](https://github.com/bee-rpc-protocol/bee-rpc-over-grpc-py).
Bee-RPC moves buffers of any size over gRPC. It splits a message into blocks, and
each block has a hash, so a receiver can skip a block that it already holds.

Status: **skeleton**. The crate has the `buffer.Buffer` types and nothing else.
`PORTING.md` lists the work.

## Why a Rust port

- The Celaut light client for Android ([celaut-project/chatui](https://github.com/celaut-project/chatui))
  needs Bee-RPC and has no Python.
- One crate for all Rust users, tested against the Python library with the same vectors.

## Compatibility rule

The Python library at v0.0.2 is the reference. A Rust peer and a Python peer must
talk to each other with no change on either side. If the two differ, the Python
library is right, unless the Python library has a bug. Then open an issue in both repos.

## Build

```sh
cargo test --workspace
cargo run -p gen-protos   # after a change of crates/bee-rpc/protos/
```

`src/gen/` is committed, so a normal build does not need `protoc`.

## License

GPL-3.0, as the Python library.
