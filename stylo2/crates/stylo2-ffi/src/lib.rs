//! A small, stable C ABI over [`stylo2_core`], plus optional JNI entry points.
//!
//! The Kotlin/Java bridge calls this surface. Keeping it tiny and stable is a
//! project goal: all policy lives in the core, and this layer only marshals
//! across the boundary.
//!
//! # ABI profile indices
//!
//! The `profile` integer in this ABI is stylo2's own stable profile index
//! (`0=Pencil, 1=Eraser, 2=Ballpoint, 3=Pen`), matching
//! [`VibrationProfile::ALL`]. It is **not** the vendor `setVibrationType(int)`
//! wire value — that mapping is isolated in `stylo2-oplus` and may differ.
//!
//! # Memory
//!
//! `stylo2_manager_new_*` returns an owning pointer; pass it to
//! [`stylo2_manager_free`] exactly once. All other functions borrow it and are
//! null-safe.

use std::os::raw::c_int;

use stylo2_core::{HapticManager, StyloError, StylusHaptics, VibrationProfile};

#[cfg(feature = "jni")]
mod jni_bridge;

/// Stable status codes returned across the ABI.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stylo2Status {
    /// Success.
    Ok = 0,
    /// Not connected to a backend.
    NotConnected = 1,
    /// OS-level permission denied.
    PermissionDenied = 2,
    /// Service/peripheral unavailable.
    ServiceUnavailable = 3,
    /// Vendor refused this app's identity.
    Unauthorized = 4,
    /// Pen vibration switched off by the user.
    VibrationDisabled = 5,
    /// Control not supported by the active backend.
    Unsupported = 6,
    /// Invalid request / ordering.
    InvalidRequest = 7,
    /// Transport failure.
    Transport = 8,
    /// Other backend error.
    Backend = 9,
    /// A null handle was passed.
    NullHandle = 20,
    /// An argument was out of range (e.g. unknown profile index).
    BadArgument = 21,
}

impl From<&StyloError> for Stylo2Status {
    fn from(e: &StyloError) -> Self {
        match e {
            StyloError::NotConnected => Stylo2Status::NotConnected,
            StyloError::PermissionDenied => Stylo2Status::PermissionDenied,
            StyloError::ServiceUnavailable => Stylo2Status::ServiceUnavailable,
            StyloError::Unauthorized => Stylo2Status::Unauthorized,
            StyloError::VibrationDisabled => Stylo2Status::VibrationDisabled,
            StyloError::Unsupported(_) => Stylo2Status::Unsupported,
            StyloError::InvalidRequest(_) => Stylo2Status::InvalidRequest,
            StyloError::Transport(_) => Stylo2Status::Transport,
            StyloError::Backend(_) => Stylo2Status::Backend,
            // `StyloError` is `#[non_exhaustive]`; map any future variant to a
            // generic backend error so the ABI stays stable.
            _ => Stylo2Status::Backend,
        }
    }
}

fn status_of(result: Result<(), StyloError>) -> Stylo2Status {
    match result {
        Ok(()) => Stylo2Status::Ok,
        Err(e) => (&e).into(),
    }
}

/// Opaque handle wrapping a [`HapticManager`].
pub struct Stylo2Manager {
    inner: HapticManager,
}

/// Translate an ABI profile index into a [`VibrationProfile`].
fn profile_from_abi(index: c_int) -> Option<VibrationProfile> {
    VibrationProfile::ALL.get(usize::try_from(index).ok()?).copied()
}

/// Create a manager backed by the in-memory mock (for host tests and the C
/// example). Never returns null.
#[no_mangle]
pub extern "C" fn stylo2_manager_new_mock() -> *mut Stylo2Manager {
    let backend = Box::new(stylo2_core::mock::MockBackend::new());
    let mgr = Stylo2Manager {
        inner: HapticManager::new(backend),
    };
    Box::into_raw(Box::new(mgr))
}

/// Create a manager backed by the OnePlus binder route in its *unavailable*
/// state (no JVM transport wired). Honest default when the real transport is
/// not yet attached. Never returns null.
#[no_mangle]
pub extern "C" fn stylo2_manager_new_oplus_unavailable() -> *mut Stylo2Manager {
    let backend = Box::new(stylo2_oplus::OplusIpeBackend::unavailable());
    let mgr = Stylo2Manager {
        inner: HapticManager::new(backend),
    };
    Box::into_raw(Box::new(mgr))
}

/// Free a manager created by `stylo2_manager_new_*`.
///
/// # Safety
/// `ptr` must have come from a `stylo2_manager_new_*` call and must not be used
/// afterwards. Passing null is allowed and does nothing.
#[no_mangle]
pub unsafe extern "C" fn stylo2_manager_free(ptr: *mut Stylo2Manager) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// Borrow a manager pointer safely for the duration of `f`.
///
/// # Safety
/// `ptr` must be null or a valid pointer from `stylo2_manager_new_*`.
unsafe fn with_manager<F>(ptr: *mut Stylo2Manager, f: F) -> Stylo2Status
where
    F: FnOnce(&HapticManager) -> Stylo2Status,
{
    match ptr.as_ref() {
        Some(m) => f(&m.inner),
        None => Stylo2Status::NullHandle,
    }
}

/// `connect()`.
///
/// # Safety
/// See [`with_manager`].
#[no_mangle]
pub unsafe extern "C" fn stylo2_connect(ptr: *mut Stylo2Manager) -> Stylo2Status {
    with_manager(ptr, |m| status_of(m.connect()))
}

/// `disconnect()`.
///
/// # Safety
/// See [`with_manager`].
#[no_mangle]
pub unsafe extern "C" fn stylo2_disconnect(ptr: *mut Stylo2Manager) -> Stylo2Status {
    with_manager(ptr, |m| status_of(m.disconnect()))
}

/// Returns the [`stylo2_core::ConnectionState`] as an int
/// (`0=Disconnected,1=Connecting,2=Connected,3=PermissionDenied,
/// 4=ServiceUnavailable`), or `-1` for a null handle.
///
/// # Safety
/// `ptr` must be null or a valid manager pointer.
#[no_mangle]
pub unsafe extern "C" fn stylo2_connection_state(ptr: *mut Stylo2Manager) -> c_int {
    use stylo2_core::ConnectionState::*;
    match ptr.as_ref() {
        None => -1,
        Some(m) => match m.inner.connection_state() {
            Disconnected => 0,
            Connecting => 1,
            Connected => 2,
            PermissionDenied => 3,
            ServiceUnavailable => 4,
        },
    }
}

/// `set_profile(profile)` where `profile` is an ABI profile index (see module
/// docs).
///
/// # Safety
/// See [`with_manager`].
#[no_mangle]
pub unsafe extern "C" fn stylo2_set_profile(
    ptr: *mut Stylo2Manager,
    profile: c_int,
) -> Stylo2Status {
    let Some(p) = profile_from_abi(profile) else {
        return Stylo2Status::BadArgument;
    };
    with_manager(ptr, |m| status_of(m.set_profile(p)))
}

/// `start_feedback()`.
///
/// # Safety
/// See [`with_manager`].
#[no_mangle]
pub unsafe extern "C" fn stylo2_start_feedback(ptr: *mut Stylo2Manager) -> Stylo2Status {
    with_manager(ptr, |m| status_of(m.start_feedback()))
}

/// `stop_feedback()`.
///
/// # Safety
/// See [`with_manager`].
#[no_mangle]
pub unsafe extern "C" fn stylo2_stop_feedback(ptr: *mut Stylo2Manager) -> Stylo2Status {
    with_manager(ptr, |m| status_of(m.stop_feedback()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_handle_is_reported_not_crashed() {
        unsafe {
            assert_eq!(stylo2_connect(std::ptr::null_mut()), Stylo2Status::NullHandle);
            assert_eq!(stylo2_connection_state(std::ptr::null_mut()), -1);
            stylo2_manager_free(std::ptr::null_mut()); // no-op
        }
    }

    #[test]
    fn mock_round_trip_through_the_abi() {
        unsafe {
            let m = stylo2_manager_new_mock();
            assert_eq!(stylo2_connect(m), Stylo2Status::Ok);
            assert_eq!(stylo2_connection_state(m), 2); // Connected
            assert_eq!(stylo2_set_profile(m, 0), Stylo2Status::Ok); // Pencil
            assert_eq!(stylo2_start_feedback(m), Stylo2Status::Ok);
            assert_eq!(stylo2_start_feedback(m), Stylo2Status::Ok); // de-duped
            assert_eq!(stylo2_stop_feedback(m), Stylo2Status::Ok);
            assert_eq!(stylo2_set_profile(m, 99), Stylo2Status::BadArgument);
            stylo2_manager_free(m);
        }
    }

    #[test]
    fn oplus_unavailable_reports_service_unavailable() {
        unsafe {
            let m = stylo2_manager_new_oplus_unavailable();
            assert_eq!(stylo2_connect(m), Stylo2Status::ServiceUnavailable);
            assert_eq!(stylo2_connection_state(m), 4);
            stylo2_manager_free(m);
        }
    }
}
