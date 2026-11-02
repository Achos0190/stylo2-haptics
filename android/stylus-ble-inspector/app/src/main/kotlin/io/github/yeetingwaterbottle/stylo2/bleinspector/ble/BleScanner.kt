package io.github.yeetingwaterbottle.stylo2.bleinspector.ble

import android.annotation.SuppressLint
import android.bluetooth.BluetoothManager
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import io.github.yeetingwaterbottle.stylo2.bleinspector.model.ScannedDevice
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow

/**
 * Low-energy scanner that emits the running set of seen devices, stylus
 * candidates sorted first. The caller must hold `BLUETOOTH_SCAN`.
 */
class BleScanner(context: Context) {

    private val adapter =
        (context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager).adapter

    @SuppressLint("MissingPermission")
    fun scan(): Flow<List<ScannedDevice>> = callbackFlow {
        val scanner = adapter?.bluetoothLeScanner
        if (scanner == null) {
            close(IllegalStateException("Bluetooth is off or unavailable"))
            return@callbackFlow
        }

        val found = LinkedHashMap<String, ScannedDevice>()

        val callback = object : ScanCallback() {
            override fun onScanResult(callbackType: Int, result: ScanResult) {
                val record = result.scanRecord
                val name = record?.deviceName
                val serviceUuids = record?.serviceUuids?.map { it.uuid.toString() } ?: emptyList()
                val mfg = record?.manufacturerSpecificData
                val mfgIds = buildList {
                    if (mfg != null) for (i in 0 until mfg.size()) add(mfg.keyAt(i))
                }
                val reason = StylusHeuristics.candidateReason(name, serviceUuids)
                found[result.device.address] = ScannedDevice(
                    address = result.device.address,
                    name = name,
                    rssi = result.rssi,
                    connectable = result.isConnectable,
                    serviceUuids = serviceUuids,
                    manufacturerIds = mfgIds,
                    isStylusCandidate = reason != null,
                    candidateReason = reason,
                )
                trySend(
                    found.values.sortedWith(
                        compareByDescending<ScannedDevice> { it.isStylusCandidate }
                            .thenByDescending { it.rssi },
                    ),
                )
            }

            override fun onScanFailed(errorCode: Int) {
                close(IllegalStateException("BLE scan failed (code $errorCode)"))
            }
        }

        val settings = ScanSettings.Builder()
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
            .build()
        scanner.startScan(null, settings, callback)
        awaitClose { runCatching { scanner.stopScan(callback) } }
    }
}
