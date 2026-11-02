//! `stylo2-core` — portable, backend-agnostic model for controlling the
//! advanced haptic feedback of the OnePlus Stylo 2 stylus.
//!
//! # Scope and honesty
//!
//! This crate contains **no transport and no platform code**. It does not open
//! Bluetooth connections, bind Android services, or send any packets. It only
//! models:
//!
//! * the public API a third-party app programs against ([`StylusHaptics`]),
//! * the data types (profiles, states, capabilities, events, errors),
//! * the [`HapticBackend`] abstraction every transport implements, and
//! * a [`HapticManager`] facade that enforces correct command sequencing
//!   (de-duplication, capability gating, precise error reporting).
//!
//! The actual transports live in sibling crates (`stylo2-oplus`,
//! `stylo2-ble`, `stylo2-android`). This separation is deliberate: the
//! *behaviour contract* in this crate is fully verifiable on any host with
//! `cargo test`, independent of whether a given transport is reachable on a
//! particular device.
//!
//! ## Two rules this crate enforces by construction
//!
//! 1. **Never report an unauthorised or failed call as success.** If the active
//!    backend is not connected, is permission-denied, or a command fails, the
//!    facade returns the precise [`StyloError`]; it never silently returns
//!    `Ok(())`.
//! 2. **Never expose a control the hardware interface does not support.** The
//!    facade gates every command on the backend's reported [`Capabilities`].
//!    The reported OnePlus accessory interface exposes *profile selection +
//!    start/stop only* — not intensity, duration, frequency or waveform — so
//!    those controls are not offered unless a backend proves them on hardware.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod backend;
pub mod capability;
pub mod error;
pub mod event;
pub mod manager;
pub mod profile;
pub mod state;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

#[cfg(test)]
mod tests;

pub use backend::{BackendId, HapticBackend};
pub use capability::{Capabilities, EventCapabilities};
pub use error::StyloError;
pub use event::{EventCallback, StylusEvent};
pub use manager::{HapticManager, StylusHaptics};
pub use profile::VibrationProfile;
pub use state::ConnectionState;
