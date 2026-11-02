package io.github.yeetingwaterbottle.stylo2.bleinspector.ui

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import io.github.yeetingwaterbottle.stylo2.bleinspector.ble.BleScanner
import io.github.yeetingwaterbottle.stylo2.bleinspector.ble.GattInspector
import io.github.yeetingwaterbottle.stylo2.bleinspector.ble.HexCodec
import io.github.yeetingwaterbottle.stylo2.bleinspector.model.GattCharacteristicInfo
import io.github.yeetingwaterbottle.stylo2.bleinspector.model.ScannedDevice
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.launch

class InspectorViewModel(app: Application) : AndroidViewModel(app) {

    private val scanner = BleScanner(app)
    val inspector = GattInspector(app)

    private val _scanning = MutableStateFlow(false)
    val scanning: StateFlow<Boolean> = _scanning.asStateFlow()

    private val _devices = MutableStateFlow<List<ScannedDevice>>(emptyList())
    val devices: StateFlow<List<ScannedDevice>> = _devices.asStateFlow()

    private val _scanError = MutableStateFlow<String?>(null)
    val scanError: StateFlow<String?> = _scanError.asStateFlow()

    private var scanJob: Job? = null

    fun startScan() {
        if (_scanning.value) return
        _scanError.value = null
        _devices.value = emptyList()
        _scanning.value = true
        scanJob = viewModelScope.launch {
            scanner.scan()
                .catch { _scanError.value = it.message; _scanning.value = false }
                .collect { _devices.value = it }
        }
    }

    fun stopScan() {
        scanJob?.cancel()
        scanJob = null
        _scanning.value = false
    }

    fun connect(address: String) {
        stopScan()
        viewModelScope.launch { inspector.connect(address) }
    }

    fun discover() = viewModelScope.launch { inspector.discover() }

    fun read(info: GattCharacteristicInfo) = viewModelScope.launch { inspector.read(info) }

    fun write(info: GattCharacteristicInfo, hexInput: String, withResponse: Boolean) =
        viewModelScope.launch {
            val bytes = HexCodec.parseHex(hexInput)
            if (bytes == null) {
                inspector.note("write", "invalid hex: \"$hexInput\"")
                return@launch
            }
            inspector.write(info, bytes, withResponse)
        }

    fun setNotify(info: GattCharacteristicInfo, enable: Boolean) =
        viewModelScope.launch { inspector.setNotify(info, enable) }

    fun disconnect() = inspector.disconnect()

    override fun onCleared() {
        stopScan()
        inspector.disconnect()
    }
}
