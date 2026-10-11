//! A generic Bee-RPC call over tonic (`client.client_grpc`). Feature `grpc`.
//!
//! Each Bee-RPC method is `rpc M(stream buffer.Buffer) returns (stream buffer.Buffer)`.
//! No generated stubs: the call uses a prost codec for `Buffer` on any path.
//!
//! Rules:
//!
//! - The request direction sends the parts with [`crate::frame::serialize_part`]
//!   and waits on the [`crate::signal::Signal`] before each Buffer.
//! - The response direction goes through a [`crate::frame::Parser`] that shares
//!   that signal.
//! - With `block_skip`, the request direction stays open after the input, and sends
//!   the skip requests of the parse side until the response ends. Without it, the
//!   request direction ends after the input (the reference default).
//! - No blocking call on the async runtime.

use std::sync::Arc;

use crate::frame::{BlockStore, ParseOptions, Part};
use crate::Error;

/// Options of [`call`].
#[derive(Clone, Debug, Default)]
pub struct CallOptions {
    /// Parser options for the response.
    pub parse: ParseOptions,
    /// Hold the request direction open for skip requests (`client_grpc(block_skip=True)`).
    pub block_skip: bool,
    /// A deadline for the whole call.
    pub timeout: Option<std::time::Duration>,
}

/// Call the bidi method at `path` (for example `/bee.WireHarness/Exchange`) on
/// `channel`, send `parts`, and return all response parts.
pub async fn call(
    channel: tonic::transport::Channel,
    path: &str,
    parts: Vec<Part>,
    options: CallOptions,
) -> Result<Vec<Part>, Error> {
    let _ = (channel, path, parts, options);
    todo!("step 7: grpc call")
}

/// [`call`], with a [`BlockStore`] for the response: blocks that the store holds
/// come from it, and with `block_skip` the peer is asked to skip them.
pub async fn call_with_store(
    channel: tonic::transport::Channel,
    path: &str,
    parts: Vec<Part>,
    options: CallOptions,
    store: Arc<dyn BlockStore>,
) -> Result<Vec<Part>, Error> {
    let _ = (channel, path, parts, options, store);
    todo!("step 7: grpc call")
}
