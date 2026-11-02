package io.github.yeetingwaterbottle.stylo2.bleinspector.ble

import android.annotation.SuppressLint
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothGattService
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothProfile
import android.bluetooth.BluetoothStatusCodes
import android.content.Context
import io.github.yeetingwaterbottle.stylo2.bleinspector.model.CharProperties
import io.github.yeetingwaterbottle.stylo2.bleinspector.model.ConnState
import io.github.yeetingwaterbottle.stylo2.bleinspector.model.GattCharacteristicInfo
import io.github.yeetingwaterbottle.stylo2.bleinspector.model.GattServiceInfo
import io.github.yeetingwaterbottle.stylo2.bleinspector.model.LogEntry
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.withTimeout
import java.util.UUID

/**
 * Drives a single GATT link for inspection: connect, discover, read, write,
 * subscribe — each serialised through a [GattOperationQueue] and logged.
 *
 * Modern (API 33+) BLE APIs only; the caller must hold `BLUETOOTH_CONNECT`.
 */
class GattInspector(private val context: Context) {

    private val manager = context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    private val queue = GattOperationQueue()

    private var gatt: BluetoothGatt? = null
    private var connectDeferred: CompletableDeferred<Unit>? = null

    private val _state = MutableStateFlow(ConnState.DISCONNECTED)
    val state: StateFlow<ConnState> = _state.asStateFlow()

    private val _services = MutableStateFlow<List<GattServiceInfo>>(emptyList())
    val services: StateFlow<List<GattServiceInfo>> = _services.asStateFlow()

    private val _log = MutableStateFlow<List<LogEntry>>(emptyList())
    val log: StateFlow<List<LogEntry>> = _log.asStateFlow()

    var connectedAddress: String? = null
        private set

    private fun log(tag: String, message: String) {
        _log.update { (it + LogEntry(System.currentTimeMillis(), tag, message)).takeLast(MAX_LOG) }
    }

    /** Clear the operation log. */
    fun clearLog() {
        _log.value = emptyList()
    }

    /** Append a UI-originated note to the log (e.g. input validation). */
    fun note(tag: String, message: String) = log(tag, message)

    private val callback = object : BluetoothGattCallback() {
        override fun onConnectionStateChange(g: BluetoothGatt, status: Int, newState: Int) {
            when (newState) {
                BluetoothProfile.STATE_CONNECTED -> {
                    _state.value = ConnState.CONNECTED
                    log("link", "connected (status=$status)")
                    connectDeferred?.complete(Unit)
                }

                BluetoothProfile.STATE_DISCONNECTED -> {
                    _state.value = ConnState.DISCONNECTED
                    _services.value = emptyList()
                    log("link", "disconnected (status=$status)")
                    queue.shutdown("disconnected")
                    connectDeferred?.completeExceptionally(
                        IllegalStateException("disconnected before ready (status=$status)"),
                    )
                }
            }
        }

        override fun onServicesDiscovered(g: BluetoothGatt, status: Int) {
            if (status == BluetoothGatt.GATT_SUCCESS) {
                _services.value = mapServices(g)
                log("discover", "${_services.value.size} services")
                queue.complete(GattOpKey.Discover, Unit)
            } else {
                queue.fail(GattOpKey.Discover, status)
            }
        }

        override fun onCharacteristicRead(
            g: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            value: ByteArray,
            status: Int,
        ) {
            val key = GattOpKey.CharacteristicRead(characteristic.instanceId, characteristic.uuid.toString())
            if (status == BluetoothGatt.GATT_SUCCESS) {
                log("read", "${characteristic.uuid} = ${HexCodec.hex(value)}")
                queue.complete(key, value)
            } else {
                log("read", "${characteristic.uuid} failed (status=$status)")
                queue.fail(key, status)
            }
        }

        override fun onCharacteristicWrite(
            g: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            status: Int,
        ) {
            val key = GattOpKey.CharacteristicWrite(characteristic.instanceId, characteristic.uuid.toString())
            if (status == BluetoothGatt.GATT_SUCCESS) {
                log("write", "${characteristic.uuid} ok")
                queue.complete(key, Unit)
            } else {
                log("write", "${characteristic.uuid} failed (status=$status)")
                queue.fail(key, status)
            }
        }

        override fun onCharacteristicChanged(
            g: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            value: ByteArray,
        ) {
            log("notify", "${characteristic.uuid} -> ${HexCodec.hex(value)}")
        }

        override fun onDescriptorWrite(
            g: BluetoothGatt,
            descriptor: BluetoothGattDescriptor,
            status: Int,
        ) {
            val key = GattOpKey.DescriptorWrite(
                descriptor.characteristic.instanceId,
                descriptor.uuid.toString(),
            )
            if (status == BluetoothGatt.GATT_SUCCESS) {
                queue.complete(key, Unit)
            } else {
                queue.fail(key, status)
            }
        }

        override fun onMtuChanged(g: BluetoothGatt, mtu: Int, status: Int) {
            log("mtu", "$mtu (status=$status)")
        }
    }

    /** Connect and wait until the link is usable (or fail/timeout). */
    @SuppressLint("MissingPermission")
    suspend fun connect(address: String): Result<Unit> {
        val adapter = manager.adapter ?: return Result.failure(IllegalStateException("No Bluetooth adapter"))
        disconnect()
        queue.reopen()
        _state.value = ConnState.CONNECTING
        connectedAddress = address
        log("link", "connecting to $address")
        val deferred = CompletableDeferred<Unit>()
        connectDeferred = deferred
        gatt = adapter.getRemoteDevice(address)
            .connectGatt(context, false, callback, android.bluetooth.BluetoothDevice.TRANSPORT_LE)
        return runCatching { withTimeout(CONNECT_TIMEOUT_MS) { deferred.await() } }
            .onFailure { log("link", "connect failed: ${it.message}") }
    }

    /** Discover services on the connected link. */
    @SuppressLint("MissingPermission")
    suspend fun discover(): Result<Unit> = runCatching {
        val g = gatt ?: error("not connected")
        log("discover", "starting service discovery")
        queue.submit<Unit>(GattOpKey.Discover, "Discover services", timeoutMs = 20_000) {
            g.discoverServices()
        }
        Unit
    }.onFailure { log("discover", "failed: ${it.message}") }

    /** Read a characteristic's value. */
    @SuppressLint("MissingPermission")
    suspend fun read(info: GattCharacteristicInfo): Result<ByteArray> = runCatching {
        val g = gatt ?: error("not connected")
        val ch = find(g, info) ?: error("characteristic not found")
        queue.submit<ByteArray>(
            GattOpKey.CharacteristicRead(ch.instanceId, ch.uuid.toString()),
            "Read ${info.uuid}",
        ) { g.readCharacteristic(ch) }.value
    }.onFailure { log("read", "failed: ${it.message}") }

    /** Write bytes to a characteristic, with or without response. */
    @SuppressLint("MissingPermission")
    suspend fun write(
        info: GattCharacteristicInfo,
        value: ByteArray,
        withResponse: Boolean,
    ): Result<Unit> = runCatching {
        val g = gatt ?: error("not connected")
        val ch = find(g, info) ?: error("characteristic not found")
        val type = if (withResponse) {
            BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT
        } else {
            BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE
        }
        log("write", "${info.uuid} <- ${HexCodec.hex(value)} (${if (withResponse) "ack" else "no-ack"})")
        queue.submit<Unit>(
            GattOpKey.CharacteristicWrite(ch.instanceId, ch.uuid.toString()),
            "Write ${info.uuid}",
        ) { g.writeCharacteristic(ch, value, type) == BluetoothStatusCodes.SUCCESS }
        Unit
    }.onFailure { log("write", "failed: ${it.message}") }

    /** Enable or disable notifications/indications for a characteristic. */
    @SuppressLint("MissingPermission")
    suspend fun setNotify(info: GattCharacteristicInfo, enable: Boolean): Result<Unit> = runCatching {
        val g = gatt ?: error("not connected")
        val ch = find(g, info) ?: error("characteristic not found")
        if (!g.setCharacteristicNotification(ch, enable)) error("setCharacteristicNotification rejected")
        val cccd = ch.getDescriptor(CCCD_UUID) ?: error("characteristic has no CCCD (0x2902)")
        val value = when {
            !enable -> BluetoothGattDescriptor.DISABLE_NOTIFICATION_VALUE
            info.properties.notify -> BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
            else -> BluetoothGattDescriptor.ENABLE_INDICATION_VALUE
        }
        log("notify", "${info.uuid} CCCD ${if (enable) "enable" else "disable"}")
        queue.submit<Unit>(
            GattOpKey.DescriptorWrite(ch.instanceId, cccd.uuid.toString()),
            "CCCD ${info.uuid}",
        ) { g.writeDescriptor(cccd, value) == BluetoothStatusCodes.SUCCESS }
        Unit
    }.onFailure { log("notify", "failed: ${it.message}") }

    /** Close the link and reset discovered state. */
    @SuppressLint("MissingPermission")
    fun disconnect() {
        gatt?.let { runCatching { it.disconnect(); it.close() } }
        gatt = null
        queue.shutdown("closed")
        _state.value = ConnState.DISCONNECTED
        _services.value = emptyList()
    }

    /** A plain-text evidence report for export. */
    fun buildReport(): String = buildString {
        appendLine("Stylo2 BLE Inspector — findings")
        appendLine("device: ${connectedAddress ?: "(none)"}")
        appendLine("state: ${_state.value}")
        appendLine()
        appendLine("GATT table:")
        for (s in _services.value) {
            appendLine("  service ${s.uuid}${if (s.primary) " (primary)" else ""}")
            for (c in s.characteristics) {
                appendLine("    char ${c.uuid}  [${c.properties.summary()}]")
                for (d in c.descriptorUuids) appendLine("      desc $d")
            }
        }
        appendLine()
        appendLine("log:")
        for (e in _log.value) appendLine("  ${e.timestampMs} [${e.tag}] ${e.message}")
    }

    private fun mapServices(g: BluetoothGatt): List<GattServiceInfo> = g.services.map { s ->
        GattServiceInfo(
            uuid = s.uuid.toString(),
            primary = s.type == BluetoothGattService.SERVICE_TYPE_PRIMARY,
            characteristics = s.characteristics.map { c ->
                val p = c.properties
                GattCharacteristicInfo(
                    serviceUuid = s.uuid.toString(),
                    uuid = c.uuid.toString(),
                    instanceId = c.instanceId,
                    properties = CharProperties(
                        read = p and BluetoothGattCharacteristic.PROPERTY_READ != 0,
                        write = p and BluetoothGattCharacteristic.PROPERTY_WRITE != 0,
                        writeNoResponse = p and BluetoothGattCharacteristic.PROPERTY_WRITE_NO_RESPONSE != 0,
                        notify = p and BluetoothGattCharacteristic.PROPERTY_NOTIFY != 0,
                        indicate = p and BluetoothGattCharacteristic.PROPERTY_INDICATE != 0,
                    ),
                    descriptorUuids = c.descriptors.map { it.uuid.toString() },
                )
            },
        )
    }

    private fun find(g: BluetoothGatt, info: GattCharacteristicInfo): BluetoothGattCharacteristic? {
        for (s in g.services) {
            if (s.uuid.toString() != info.serviceUuid) continue
            for (c in s.characteristics) {
                if (c.uuid.toString() == info.uuid && c.instanceId == info.instanceId) return c
            }
        }
        return null
    }

    companion object {
        private const val MAX_LOG = 2_000
        private const val CONNECT_TIMEOUT_MS = 15_000L
        private val CCCD_UUID: UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")
    }
}
