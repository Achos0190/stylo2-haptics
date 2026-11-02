# stylo2-bridge (Kotlin)

The minimal Android integration layer over the Rust `stylo2` core. It exposes
[`Stylo2Haptics`](src/main/kotlin/io/github/yeetingwaterbottle/stylo2/haptics/Stylo2Haptics.kt):
`connect` / `disconnect` / `setProfile` / `startFeedback` / `stopFeedback` and a
`StateFlow<ConnectionState>`, mirroring the existing app's ergonomics. All
sequencing policy lives in the native core; this layer only marshals.

> The `:stylo2-bridge` module of the repo-root Gradle build. The Kotlin
> compiles without the native library (`:stylo2-bridge:assembleDebug`); the Rust
> `.so` is only needed at runtime. Not exercised on a device. The Rust side it
> binds to is built and tested (`../../stylo2`).

## Build the Kotlin module

From the **repo root** (with `ANDROID_HOME` pointing at your Android SDK, or a
`local.properties` with `sdk.dir`; use `gradlew.bat` on Windows):

```bash
./gradlew :stylo2-bridge:assembleDebug
```

To consume it from an app module, add
`implementation(project(":stylo2-bridge"))` to that module's `dependencies`.

## Build the native library

```bash
cd ../../stylo2
cargo install cargo-ndk
cargo ndk -t arm64-v8a -t x86_64 \
    -o ../android/stylo2-bridge/src/main/jniLibs \
    build -p stylo2-ffi --release --features jni
```

This produces `libstylo2_ffi.so` per ABI; `NativeBridge` loads it via
`System.loadLibrary("stylo2_ffi")`.

## Module dependencies

- `org.jetbrains.kotlinx:kotlinx-coroutines-core` (for `StateFlow`).
- Standard AGP Android library plugin; `minSdk 30` to match the app.

## Wiring the real accessory-service transport

`Stylo2Haptics.createOplus()` currently returns the core in its honest
`SERVICE_UNAVAILABLE` state, because the privileged binder bind is **not**
available to an ordinary app on stock firmware (see
`../../stylo2/docs/B-feasibility.md`). Two production wirings are possible once a
legitimate route exists:

1. **Kotlin owns the binder (simplest).** Keep the repository's existing
   `StylusVibrationClient` for the AIDL calls (bind `ACTION.PENCIL_SDK`, confirm
   the `ISdkAidlInterface` descriptor, call `setVibrationType` /
   `startFeedBackVibration` / `stopFeedBackVibration`). Use the Rust core only
   for portable state/guarding, or skip it for this route. No extra JNI.
2. **Native delegates to a Kotlin transport.** Extend `stylo2-ffi` with JNI
   up-calls so the native `OplusIpeBackend` forwards `BinderTransport` calls to
   a Kotlin object. Heavier; only worth it to share one code path across
   platforms.

Either way, the reported profile→int mapping stays only in
`stylo2-oplus::wire`, and failures surface as `Stylo2Status`
(`PERMISSION_DENIED` / `UNAUTHORIZED` / …) — never as a fake success.

## Usage

```kotlin
val haptics = Stylo2Haptics.createOplus()   // or .createMock() for UI dev
when (haptics.connect()) {
    Stylo2Status.OK -> {
        haptics.setProfile(VibrationProfile.PENCIL)
        haptics.startFeedback()
    }
    else -> { /* render haptics.state.value: PERMISSION_DENIED, etc. */ }
}
// later: haptics.stopFeedback(); haptics.close()
```
