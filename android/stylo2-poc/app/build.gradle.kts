plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "io.github.yeetingwaterbottle.stylo2.poc"
    compileSdk = 36

    defaultConfig {
        applicationId = "io.github.yeetingwaterbottle.stylo2.poc"
        minSdk = 30
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release { isMinifyEnabled = false }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
}

// No androidx/Compose: a plain Activity + android.widget views keep the PoC as
// small and buildable as possible. The Kotlin plugin supplies the stdlib.
dependencies {}
