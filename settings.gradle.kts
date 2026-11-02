pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "Stylo2 Haptics"

// `:app` is the upstream Magisk companion app. The three modules below live
// under android/ and are wired in here so the whole repo is ONE Gradle build:
//   ./gradlew :stylo2-poc:installDebug
include(":app")

include(":stylo2-poc")
project(":stylo2-poc").projectDir = file("android/stylo2-poc")

include(":stylus-ble-inspector")
project(":stylus-ble-inspector").projectDir = file("android/stylus-ble-inspector")

include(":stylo2-bridge")
project(":stylo2-bridge").projectDir = file("android/stylo2-bridge")
