// Android library module for the stylo2 Kotlin bridge (`:stylo2-bridge`).
//
// Included from the root `settings.gradle.kts`. The native `jniLibs` produced by
// `cargo ndk` (see README) are only needed at runtime, not to compile.
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
    implementation(libs.kotlinx.coroutines.core)
}
