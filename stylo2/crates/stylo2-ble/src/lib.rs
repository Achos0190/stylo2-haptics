//! Independent Bluetooth Low Energy transport — **scaffold only**.
//!
//! # Why this backend refuses to operate
//!
//! For a direct-BLE haptic route to exist, several things must be true, and
//! **none are verified** (see `stylo2/docs/C-protocol.md`):
//!
//! 1. the pen advertises or exposes reachable GATT services to a second
//!    (non-system) GATT client while it is bonded to the system;
//! 2. there is a *writable* characteristic that triggers the pen's haptic
//!    motor;
//! 3. the command encoding for that characteristic is known;
//! 4. a second connection does not disrupt the system's own connection.
//!
//! Until a [`Stylo2GattProfile`] is supplied with verified UUIDs and a verified
//! command encoding, [`BleBackend`] reports no capabilities and returns
//! [`StyloError::Unsupported`] for every command. This enforces the project
//! rule: *do not silently replace a missing capability with a simulated one.*
//!
//! The GATT I/O itself (scan, connect, discover, write, subscribe) is expressed
//! as the [`GattTransport`] seam. On Android it is implemented with the
//! platform BLE APIs JVM-side; on a desktop host a `btleplug`-based
//! implementation could be added behind a feature. This crate ships only the
//! honest [`NoGatt`] stub so it compiles and tests anywhere without claiming a
//! working radio.

#![forbid(unsafe_code)]

pub mod recon;

pub use recon::{
    candidate_control_characteristics, is_standard_sig_uuid, ControlCandidate,
    DiscoveredCharacteristic,
};

use stylo2_core::{
    BackendId, Capabilities, ConnectionState, EventCallback, HapticBackend, StyloError,
    VibrationProfile,
};

/// A 128-bit Bluetooth UUID.
pub type Uuid = [u8; 16];

/// A **verified** description of the Stylo 2's haptic GATT surface.
///
/// There is deliberately no default and no example value: constructing one
/// asserts that the UUIDs and the encoder were confirmed on hardware. The
/// encoder turns a profile + start/stop intent into the exact bytes to write.
pub struct Stylo2GattProfile {
    /// The service containing the haptic characteristic.
    pub service: Uuid,
    /// The writable characteristic that controls the pen motor.
    pub control_characteristic: Uuid,
    /// Encodes a command into the bytes to write to `control_characteristic`.
    pub encode: fn(BleCommand) -> Vec<u8>,
}

/// A command to encode for the control characteristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BleCommand {
    /// Select a profile.
    SetProfile(VibrationProfile),
    /// Start feedback.
    Start,
    /// Stop feedback.
    Stop,
}

/// Low-level GATT operations a platform must provide.
pub trait GattTransport: Send {
    /// Current link state.
    fn connection_state(&self) -> ConnectionState;
    /// Scan for, and connect to, the pen; discover services.
    fn connect(&mut self) -> Result<(), StyloError>;
    /// Disconnect the GATT link.
    fn disconnect(&mut self) -> Result<(), StyloError>;
    /// Write bytes to a characteristic.
    fn write(&mut self, service: &Uuid, characteristic: &Uuid, value: &[u8])
        -> Result<(), StyloError>;
}

/// A [`GattTransport`] with no radio. Honestly reports BLE as unavailable.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoGatt;

impl GattTransport for NoGatt {
    fn connection_state(&self) -> ConnectionState {
        ConnectionState::ServiceUnavailable
    }
    fn connect(&mut self) -> Result<(), StyloError> {
        Err(StyloError::ServiceUnavailable)
    }
    fn disconnect(&mut self) -> Result<(), StyloError> {
        Ok(())
    }
    fn write(&mut self, _s: &Uuid, _c: &Uuid, _v: &[u8]) -> Result<(), StyloError> {
        Err(StyloError::NotConnected)
    }
}

/// A [`HapticBackend`] over direct BLE. Inert without a verified
/// [`Stylo2GattProfile`].
pub struct BleBackend<T: GattTransport> {
    transport: T,
    profile_def: Option<Stylo2GattProfile>,
    selected: Option<VibrationProfile>,
    event_callback: Option<EventCallback>,
}

impl<T: GattTransport> BleBackend<T> {
    /// Build a backend. `profile_def` is `None` until the protocol is verified;
    /// while `None`, the backend advertises nothing and performs nothing.
    pub fn new(transport: T, profile_def: Option<Stylo2GattProfile>) -> Self {
        BleBackend {
            transport,
            profile_def,
            selected: None,
            event_callback: None,
        }
    }

    fn require_profile_def(&self) -> Result<&Stylo2GattProfile, StyloError> {
        self.profile_def.as_ref().ok_or_else(|| {
            StyloError::Unsupported(
                "Stylo 2 BLE haptic protocol is not verified on this build".to_string(),
            )
        })
    }
}

impl BleBackend<NoGatt> {
    /// The honest default: no radio, no verified protocol.
    pub fn unavailable() -> Self {
        BleBackend::new(NoGatt, None)
    }
}

impl<T: GattTransport> HapticBackend for BleBackend<T> {
    fn id(&self) -> BackendId {
        BackendId::Ble
    }

    fn connection_state(&self) -> ConnectionState {
        // Even a live GATT link is useless without a verified protocol, so the
        // backend is only "usable" when both are present.
        if self.profile_def.is_some() {
            self.transport.connection_state()
        } else {
            ConnectionState::ServiceUnavailable
        }
    }

    fn capabilities(&self) -> Capabilities {
        if self.profile_def.is_some() && self.transport.connection_state().is_usable() {
            // A verified BLE profile would advertise profile-select + start/stop.
            // Extended params stay false until proven.
            Capabilities::profile_switch_only()
        } else {
            Capabilities::none()
        }
    }

    fn connect(&mut self) -> Result<(), StyloError> {
        self.require_profile_def()?;
        self.transport.connect()
    }

    fn disconnect(&mut self) -> Result<(), StyloError> {
        self.transport.disconnect()
    }

    fn set_profile(&mut self, profile: VibrationProfile) -> Result<(), StyloError> {
        let def = self.require_profile_def()?;
        let bytes = (def.encode)(BleCommand::SetProfile(profile));
        let (service, ch) = (def.service, def.control_characteristic);
        self.transport.write(&service, &ch, &bytes)?;
        self.selected = Some(profile);
        Ok(())
    }

    fn start_feedback(&mut self) -> Result<(), StyloError> {
        let def = self.require_profile_def()?;
        let bytes = (def.encode)(BleCommand::Start);
        let (service, ch) = (def.service, def.control_characteristic);
        self.transport.write(&service, &ch, &bytes)
    }

    fn stop_feedback(&mut self) -> Result<(), StyloError> {
        let def = self.require_profile_def()?;
        let bytes = (def.encode)(BleCommand::Stop);
        let (service, ch) = (def.service, def.control_characteristic);
        self.transport.write(&service, &ch, &bytes)
    }

    fn set_event_callback(&mut self, callback: Option<EventCallback>) {
        self.event_callback = callback;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inert_without_verified_protocol() {
        let mut backend = BleBackend::unavailable();
        assert_eq!(
            backend.connection_state(),
            ConnectionState::ServiceUnavailable
        );
        assert_eq!(backend.capabilities(), Capabilities::none());
        match backend.set_profile(VibrationProfile::Pencil) {
            Err(StyloError::Unsupported(_)) => {}
            other => panic!("expected Unsupported, got {other:?}"),
        }
        match backend.start_feedback() {
            Err(StyloError::Unsupported(_)) => {}
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }
}
