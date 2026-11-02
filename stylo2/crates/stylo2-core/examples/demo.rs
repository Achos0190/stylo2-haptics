//! End-to-end demo of the public API against the in-memory mock backend.
//!
//! Run with:
//! ```text
//! cargo run -p stylo2-core --example demo --features mock
//! ```
//!
//! It demonstrates the behaviour contract without a device: guarding before
//! connect, capability-aware profile selection, start/stop with de-duplication,
//! and honest error reporting.

use std::sync::{Arc, Mutex};

use stylo2_core::mock::MockBackend;
use stylo2_core::{
    ConnectionState, HapticManager, StylusEvent, StylusHaptics, VibrationProfile,
};

fn main() {
    let backend = MockBackend::new();
    let handle = backend.handle();
    let haptics = HapticManager::new(Box::new(backend));

    // Observe events.
    let log = Arc::new(Mutex::new(Vec::<StylusEvent>::new()));
    let sink = log.clone();
    haptics.set_event_listener(Some(Arc::new(move |e| sink.lock().unwrap().push(e))));

    // Before connecting, commands are refused with a precise reason.
    println!("state: {:?}", haptics.connection_state());
    println!(
        "start before connect -> {:?}",
        haptics.start_feedback() // Err(NotConnected)
    );

    // Connect (the mock simulates a successful bind).
    haptics.connect().expect("connect");
    assert_eq!(haptics.connection_state(), ConnectionState::Connected);
    println!("connected; capabilities: {:?}", haptics.capabilities());

    // Pick a profile and start feedback.
    haptics.set_profile(VibrationProfile::Pencil).expect("set profile");
    haptics.start_feedback().expect("start");
    println!("feedback active: {}", haptics.is_feedback_active());

    // Duplicate start is a no-op; the device sees exactly one start command.
    haptics.start_feedback().expect("start again (de-duped)");

    // A disabled-vibration backend error (simulated) is surfaced, never masked.
    haptics.stop_feedback().expect("stop");
    handle.fail_next(stylo2_core::StyloError::VibrationDisabled);
    println!("start with vibration off -> {:?}", haptics.start_feedback());

    // Simulate a device event and show it reached the listener.
    handle.emit(StylusEvent::VibrationSwitchChanged(false));
    haptics.disconnect().expect("disconnect");

    println!("events observed: {:?}", log.lock().unwrap());
}
