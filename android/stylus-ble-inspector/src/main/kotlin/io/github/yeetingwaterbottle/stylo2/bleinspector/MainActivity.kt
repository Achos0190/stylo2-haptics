package io.github.yeetingwaterbottle.stylo2.bleinspector

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import io.github.yeetingwaterbottle.stylo2.bleinspector.ui.InspectorApp
import io.github.yeetingwaterbottle.stylo2.bleinspector.ui.InspectorTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent { InspectorTheme { InspectorApp() } }
    }
}
