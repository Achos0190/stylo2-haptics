package io.github.yeetingwaterbottle.stylo2.poc

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.content.pm.PackageManager
import android.os.Build
import android.os.IBinder
import com.oplus.ipemanager.sdk.ISdkAidlInterface
import com.oplus.ipemanager.sdk.Vibration

/**
 * Minimal Route-A probe and proof-of-concept.
 *
 * It attempts the `com.oplus.ipemanager` binder route from an ordinary app and
 * reports each stage separately — **discovery → binding → authorization →
 * execution** — so that one run on the device produces the evidence the mission
 * requires and, if the actuator fires, *is* the proof of concept.
 *
 * It deliberately does **not** short-circuit on the IOT permission: it attempts
 * the bind regardless of whether the permission looks granted, and records
 * exactly where and how it is refused. Nothing here needs root; the only
 * question is whether the stock platform lets an ordinary caller through.
 */
class StyloPocClient(private val context: Context) {

    companion object {
        const val PKG = "com.oplus.ipemanager"
        const val ACTION = "com.oplus.ipemanager.ACTION.PENCIL_SDK"
        const val DESCRIPTOR = "com.oplus.ipemanager.sdk.ISdkAidlInterface"
        const val PERM = "com.oplus.permission.safe.IOT"
    }

    /** UI log sink; set by the Activity. */
    var onLog: (String) -> Unit = {}

    /** Connection status sink; set by the Activity. */
    var onConnected: (Boolean) -> Unit = {}

    private var sdk: ISdkAidlInterface? = null
    private var bound = false

    private fun log(s: String) = onLog(s)

    private val conn = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName, binder: IBinder) {
            val desc = try { binder.interfaceDescriptor } catch (t: Throwable) { "?(${t.javaClass.simpleName})" }
            log("BIND: connected to $name; descriptor=$desc")
            if (desc == DESCRIPTOR) {
                sdk = ISdkAidlInterface.Stub.asInterface(binder)
                onConnected(true)
                // AUTHORIZATION probe: a harmless query. If the service validates
                // the caller (UID/package/signature) it will reject this even
                // though the bind succeeded — that isolates B2 from B1.
                try {
                    val state = sdk!!.getPencilConnectState()
                    log("AUTHZ: getPencilConnectState() = $state  (service accepted the call)")
                } catch (t: Throwable) {
                    log("AUTHZ: query threw ${t.javaClass.simpleName}: ${t.message}  (caller rejected by the service -> B2)")
                }
            } else {
                log("BIND: unexpected descriptor — the reproduced AIDL may not match this firmware")
                onConnected(false)
            }
        }

        override fun onServiceDisconnected(name: ComponentName) {
            log("BIND: disconnected $name")
            sdk = null
            bound = false
            onConnected(false)
        }
    }

    /** Stage 0: environment facts. */
    fun describeEnvironment() {
        log("─".repeat(32))
        log("ENV: Android ${Build.VERSION.SDK_INT}  ${Build.MANUFACTURER} ${Build.MODEL} (${Build.DEVICE})")
        val granted = context.checkSelfPermission(PERM) == PackageManager.PERMISSION_GRANTED
        log("ENV: $PERM = ${if (granted) "GRANTED" else "DENIED"}")
        val version = try {
            context.packageManager.getPackageInfo(PKG, 0).versionName
        } catch (t: Throwable) {
            null
        }
        log("ENV: $PKG version = ${version ?: "NOT INSTALLED"}")
    }

    /** Stage 1–2: discovery + binding (attempted regardless of permission). */
    fun bind() {
        if (bound) {
            log("BIND: already bound")
            return
        }
        val intent = Intent(ACTION).setPackage(PKG)
        val matches = context.packageManager.queryIntentServices(intent, 0)
        log("DISCOVERY: ${matches.size} service match(es) for $ACTION")
        for (m in matches) {
            val si = m.serviceInfo
            log("  -> ${si.packageName}/${si.name} exported=${si.exported} bindPermission=${si.permission ?: "none"}")
        }
        val granted = context.checkSelfPermission(PERM) == PackageManager.PERMISSION_GRANTED
        log("BIND: attempting bindService (IOT ${if (granted) "granted" else "NOT granted — attempting anyway"})")
        try {
            val ok = context.bindService(intent, conn, Context.BIND_AUTO_CREATE)
            bound = ok
            log("BIND: bindService returned $ok")
            if (!ok) {
                try { context.unbindService(conn) } catch (_: Throwable) {}
                log("BIND: could not bind — service missing/not exported, or permission enforced at bind (B1/B2)")
            }
        } catch (se: SecurityException) {
            log("BIND: SecurityException: ${se.message}  (framework/permission refused the bind -> B1)")
        } catch (t: Throwable) {
            log("BIND: ${t.javaClass.simpleName}: ${t.message}")
        }
    }

    /** Stage 3 (execution): select a profile. */
    fun setProfile(profile: Vibration) {
        val api = sdk ?: run { log("SET: not connected"); return }
        try {
            api.setVibrationType(profile.ordinal)
            log("EXEC: setVibrationType(${profile.name}=${profile.ordinal}) ok")
        } catch (t: Throwable) {
            log("EXEC: setVibrationType threw ${t.javaClass.simpleName}: ${t.message}")
        }
    }

    /** Stage 3 (execution): start feedback — the physical test. */
    fun start() {
        val api = sdk ?: run { log("START: not connected"); return }
        try {
            api.startFeedBackVibration()
            log("EXEC: startFeedBackVibration() returned — WATCH THE PEN: did it vibrate?")
        } catch (t: Throwable) {
            log("EXEC: startFeedBackVibration threw ${t.javaClass.simpleName}: ${t.message}")
        }
    }

    /** Stage 3 (execution): stop feedback. */
    fun stop() {
        val api = sdk ?: run { log("STOP: not connected"); return }
        try {
            api.stopFeedBackVibration()
            log("EXEC: stopFeedBackVibration() ok")
        } catch (t: Throwable) {
            log("EXEC: stopFeedBackVibration threw ${t.javaClass.simpleName}: ${t.message}")
        }
    }

    fun unbind() {
        if (!bound) {
            log("UNBIND: not bound")
            return
        }
        try { context.unbindService(conn) } catch (t: Throwable) { log("UNBIND: ${t.message}") }
        bound = false
        sdk = null
        onConnected(false)
        log("UNBIND: done")
    }
}
