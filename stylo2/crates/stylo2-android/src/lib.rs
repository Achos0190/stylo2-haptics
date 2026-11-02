//! Android glue: the names, the backend-selection policy, and the fallback.
//!
//! This crate holds the Android-specific *constants and decisions* that are
//! useful to express (and test) in Rust, while the parts that must run inside
//! the JVM (binding services, checking permissions, driving the BLE radio)
//! live JVM-side and are reached through the transport seams in `stylo2-oplus`
//! and `stylo2-ble`.

#![forbid(unsafe_code)]

use stylo2_core::{
    BackendId, Capabilities, ConnectionState, EventCallback, HapticBackend, StyloError,
    VibrationProfile,
};

/// Android permission strings this project touches.
pub mod permissions {
    /// Restricted vendor permission guarding the accessory service. On stock
    /// firmware this is **not** grantable to an ordinary third-party app
    /// (reported/inferred protection level: at least `privileged`). Verify on
    /// device with `adb shell dumpsys package com.oplus.ipemanager | grep -i
    /// permission` and the framework `*-permissions-*.xml`.
    pub const OPLUS_IOT: &str = "com.oplus.permission.safe.IOT";
    /// Needed to talk to a bonded BLE device (Android 12+).
    pub const BLUETOOTH_CONNECT: &str = "android.permission.BLUETOOTH_CONNECT";
    /// Needed to scan for BLE devices (Android 12+).
    pub const BLUETOOTH_SCAN: &str = "android.permission.BLUETOOTH_SCAN";
}

/// Identifiers of the OnePlus accessory service, as observed by the existing
/// implementation. Treated as *reported* until re-confirmed on the target
/// firmware.
pub mod service {
    /// Package that hosts the accessory service.
    pub const PACKAGE: &str = "com.oplus.ipemanager";
    /// Intent action used to bind the Pencil SDK service.
    pub const ACTION_PENCIL_SDK: &str = "com.oplus.ipemanager.ACTION.PENCIL_SDK";
    /// Fully-qualified core service component.
    pub const CORE_SERVICE: &str = "com.oplus.ipemanager/.btadsorb.CoreService";
    /// Expected AIDL interface descriptor; confirm before trusting the binder.
    pub const SDK_DESCRIPTOR: &str = "com.oplus.ipemanager.sdk.ISdkAidlInterface";
}

/// Which concrete backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    /// Accessory-service binder route.
    OplusIpe,
    /// Direct BLE route.
    Ble,
    /// Tablet-vibrator fallback (not the pen).
    AndroidVibrator,
}

/// What a probe (run JVM-side) found to be actually available on this device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Availability {
    /// The accessory service binds *and* the vendor accepted this app (the
    /// permission was granted and the interface descriptor matched).
    pub oplus_authorized: bool,
    /// A direct-BLE haptic protocol has been verified for this device.
    pub ble_protocol_verified: bool,
    /// The tablet has a usable vibrator motor.
    pub tablet_vibrator: bool,
}

/// Choose the preferred backend, most-capable first, returning the choice and a
/// short human-readable reason. Returns `None` when nothing is available.
///
/// Order reflects fidelity to the pen's own hardware: accessory service →
/// direct BLE → tablet motor. The fallback is deliberately last and clearly
/// labelled, because it does not drive the pen.
pub fn preferred_backend(available: Availability) -> Option<(BackendKind, &'static str)> {
    if available.oplus_authorized {
        Some((
            BackendKind::OplusIpe,
            "authorised OnePlus accessory-service binder (drives the pen motor)",
        ))
    } else if available.ble_protocol_verified {
        Some((
            BackendKind::Ble,
            "verified direct BLE protocol (drives the pen motor)",
        ))
    } else if available.tablet_vibrator {
        Some((
            BackendKind::AndroidVibrator,
            "fallback: tablet vibrator motor — NOT the pen",
        ))
    } else {
        None
    }
}

/// The JVM operations the tablet-vibrator fallback needs.
pub trait VibratorTransport: Send {
    /// Whether the tablet exposes a usable vibrator.
    fn available(&self) -> bool;
    /// Start a (repeating) vibration.
    fn start(&mut self) -> Result<(), StyloError>;
    /// Stop vibration.
    fn stop(&mut self) -> Result<(), StyloError>;
}

/// A vibrator transport for hosts with no motor.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoVibrator;

impl VibratorTransport for NoVibrator {
    fn available(&self) -> bool {
        false
    }
    fn start(&mut self) -> Result<(), StyloError> {
        Err(StyloError::ServiceUnavailable)
    }
    fn stop(&mut self) -> Result<(), StyloError> {
        Ok(())
    }
}

/// The clearly-labelled fallback backend. It buzzes the **tablet**, not the
/// pen, and therefore advertises **no profiles** — only start/stop.
pub struct AndroidVibratorBackend<T: VibratorTransport> {
    transport: T,
    event_callback: Option<EventCallback>,
}

impl<T: VibratorTransport> AndroidVibratorBackend<T> {
    /// Build the fallback over a vibrator transport.
    pub fn new(transport: T) -> Self {
        AndroidVibratorBackend {
            transport,
            event_callback: None,
        }
    }
}

impl AndroidVibratorBackend<NoVibrator> {
    /// The honest default on a host with no motor.
    pub fn unavailable() -> Self {
        AndroidVibratorBackend::new(NoVibrator)
    }
}

impl<T: VibratorTransport> HapticBackend for AndroidVibratorBackend<T> {
    fn id(&self) -> BackendId {
        BackendId::AndroidVibrator
    }

    fn connection_state(&self) -> ConnectionState {
        if self.transport.available() {
            ConnectionState::Connected
        } else {
            ConnectionState::ServiceUnavailable
        }
    }

    fn capabilities(&self) -> Capabilities {
        if self.transport.available() {
            // No pen profiles: it is the tablet motor. Only start/stop.
            Capabilities {
                start_stop: true,
                ..Capabilities::none()
            }
        } else {
            Capabilities::none()
        }
    }

    fn connect(&mut self) -> Result<(), StyloError> {
        if self.transport.available() {
            Ok(())
        } else {
            Err(StyloError::ServiceUnavailable)
        }
    }

    fn disconnect(&mut self) -> Result<(), StyloError> {
        self.transport.stop()
    }

    fn set_profile(&mut self, _profile: VibrationProfile) -> Result<(), StyloError> {
        Err(StyloError::Unsupported(
            "the tablet vibrator has no pen profiles; it buzzes the tablet, not the pen"
                .to_string(),
        ))
    }

    fn start_feedback(&mut self) -> Result<(), StyloError> {
        self.transport.start()
    }

    fn stop_feedback(&mut self) -> Result<(), StyloError> {
        self.transport.stop()
    }

    fn set_event_callback(&mut self, callback: Option<EventCallback>) {
        self.event_callback = callback;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_prefers_pen_routes_over_fallback() {
        assert_eq!(
            preferred_backend(Availability {
                oplus_authorized: true,
                ble_protocol_verified: true,
                tablet_vibrator: true,
            })
            .map(|(k, _)| k),
            Some(BackendKind::OplusIpe)
        );

        assert_eq!(
            preferred_backend(Availability {
                ble_protocol_verified: true,
                tablet_vibrator: true,
                ..Availability::default()
            })
            .map(|(k, _)| k),
            Some(BackendKind::Ble)
        );

        assert_eq!(
            preferred_backend(Availability {
                tablet_vibrator: true,
                ..Availability::default()
            })
            .map(|(k, _)| k),
            Some(BackendKind::AndroidVibrator)
        );

        assert_eq!(preferred_backend(Availability::default()), None);
    }

    #[test]
    fn vibrator_fallback_has_no_profiles() {
        let mut backend = AndroidVibratorBackend::unavailable();
        assert!(backend.capabilities().profiles.is_empty());
        match backend.set_profile(VibrationProfile::Pencil) {
            Err(StyloError::Unsupported(_)) => {}
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }
}
