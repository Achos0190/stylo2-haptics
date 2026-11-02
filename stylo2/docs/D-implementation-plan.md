# Deliverable D — Rust implementation plan

## 1. Workspace structure (implemented)

```
stylo2/crates/
  stylo2-core      portable model + policy (no deps, no I/O)
  stylo2-oplus     accessory-service binder adapter (+ reported wire mapping)
  stylo2-ble       direct-BLE transport scaffold (inert until verified)
  stylo2-android   permission/service names, backend selection, fallback
  stylo2-ffi       C ABI (+ optional JNI entry points)
android/stylo2-bridge   Kotlin integration layer
```

## 2. Crate responsibilities

- **stylo2-core** — The only crate an API consumer must understand. Owns
  `VibrationProfile`, `ConnectionState`, `Capabilities`, `StylusEvent`,
  `StyloError`, the `HapticBackend` trait, and `HapticManager` (the policy:
  guarding, capability gating, command de-duplication, honest error reporting).
  No transport, no platform, no third-party deps → testable anywhere.
- **stylo2-oplus** — Implements `HapticBackend` over a `BinderTransport` seam.
  Owns the single, labelled, reported profile→int mapping. The privileged binder
  work is JVM-side.
- **stylo2-ble** — Implements `HapticBackend` over a `GattTransport` seam. Inert
  until a verified `Stylo2GattProfile` is supplied; never simulates.
- **stylo2-android** — Permission/service string constants, `preferred_backend`
  selection policy, and the `AndroidVibratorBackend` fallback (tablet motor,
  no profiles).
- **stylo2-ffi** — Minimal, stable C ABI over the core; optional `jni` feature
  adds `Java_…_native*` entry points for the Kotlin bridge.

## 3. Public API

**Rust** (`stylo2_core`):

```rust
trait StylusHaptics {
    fn connection_state(&self) -> ConnectionState;
    fn capabilities(&self) -> Capabilities;
    fn set_profile(&self, profile: VibrationProfile) -> Result<(), StyloError>;
    fn start_feedback(&self) -> Result<(), StyloError>;
    fn stop_feedback(&self) -> Result<(), StyloError>;
}
```

`HapticManager::new(Box<dyn HapticBackend>)` implements it, plus `connect`,
`disconnect`, `set_event_listener`, `is_feedback_active`, `current_profile`.

**C ABI** (`stylo2-ffi`): `stylo2_manager_new_*`, `stylo2_connect`,
`stylo2_disconnect`, `stylo2_connection_state`, `stylo2_set_profile`,
`stylo2_start_feedback`, `stylo2_stop_feedback`, `stylo2_manager_free`, all
returning stable `Stylo2Status` codes. (ABI profile index = stylo2's own enum
order, **not** the vendor wire value.)

**Kotlin** (`android/stylo2-bridge`): `Stylo2Haptics` wrapping the native
handle, exposing suspend-free synchronous calls + a `StateFlow` of state/events,
mirroring the existing app's ergonomics.

## 4. Kotlin integration boundary

The JVM owns everything that cannot be done in Rust:

- **Binder transport** — a Kotlin `BinderTransport` implementation reusing the
  existing `StylusVibrationClient` logic (bind, descriptor check, AIDL calls).
  It is the Kotlin object the native `OplusIpeBackend` conceptually delegates
  to; in practice the Kotlin layer calls the native C ABI for *policy* and
  performs the AIDL calls itself, with the native layer tracking state. (Two
  valid wirings are described in `F-crate.md`; the simplest keeps the binder
  calls in Kotlin and uses the Rust core for state/guarding.)
- **BLE transport** — Kotlin over Android BLE APIs, feeding a verified
  `Stylo2GattProfile`.
- **Vibrator fallback** — Kotlin over `VibratorManager`.

The native surface is intentionally tiny so this boundary stays stable.

## 5. Android permissions

- `com.oplus.permission.safe.IOT` — declared; expected to be **denied** to an
  ordinary app (surfaced as `PermissionDenied`, not hidden).
- `BLUETOOTH_CONNECT`, `BLUETOOTH_SCAN` — runtime, requested with rationale, for
  the BLE route.
- No other sensitive permissions.

## 6. Error handling

One `StyloError` taxonomy, mapped 1:1 to `Stylo2Status` (FFI) and to a Kotlin
sealed type. The rule enforced in `HapticManager`: a non-connected state or a
backend failure yields the precise error; success is returned only when the
backend call actually succeeded.

## 7. Concurrency model

- `HapticManager` serialises commands behind a `Mutex`; `StylusHaptics` takes
  `&self`, so one `Arc<HapticManager>` is shared across threads and held for the
  process lifetime behind JNI.
- Event delivery is a `Arc<dyn Fn(StylusEvent) + Send + Sync>`; a backend worker
  thread (or the JVM binder callback thread) invokes it. The manager forwards a
  user listener to the backend.
- Lock poisoning is recovered (a panicked command does not wedge the handle).

## 8. Test strategy

- **Host unit tests (done, 22 passing):** all `HapticManager` policy via a mock
  backend; the oplus/ble/android backends' honest "unavailable" behaviour; the
  C ABI round-trip and null-safety.
- **JNI type-check (done):** `--features jni` compiles against `jni 0.21`.
- **On-device (pending hardware):** the matrix in `E-prototype.md`. Results must
  be labelled by method (inspection / static / emulator / adb / physical) and
  never inferred from a successful compile.

## 9. Distribution

- **Rust:** the crates publish to a registry or are consumed via git; `core` is
  `#![forbid(unsafe_code)]` and dependency-free.
- **Android:** build `stylo2-ffi` with `cargo-ndk` for `arm64-v8a` (+ others),
  package the `.so` as `jniLibs`, ship the Kotlin bridge as an AAR. The example
  app depends on the AAR.
- **Versioning:** semver across the workspace; the C ABI and `StyloError`/
  `Stylo2Status` mapping are the stability surface.
