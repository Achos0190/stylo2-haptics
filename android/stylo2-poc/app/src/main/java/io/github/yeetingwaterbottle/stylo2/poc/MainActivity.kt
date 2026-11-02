package io.github.yeetingwaterbottle.stylo2.poc

import android.app.Activity
import android.graphics.Typeface
import android.os.Bundle
import android.view.View
import android.view.ViewGroup
import android.widget.AdapterView
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.Spinner
import android.widget.TextView
import com.oplus.ipemanager.sdk.Vibration

/**
 * The minimal test screen: connection status, profile selection, a trigger
 * button, a stop button. Built programmatically to avoid any UI dependency.
 */
class MainActivity : Activity() {

    private lateinit var client: StyloPocClient
    private lateinit var logView: TextView
    private lateinit var statusView: TextView
    private var profile = Vibration.PENCIL

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        client = StyloPocClient(this)

        val root = LinearLayout(this)
        root.orientation = LinearLayout.VERTICAL
        val pad = (16 * resources.displayMetrics.density).toInt()
        root.setPadding(pad, pad, pad, pad)

        statusView = TextView(this)
        statusView.text = "Status: not connected"
        statusView.setTypeface(null, Typeface.BOLD)
        root.addView(statusView)

        val spinner = Spinner(this)
        spinner.adapter = ArrayAdapter(
            this,
            android.R.layout.simple_spinner_dropdown_item,
            listOf("Pencil", "Eraser", "Ballpoint", "Pen"),
        )
        spinner.onItemSelectedListener = object : AdapterView.OnItemSelectedListener {
            override fun onItemSelected(parent: AdapterView<*>?, view: View?, position: Int, id: Long) {
                profile = when (position) {
                    0 -> Vibration.PENCIL
                    1 -> Vibration.ERASER
                    2 -> Vibration.BALLPEN
                    3 -> Vibration.PEN
                    else -> Vibration.PENCIL
                }
            }
            override fun onNothingSelected(parent: AdapterView<*>?) {}
        }
        root.addView(spinner)

        root.addView(button("1. Bind / probe service") {
            client.describeEnvironment()
            client.bind()
        })
        root.addView(button("2. Trigger haptic (set profile + start)") {
            client.setProfile(profile)
            client.start()
        })
        root.addView(button("3. Stop feedback") { client.stop() })
        root.addView(button("Unbind") { client.unbind() })

        logView = TextView(this)
        logView.setTypeface(Typeface.MONOSPACE)
        logView.textSize = 11f
        logView.setTextIsSelectable(true)
        val scroll = ScrollView(this)
        scroll.addView(logView)
        root.addView(
            scroll,
            LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f),
        )

        setContentView(root)

        client.onLog = { line -> runOnUiThread { logView.append(line + "\n") } }
        client.onConnected = { connected ->
            runOnUiThread {
                statusView.text = "Status: " + if (connected) "CONNECTED (SDK bound)" else "not connected"
            }
        }
    }

    private fun button(label: String, onClick: () -> Unit): Button {
        val b = Button(this)
        b.text = label
        b.setOnClickListener { onClick() }
        return b
    }

    override fun onDestroy() {
        super.onDestroy()
        client.unbind()
    }
}
