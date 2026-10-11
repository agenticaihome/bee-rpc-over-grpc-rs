//! Reverse-direction block skip (`control.py`, `client.skip_requested_blocks`).
//!
//! The receiver of a block that it already holds sends a skip request in the
//! other direction: a Buffer with `skip` (field 6) set to a one-hash pointer and
//! an empty `chunk`. The sender then drops the rest of that block.
//!
//! # Sender filter rules ([`SkipFilter`], must match the vectors)
//!
//! - The skip set is checked again for each Buffer, not once per block.
//! - The end marker of a block is always emitted, exactly once, also when the
//!   block is skipped.
//! - A skipped block drops its children: their start and end markers too.
//! - When a parent is skipped while a child is open, the shallowest skipped open
//!   block wins.
//! - Start markers that come before the skip request are emitted.

use crate::buffer::Buffer;
use crate::hash::HashAlgorithm;

/// The wire form of a skip request (`control.skip_buffer`).
pub fn skip_buffer(block_id: &[u8], algorithm: HashAlgorithm) -> Buffer {
    let _ = (block_id, algorithm);
    todo!("step 5: skip")
}

/// The block id of a skip request in this algorithm, if `buffer` has one
/// (`control.skipped_block_id`).
pub fn skipped_block_id(buffer: &Buffer, algorithm: HashAlgorithm) -> Option<Vec<u8>> {
    let _ = (buffer, algorithm);
    todo!("step 5: skip")
}

/// The skip state of one call, shared by its parse side and its send side
/// (`control.StreamControl`). Clone it to share it. It must be `Send + Sync`.
#[derive(Clone, Debug)]
pub struct StreamControl {
    // Add the state you need (for example an Arc with a mutex).
    _private: (),
}

impl StreamControl {
    /// A new control, enabled, for block ids in `algorithm`.
    pub fn new(algorithm: HashAlgorithm) -> Self {
        let _ = algorithm;
        todo!("step 5: skip")
    }

    /// Receiver side: ask the peer to skip `block_id`. Queues one skip Buffer.
    /// A second request for the same id queues nothing.
    pub fn request_skip(&self, block_id: &[u8]) {
        let _ = block_id;
        todo!("step 5: skip")
    }

    /// Take all queued skip Buffers, oldest first.
    pub fn take_outbound(&self) -> Vec<Buffer> {
        todo!("step 5: skip")
    }

    /// Record a skip request from the peer, if `buffer` has one. True if it had.
    pub fn observe(&self, buffer: &Buffer) -> bool {
        let _ = buffer;
        todo!("step 5: skip")
    }

    /// Sender side: true if the peer asked to skip `block_id`.
    pub fn should_skip(&self, block_id: &[u8]) -> bool {
        let _ = block_id;
        todo!("step 5: skip")
    }

    /// The ids that the peer asked to skip, sorted.
    pub fn peer_skips(&self) -> Vec<Vec<u8>> {
        todo!("step 5: skip")
    }
}

/// Receiver side of an incoming stream (`StreamControl.reader`): record the skip
/// request of `buffer` in `control`, then drop the Buffer if it carries nothing
/// else, or return it with the `skip` field cleared.
pub fn strip_skip(control: &StreamControl, buffer: Buffer) -> Option<Buffer> {
    let _ = (control, buffer);
    todo!("step 5: skip")
}

/// Sender side: drops the body of blocks that the peer asked to skip
/// (`client.skip_requested_blocks`). See the rules in the module docs.
pub struct SkipFilter {
    // Add the state you need.
    _private: (),
}

impl SkipFilter {
    /// A new filter that reads the skip set of `control`.
    pub fn new(control: StreamControl) -> Self {
        let _ = control;
        todo!("step 5: skip")
    }

    /// Give the next outgoing Buffer. Returns it if it must be sent.
    pub fn push(&mut self, buffer: Buffer) -> Option<Buffer> {
        let _ = buffer;
        todo!("step 5: skip")
    }
}
