//! Flow control signal (`utils.Signal`).
//!
//! A Buffer with `signal: true` from the peer toggles the state: the first one
//! closes (stop sending), the next one opens again. The sender waits while the
//! state is closed.

use std::future::poll_fn;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Poll, Waker};

/// The flow control state shared by the parser (which toggles it) and the
/// serializer (which waits on it). Clone it to share it.
#[derive(Clone, Debug)]
pub struct Signal {
    state: Arc<Mutex<State>>,
}

#[derive(Debug)]
struct State {
    open: bool,
    /// Counts the changes from closed to open. A waiter that saw the signal
    /// closed returns when this moves, even if the signal closed again before it
    /// ran (as `Condition.notify_all` wakes the waiters of the reference).
    opened: u64,
    /// The next id for a waiter.
    next_id: u64,
    /// The waiters, by id. A waiter replaces its waker at each poll and removes
    /// it when it is dropped.
    waiters: Vec<(u64, Waker)>,
}

impl Signal {
    /// A new signal, open.
    pub fn new() -> Self {
        Signal {
            state: Arc::new(Mutex::new(State {
                open: true,
                opened: 0,
                next_id: 0,
                waiters: Vec::new(),
            })),
        }
    }

    /// Toggle the state (`Signal.change`).
    pub fn change(&self) {
        let wake = {
            let mut state = self.lock();
            if state.open {
                state.open = false;
                Vec::new()
            } else {
                state.open = true;
                state.opened = state.opened.wrapping_add(1);
                std::mem::take(&mut state.waiters)
            }
        };
        for (_, waker) in wake {
            waker.wake();
        }
    }

    /// True when the sender can send.
    pub fn is_open(&self) -> bool {
        self.lock().open
    }

    /// Wait until the state is open. Returns at once when it is open.
    pub async fn wait_open(&self) {
        let mut waiter = Waiter {
            signal: self,
            seen: None,
        };
        poll_fn(|cx| {
            let mut state = waiter.signal.lock();
            if state.open {
                return Poll::Ready(());
            }
            match waiter.seen {
                Some((_, opened)) if opened != state.opened => Poll::Ready(()),
                Some((id, _)) => {
                    match state.waiters.iter_mut().find(|(i, _)| *i == id) {
                        Some((_, waker)) => waker.clone_from(cx.waker()),
                        None => state.waiters.push((id, cx.waker().clone())),
                    }
                    Poll::Pending
                }
                None => {
                    let id = state.next_id;
                    state.next_id = state.next_id.wrapping_add(1);
                    waiter.seen = Some((id, state.opened));
                    state.waiters.push((id, cx.waker().clone()));
                    Poll::Pending
                }
            }
        })
        .await
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        // The state is a bool and counters: a panic in another thread cannot
        // leave it half changed.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for Signal {
    fn default() -> Self {
        Self::new()
    }
}

/// A pending [`Signal::wait_open`]. Removes its waker when the wait ends or is
/// cancelled.
struct Waiter<'a> {
    signal: &'a Signal,
    /// The id of the waiter and the value of `opened` when it first saw the
    /// signal closed.
    seen: Option<(u64, u64)>,
}

impl Drop for Waiter<'_> {
    fn drop(&mut self) {
        if let Some((id, _)) = self.seen {
            self.signal.lock().waiters.retain(|(i, _)| *i != id);
        }
    }
}
