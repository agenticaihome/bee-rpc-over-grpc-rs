//! Regenerates `crates/bee-rpc/src/gen/` from `crates/bee-rpc/protos/`.
//!
//! The generated code is committed, so a normal build does not need `protoc`.
//! Run this after you update the protos: `cargo run -p gen-protos`.

use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/bee-rpc");
    let protos = root.join("protos");
    let out = root.join("src/gen");
    std::fs::create_dir_all(&out).expect("create output directory");
    prost_build::Config::new()
        .out_dir(&out)
        .compile_protos(&[protos.join("buffer.proto")], &[protos])
        .expect("compile protos");
}
