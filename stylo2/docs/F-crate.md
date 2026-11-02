# Deliverable F — Final reusable crate

What ships, and how another app reuses it. (Device-dependent behaviour remains
subject to the access routes in `B-feasibility.md`.)

## Contents

| Item | Location | State |
| --- | --- | --- |
| Rust source | `stylo2/crates/*` | Implemented, host-tested |
| Kotlin Android bridge | `android/stylo2-bridge` | Source provided |
| Build configuration | `stylo2/Cargo.toml`, per-crate `Cargo.toml` | Done |
| Integration documentation | this file + `D-implementation-plan.md` | Done |
| API documentation | `cargo doc -p stylo2-core --no-deps` | Generated from doc-comments |
| Test suite | `cargo test --workspace` (26 tests) | Passing on host |
| Example | `cargo run -p stylo2-core --example demo --features mock` | Runs |
| Install instructions | below (no root) | Done |

## Integrating into an Android app

1. **Native build.** Build `stylo2-ffi` for Android ABIs and bundle as
   `jniLibs`:
   ```bash
   cargo install cargo-ndk
   cargo ndk -t arm64-v8a -t x86_64 -o app/src/main/jniLibs \
       build -p stylo2-ffi --release --features jni
   ```
2. **Kotlin bridge.** Add `android/stylo2-bridge` (as a module or AAR). It loads
   the `.so`, holds the native handle, and exposes `Stylo2Haptics`.
3. **Permissions.** Declare `com.oplus.permission.safe.IOT` (expected denied on
   stock — surfaced, not hidden) and, for the BLE route,
   `BLUETOOTH_CONNECT`/`BLUETOOTH_SCAN` (requested at runtime).
4. **Use it:**
   ```kotlin
   val haptics = Stylo2Haptics.create(context)
   haptics.connect()
   if (haptics.state.value == ConnectionState.CONNECTED) {
       haptics.setProfile(VibrationProfile.PENCIL)
       haptics.startFeedback()
       // ... later
       haptics.stopFeedback()
   } else {
       // show haptics.state.value: PermissionDenied / ServiceUnavailable, etc.
   }
   ```

## Wiring a real transport (which route lit up?)

The core is backend-agnostic; pick the backend the device proved out:

- **Binder route (vendor-authorised or — out of scope here — privileged):** the
  Kotlin bridge performs the AIDL calls (reusing the existing
  `StylusVibrationClient` logic: bind `ACTION.PENCIL_SDK`, confirm the
  descriptor, call `setVibrationType`/`startFeedBackVibration`/
  `stopFeedBackVibration`). It maps results into `Stylo2Status` for the core's
  state tracking. The reported profile integers live only in
  `stylo2-oplus::wire`.
- **BLE route:** supply a verified `Stylo2GattProfile` to `BleBackend`; the
  Kotlin `GattTransport` does the GATT I/O.
- **Fallback:** `AndroidVibratorBackend` over a `VibratorManager` transport —
  clearly labelled as the tablet motor.

## Reuse outside Android

`stylo2-core` is pure, `#![forbid(unsafe_code)]`, dependency-free. A desktop
tool could implement `HapticBackend` over `btleplug` once the BLE protocol is
known, and reuse all of the policy (guarding, de-dup, capability gating) and the
C ABI unchanged.

## Honesty guarantees carried into the final crate

- No root, no priv-app, no firmware edits anywhere in this workspace.
- An unavailable/unauthorised route reports its precise state; it is never
  presented as working.
- Controls the interface does not expose (intensity/duration/frequency/
  waveform/custom patterns) are not offered.
- No vendor keys, tokens, or credentials are bundled, extracted, or required by
  this code.
