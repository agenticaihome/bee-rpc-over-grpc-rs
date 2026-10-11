//! Step 3: the flow control signal.
use std::time::Duration;

use bee_rpc::signal::Signal;

#[test]
fn new_signal_is_open() {
    assert!(Signal::new().is_open());
}

#[test]
fn change_toggles() {
    let s = Signal::new();
    s.change();
    assert!(!s.is_open());
    s.change();
    assert!(s.is_open());
}

#[test]
fn clones_share_the_state() {
    let s = Signal::new();
    let t = s.clone();
    t.change();
    assert!(!s.is_open());
}

#[test]
fn signal_is_send_and_sync() {
    fn check<T: Send + Sync + 'static>() {}
    check::<Signal>();
}

#[tokio::test]
async fn wait_open_returns_at_once_when_open() {
    tokio::time::timeout(Duration::from_millis(100), Signal::new().wait_open())
        .await
        .expect("wait_open blocked on an open signal");
}

#[tokio::test]
async fn wait_open_waits_until_change() {
    let s = Signal::new();
    s.change();
    let waiter = {
        let s = s.clone();
        tokio::spawn(async move { s.wait_open().await })
    };
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!waiter.is_finished(), "wait_open returned while closed");
    s.change();
    tokio::time::timeout(Duration::from_secs(2), waiter)
        .await
        .expect("wait_open did not wake up")
        .unwrap();
}
