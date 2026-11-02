package io.github.yeetingwaterbottle.stylo2.haptics

/**
 * Thin JNI surface over `stylo2-ffi`. Names match the `Java_..._native*`
 * entry points in `stylo2/crates/stylo2-ffi/src/jni_bridge.rs` (built with
 * `--features jni`). This object marshals only; all policy lives in the Rust
 * core.
 *
 * The `long` handle is an opaque pointer from a `nativeNew*` call; pass it back
 * to exactly one `nativeFree`. Status ints map to [Stylo2Status]; the
 * connection-state int maps to [ConnectionState].
 */
internal object NativeBridge {
    init {
        System.loadLibrary("stylo2_ffi")
    }

    external fun nativeNewMock(): Long
    external fun nativeNewOplusUnavailable(): Long
    external fun nativeFree(handle: Long)
    external fun nativeConnect(handle: Long): Int
    external fun nativeDisconnect(handle: Long): Int
    external fun nativeConnectionState(handle: Long): Int
    external fun nativeSetProfile(handle: Long, profile: Int): Int
    external fun nativeStartFeedback(handle: Long): Int
    external fun nativeStopFeedback(handle: Long): Int
}
