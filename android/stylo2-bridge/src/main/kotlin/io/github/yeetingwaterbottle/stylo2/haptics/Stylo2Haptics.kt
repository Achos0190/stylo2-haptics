package io.github.yeetingwaterbottle.stylo2.haptics

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

/** Connection state, mirroring `stylo2_core::ConnectionState`. */
enum class ConnectionState {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    PERMISSION_DENIED,
    SERVICE_UNAVAILABLE;

    internal companion object {
        fun fromCode(code: Int): ConnectionState = when (code) {
            0 -> DISCONNECTED
            1 -> CONNECTING
            2 -> CONNECTED
            3 -> PERMISSION_DENIED
            4 -> SERVICE_UNAVAILABLE
            else -> SERVICE_UNAVAILABLE // includes -1 (null handle): treat as unusable
        }
    }
}

/**
 * Vibration profile. [abiIndex] is stylo2's own stable index passed across the
 * FFI — **not** the vendor `setVibrationType(int)` wire value (that mapping is
 * isolated in `stylo2-oplus`).
 */
enum class VibrationProfile {
    PENCIL,
    ERASER,
    BALLPOINT,
    PEN;

    internal val abiIndex: Int get() = ordinal
}

/** Result of a native call, mirroring `stylo2_ffi::Stylo2Status`. */
enum class Stylo2Status {
    OK,
    NOT_CONNECTED,
    PERMISSION_DENIED,
    SERVICE_UNAVAILABLE,
    UNAUTHORIZED,
    VIBRATION_DISABLED,
    UNSUPPORTED,
    INVALID_REQUEST,
    TRANSPORT,
    BACKEND,
    NULL_HANDLE,
    BAD_ARGUMENT,
    UNKNOWN;

    val isOk: Boolean get() = this == OK

    internal companion object {
        fun fromCode(code: Int): Stylo2Status = when (code) {
            0 -> OK
            1 -> NOT_CONNECTED
            2 -> PERMISSION_DENIED
            3 -> SERVICE_UNAVAILABLE
            4 -> UNAUTHORIZED
            5 -> VIBRATION_DISABLED
            6 -> UNSUPPORTED
            7 -> INVALID_REQUEST
            8 -> TRANSPORT
            9 -> BACKEND
            20 -> NULL_HANDLE
            21 -> BAD_ARGUMENT
            else -> UNKNOWN
        }
    }
}

/**
 * High-level handle over the Rust haptic core.
 *
 * All the sequencing policy (guarding, de-duplication, capability gating,
 * honest error reporting) lives in the native core; this class only forwards
 * calls, refreshes [state], and frees the handle in [close].
 *
 * Construct with one of the factory methods. The real accessory-service binder
 * transport is wired per the module README; [createOplus] currently yields the
 * honest "service unavailable" core until that transport is attached, and
 * [createMock] is for UI development without a device.
 */
class Stylo2Haptics private constructor(private var handle: Long) : AutoCloseable {

    private val _state = MutableStateFlow(ConnectionState.DISCONNECTED)
    val state: StateFlow<ConnectionState> = _state

    private fun refreshState() {
        _state.value = ConnectionState.fromCode(NativeBridge.nativeConnectionState(handle))
    }

    /** Begin (or re-check) the connection; updates [state]. */
    fun connect(): Stylo2Status {
        val status = Stylo2Status.fromCode(NativeBridge.nativeConnect(handle))
        refreshState()
        return status
    }

    /** Tear down the connection; updates [state]. */
    fun disconnect(): Stylo2Status {
        val status = Stylo2Status.fromCode(NativeBridge.nativeDisconnect(handle))
        refreshState()
        return status
    }

    /** Select a profile. */
    fun setProfile(profile: VibrationProfile): Stylo2Status =
        Stylo2Status.fromCode(NativeBridge.nativeSetProfile(handle, profile.abiIndex))

    /** Start feedback (de-duplicated by the core). */
    fun startFeedback(): Stylo2Status =
        Stylo2Status.fromCode(NativeBridge.nativeStartFeedback(handle))

    /** Stop feedback (no-op if idle). */
    fun stopFeedback(): Stylo2Status =
        Stylo2Status.fromCode(NativeBridge.nativeStopFeedback(handle))

    /** Free the native handle. Safe to call more than once. */
    override fun close() {
        if (handle != 0L) {
            NativeBridge.nativeFree(handle)
            handle = 0L
            _state.value = ConnectionState.DISCONNECTED
        }
    }

    companion object {
        /** Honest default: the OnePlus binder core (reports unavailable until a
         *  JVM binder transport is attached — see README). */
        fun createOplus(): Stylo2Haptics =
            Stylo2Haptics(NativeBridge.nativeNewOplusUnavailable())

        /** In-memory mock core for UI development without a device. */
        fun createMock(): Stylo2Haptics =
            Stylo2Haptics(NativeBridge.nativeNewMock())
    }
}
