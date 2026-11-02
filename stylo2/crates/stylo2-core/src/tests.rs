//! Behaviour tests for [`HapticManager`]. These exercise the policy the
//! library promises: guarding, capability gating, de-duplication, and honest
//! error reporting — all without a device.

use std::sync::{Arc, Mutex};

use crate::capability::{Capabilities, EventCapabilities};
use crate::error::StyloError;
use crate::event::StylusEvent;
use crate::manager::{HapticManager, StylusHaptics};
use crate::mock::{MockBackend, MockCall, MockHandle};
use crate::profile::VibrationProfile;
use crate::state::ConnectionState;

/// Build a manager plus a handle to its mock backend.
fn manager() -> (HapticManager, MockHandle) {
    let backend = MockBackend::new();
    let handle = backend.handle();
    (HapticManager::new(Box::new(backend)), handle)
}

/// Build a manager whose backend is already connected.
fn connected_manager() -> (HapticManager, MockHandle) {
    let (mgr, handle) = manager();
    handle.set_state(ConnectionState::Connected);
    (mgr, handle)
}

#[test]
fn commands_require_a_connection() {
    let (mgr, _h) = manager(); // starts Disconnected
    assert_eq!(
        mgr.set_profile(VibrationProfile::Pencil),
        Err(StyloError::NotConnected)
    );
    assert_eq!(mgr.start_feedback(), Err(StyloError::NotConnected));
}

#[test]
fn permission_denied_is_surfaced_precisely() {
    let (mgr, h) = manager();
    h.set_state(ConnectionState::PermissionDenied);

    assert_eq!(mgr.connect(), Err(StyloError::PermissionDenied));
    assert_eq!(
        mgr.set_profile(VibrationProfile::Pencil),
        Err(StyloError::PermissionDenied)
    );
    // The guard runs before any command reaches the backend.
    assert!(!h.calls().contains(&MockCall::SetProfile(VibrationProfile::Pencil)));
}

#[test]
fn service_unavailable_is_surfaced_precisely() {
    let (mgr, h) = manager();
    h.set_state(ConnectionState::ServiceUnavailable);
    assert_eq!(mgr.connect(), Err(StyloError::ServiceUnavailable));
    assert_eq!(mgr.start_feedback(), Err(StyloError::ServiceUnavailable));
}

#[test]
fn selecting_the_same_profile_twice_sends_one_command() {
    let (mgr, h) = connected_manager();
    assert!(mgr.set_profile(VibrationProfile::Pencil).is_ok());
    assert!(mgr.set_profile(VibrationProfile::Pencil).is_ok()); // de-duped
    assert_eq!(h.count(&MockCall::SetProfile(VibrationProfile::Pencil)), 1);

    // A genuine change still goes through.
    assert!(mgr.set_profile(VibrationProfile::Pen).is_ok());
    assert_eq!(h.count(&MockCall::SetProfile(VibrationProfile::Pen)), 1);
}

#[test]
fn start_requires_a_selected_profile() {
    let (mgr, h) = connected_manager();
    match mgr.start_feedback() {
        Err(StyloError::InvalidRequest(_)) => {}
        other => panic!("expected InvalidRequest, got {other:?}"),
    }
    assert_eq!(h.count(&MockCall::StartFeedback), 0);
}

#[test]
fn double_start_sends_one_command() {
    let (mgr, h) = connected_manager();
    mgr.set_profile(VibrationProfile::Pencil).unwrap();
    assert!(mgr.start_feedback().is_ok());
    assert!(mgr.start_feedback().is_ok()); // de-duped
    assert_eq!(h.count(&MockCall::StartFeedback), 1);
    assert!(mgr.is_feedback_active());
}

#[test]
fn stop_when_idle_is_a_noop() {
    let (mgr, h) = connected_manager();
    assert!(mgr.stop_feedback().is_ok());
    assert_eq!(h.count(&MockCall::StopFeedback), 0);
    assert!(!mgr.is_feedback_active());
}

#[test]
fn unsupported_profile_is_rejected_and_not_sent() {
    let (mgr, h) = connected_manager();
    // Backend advertises only Pen.
    h.set_capabilities(Capabilities {
        profiles: vec![VibrationProfile::Pen],
        start_stop: true,
        ..Capabilities::none()
    });
    match mgr.set_profile(VibrationProfile::Pencil) {
        Err(StyloError::Unsupported(_)) => {}
        other => panic!("expected Unsupported, got {other:?}"),
    }
    assert_eq!(h.count(&MockCall::SetProfile(VibrationProfile::Pencil)), 0);
    // The advertised profile still works.
    assert!(mgr.set_profile(VibrationProfile::Pen).is_ok());
}

#[test]
fn start_stop_is_rejected_when_unsupported() {
    let (mgr, h) = connected_manager();
    h.set_capabilities(Capabilities {
        profiles: VibrationProfile::ALL.to_vec(),
        start_stop: false, // interface cannot start/stop
        ..Capabilities::none()
    });
    mgr.set_profile(VibrationProfile::Pencil).unwrap();
    match mgr.start_feedback() {
        Err(StyloError::Unsupported(_)) => {}
        other => panic!("expected Unsupported, got {other:?}"),
    }
    assert_eq!(h.count(&MockCall::StartFeedback), 0);
}

#[test]
fn a_failed_start_is_not_reported_as_active() {
    let (mgr, h) = connected_manager();
    mgr.set_profile(VibrationProfile::Pencil).unwrap();
    h.fail_next(StyloError::Transport("write failed".into()));
    match mgr.start_feedback() {
        Err(StyloError::Transport(_)) => {}
        other => panic!("expected Transport, got {other:?}"),
    }
    assert!(!mgr.is_feedback_active(), "failed start must not mark active");
}

#[test]
fn an_unauthorized_call_is_never_reported_as_success() {
    let (mgr, h) = connected_manager();
    mgr.set_profile(VibrationProfile::Pencil).unwrap();
    h.fail_next(StyloError::Unauthorized);
    assert_eq!(mgr.start_feedback(), Err(StyloError::Unauthorized));
    assert!(!mgr.is_feedback_active());
}

#[test]
fn disconnect_stops_feedback_and_resets_state() {
    let (mgr, h) = connected_manager();
    mgr.set_profile(VibrationProfile::Pencil).unwrap();
    mgr.start_feedback().unwrap();
    assert!(mgr.is_feedback_active());

    mgr.disconnect().unwrap();
    assert!(!mgr.is_feedback_active());
    assert_eq!(mgr.current_profile(), None);

    let calls = h.calls();
    let stop = calls.iter().position(|c| *c == MockCall::StopFeedback);
    let disc = calls.iter().position(|c| *c == MockCall::Disconnect);
    assert!(stop.is_some() && disc.is_some() && stop < disc, "stop must precede disconnect");
}

#[test]
fn events_reach_a_registered_listener() {
    let (mgr, h) = connected_manager();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    mgr.set_event_listener(Some(Arc::new(move |e: StylusEvent| {
        sink.lock().unwrap().push(e);
    })));

    h.emit(StylusEvent::Connected);
    h.emit(StylusEvent::VibrationSwitchChanged(true));

    let got = seen.lock().unwrap().clone();
    assert_eq!(
        got,
        vec![
            StylusEvent::Connected,
            StylusEvent::VibrationSwitchChanged(true)
        ]
    );
}

#[test]
fn profile_parsing_is_case_insensitive_with_alias() {
    assert_eq!(
        VibrationProfile::from_str_ci("BallPen"),
        Some(VibrationProfile::Ballpoint)
    );
    assert_eq!(
        VibrationProfile::from_str_ci(" pencil "),
        Some(VibrationProfile::Pencil)
    );
    assert_eq!(VibrationProfile::from_str_ci("crayon"), None);
}

#[test]
fn default_event_capabilities_are_all_false() {
    let e = EventCapabilities::default();
    assert!(!e.connection && !e.button && !e.battery && !e.double_tap);
}
