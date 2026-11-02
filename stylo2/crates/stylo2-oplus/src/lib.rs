//! OnePlus accessory-service (`com.oplus.ipemanager`) binder adapter.
//!
//! # What this crate can and cannot do
//!
//! The OnePlus route is an **Android binder/AIDL** interaction with the system
//! service `com.oplus.ipemanager/.btadsorb.CoreService`. That interaction can
//! only happen inside the Android runtime (the JVM), so it is **not** performed
//! in Rust. The division of labour is:
//!
//! * The JVM side (Kotlin, see `android/stylo2-bridge`) binds the service,
//!   confirms the interface descriptor, and makes the AIDL calls. It is the
//!   only place the restricted permission `com.oplus.permission.safe.IOT`
//!   matters.
//! * This crate provides [`OplusIpeBackend`], which implements the core
//!   [`HapticBackend`] trait by translating profiles into the service's
//!   integer argument and forwarding each call across a [`BinderTransport`]
//!   seam to that JVM side.
//!
//! On any non-Android host there is no binder, so the default
//! [`UnavailableBinder`] transport honestly reports
//! [`ConnectionState::ServiceUnavailable`]. This lets the crate compile and be
//! unit-tested anywhere without ever pretending the route works.
//!
//! [`HapticBackend`]: stylo2_core::HapticBackend
//! [`ConnectionState::ServiceUnavailable`]: stylo2_core::ConnectionState::ServiceUnavailable

#![forbid(unsafe_code)]

use stylo2_core::{
    BackendId, Capabilities, ConnectionState, EventCallback, HapticBackend, StyloError,
    VibrationProfile,
};

/// Reported, **unverified** wire values for the vendor `setVibrationType(int)`
/// call.
///
/// # Provenance and status: REPORTED / UNVERIFIED
///
/// The existing community implementation passes the ordinal of a vendor
/// `Vibration` enum (`PENCIL, ERASER, BALLPEN, PEN, FUNCTION_VIBRATION`) to
/// `setVibrationType(int)`. These values are therefore *inferred from
/// declaration order*, which the project specification explicitly warns against
/// trusting. They are isolated here so that, once confirmed against the actual
/// service on the target firmware, they can be corrected in exactly one place.
///
/// Do not scatter these integers through the codebase.
pub mod wire {
    use super::VibrationProfile;

    /// The reported integer argument for a profile. **Verify on hardware.**
    pub fn reported_code(profile: VibrationProfile) -> i32 {
        match profile {
            VibrationProfile::Pencil => 0,
            VibrationProfile::Eraser => 1,
            VibrationProfile::Ballpoint => 2, // vendor enum: BALLPEN
            VibrationProfile::Pen => 3,
        }
    }

    /// The vendor enum also contains `FUNCTION_VIBRATION` (reported ordinal 4),
    /// which is a feedback mode rather than a writing-feel profile and is not
    /// exposed as a [`VibrationProfile`]. Documented here for completeness.
    pub const REPORTED_FUNCTION_VIBRATION_CODE: i32 = 4;
}

/// The operations the JVM/binder side must provide. Mirrors the subset of the
/// reported `ISdkAidlInterface` this library uses.
///
/// Implementations live JVM-side and are reached over JNI. A pure-Rust
/// implementation ([`UnavailableBinder`]) is provided for hosts without a
/// binder.
pub trait BinderTransport: Send {
    /// Current connection state of the bound service / pen.
    fn connection_state(&self) -> ConnectionState;
    /// Bind the service and confirm the interface descriptor.
    fn connect(&mut self) -> Result<(), StyloError>;
    /// Unbind the service.
    fn disconnect(&mut self) -> Result<(), StyloError>;
    /// `setVibrationType(int)`.
    fn set_vibration_type(&mut self, code: i32) -> Result<(), StyloError>;
    /// `startFeedBackVibration()`.
    fn start_feedback(&mut self) -> Result<(), StyloError>;
    /// `stopFeedBackVibration()`.
    fn stop_feedback(&mut self) -> Result<(), StyloError>;
}

/// A [`BinderTransport`] for environments with no Android binder. Every call
/// honestly reports the route as unavailable.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnavailableBinder;

impl BinderTransport for UnavailableBinder {
    fn connection_state(&self) -> ConnectionState {
        ConnectionState::ServiceUnavailable
    }
    fn connect(&mut self) -> Result<(), StyloError> {
        Err(StyloError::ServiceUnavailable)
    }
    fn disconnect(&mut self) -> Result<(), StyloError> {
        Ok(())
    }
    fn set_vibration_type(&mut self, _code: i32) -> Result<(), StyloError> {
        Err(StyloError::NotConnected)
    }
    fn start_feedback(&mut self) -> Result<(), StyloError> {
        Err(StyloError::NotConnected)
    }
    fn stop_feedback(&mut self) -> Result<(), StyloError> {
        Err(StyloError::NotConnected)
    }
}

/// A [`HapticBackend`] backed by the OnePlus accessory service over a
/// [`BinderTransport`].
pub struct OplusIpeBackend<T: BinderTransport> {
    transport: T,
    event_callback: Option<EventCallback>,
}

impl<T: BinderTransport> OplusIpeBackend<T> {
    /// Build the backend over a transport.
    pub fn new(transport: T) -> Self {
        OplusIpeBackend {
            transport,
            event_callback: None,
        }
    }

    /// Borrow the registered event listener, so a JVM-side transport can push
    /// callback events through it.
    pub fn event_callback(&self) -> Option<&EventCallback> {
        self.event_callback.as_ref()
    }
}

impl OplusIpeBackend<UnavailableBinder> {
    /// The honest default on a host with no binder.
    pub fn unavailable() -> Self {
        OplusIpeBackend::new(UnavailableBinder)
    }
}

impl<T: BinderTransport> HapticBackend for OplusIpeBackend<T> {
    fn id(&self) -> BackendId {
        BackendId::OplusIpe
    }

    fn connection_state(&self) -> ConnectionState {
        self.transport.connection_state()
    }

    fn capabilities(&self) -> Capabilities {
        // Only advertise capabilities when actually connected. The reported
        // interface is profile-select + start/stop + feedback-layer events.
        if self.transport.connection_state().is_usable() {
            Capabilities::profile_switch_only()
        } else {
            Capabilities::none()
        }
    }

    fn connect(&mut self) -> Result<(), StyloError> {
        self.transport.connect()
    }

    fn disconnect(&mut self) -> Result<(), StyloError> {
        self.transport.disconnect()
    }

    fn set_profile(&mut self, profile: VibrationProfile) -> Result<(), StyloError> {
        self.transport.set_vibration_type(wire::reported_code(profile))
    }

    fn start_feedback(&mut self) -> Result<(), StyloError> {
        self.transport.start_feedback()
    }

    fn stop_feedback(&mut self) -> Result<(), StyloError> {
        self.transport.stop_feedback()
    }

    fn set_event_callback(&mut self, callback: Option<EventCallback>) {
        self.event_callback = callback;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reported_codes_match_community_ordinals() {
        assert_eq!(wire::reported_code(VibrationProfile::Pencil), 0);
        assert_eq!(wire::reported_code(VibrationProfile::Eraser), 1);
        assert_eq!(wire::reported_code(VibrationProfile::Ballpoint), 2);
        assert_eq!(wire::reported_code(VibrationProfile::Pen), 3);
    }

    #[test]
    fn unavailable_backend_reports_service_unavailable_and_no_caps() {
        let mut backend = OplusIpeBackend::unavailable();
        assert_eq!(
            backend.connection_state(),
            ConnectionState::ServiceUnavailable
        );
        assert_eq!(backend.capabilities(), Capabilities::none());
        assert_eq!(backend.connect(), Err(StyloError::ServiceUnavailable));
        // Never fakes success.
        assert_eq!(
            backend.set_profile(VibrationProfile::Pencil),
            Err(StyloError::NotConnected)
        );
    }
}
