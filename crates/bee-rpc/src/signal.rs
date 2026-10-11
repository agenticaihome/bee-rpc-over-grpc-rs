//! Flow control signal (`utils.Signal`).
//!
//! A Buffer with `signal: true` from the peer toggles the state: the first one
//! closes (stop sending), the next one opens again. The sender waits while the
//! state is closed.

/// The flow control state shared by the parser (which toggles it) and the
/// serializer (which waits on it). Clone it to share it.
#[derive(Clone, Debug)]
pub struct Signal {
    // Add the state you need (for example an Arc with a mutex and a notifier).
    _private: (),
}

impl Signal {
    /// A new signal, open.
    pub fn new() -> Self {
        todo!("step 3: signal")
    }

    /// Toggle the state (`Signal.change`).
    pub fn change(&self) {
        todo!("step 3: signal")
    }

    /// True when the sender can send.
    pub fn is_open(&self) -> bool {
        todo!("step 3: signal")
    }

    /// Wait until the state is open. Returns at once when it is open.
    pub async fn wait_open(&self) {
        todo!("step 3: signal")
    }
}

impl Default for Signal {
    fn default() -> Self {
        Self::new()
    }
}
