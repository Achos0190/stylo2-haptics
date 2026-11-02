// Android library module for the stylo2 Kotlin bridge.
//
// Not wired into the root Gradle build (`settings.gradle.kts`) yet: it is the
// integration layer delivered alongside the Rust workspace. Add
// `include(":android:stylo2-bridge")` and the native `jniLibs` (see README) to
// consume it from an app.
plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
}

android {
    namespace = "io.github.yeetingwaterbottle.stylo2.haptics.bridge"
    compileSdk = 36

    defaultConfig {
        minSdk = 30
        // Native libraries are provided as prebuilt jniLibs produced by
        // `cargo ndk` (see README), so no externalNativeBuild here.
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }

    sourceSets["main"].kotlin.srcDir("src/main/kotlin")
}

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1")
}
