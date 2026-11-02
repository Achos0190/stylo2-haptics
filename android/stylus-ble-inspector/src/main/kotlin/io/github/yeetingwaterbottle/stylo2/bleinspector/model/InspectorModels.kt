package io.github.yeetingwaterbottle.stylo2.bleinspector.model

/** A device seen during scanning. */
data class ScannedDevice(
    val address: String,
    val name: String?,
    val rssi: Int,
    val connectable: Boolean,
    val serviceUuids: List<String>,
    val manufacturerIds: List<Int>,
    val isStylusCandidate: Boolean,
    val candidateReason: String?,
)

/** Connection lifecycle of the single inspected link. */
enum class ConnState { DISCONNECTED, CONNECTING, CONNECTED }

/** Decoded characteristic property flags. */
data class CharProperties(
    val read: Boolean,
    val write: Boolean,
    val writeNoResponse: Boolean,
    val notify: Boolean,
    val indicate: Boolean,
) {
    fun summary(): String = buildList {
        if (read) add("READ")
        if (write) add("WRITE")
        if (writeNoResponse) add("WRITE_NR")
        if (notify) add("NOTIFY")
        if (indicate) add("INDICATE")
    }.joinToString("|").ifEmpty { "—" }

    /** Writable in either form — the first thing a haptic-control probe looks for. */
    val writable: Boolean get() = write || writeNoResponse
    val subscribable: Boolean get() = notify || indicate
}

/** One characteristic within a discovered service. */
data class GattCharacteristicInfo(
    val serviceUuid: String,
    val uuid: String,
    val instanceId: Int,
    val properties: CharProperties,
    val descriptorUuids: List<String>,
)

/** One discovered GATT service. */
data class GattServiceInfo(
    val uuid: String,
    val primary: Boolean,
    val characteristics: List<GattCharacteristicInfo>,
)

/** A single timestamped line in the operation log. */
data class LogEntry(val timestampMs: Long, val tag: String, val message: String)
