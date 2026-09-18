import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.mpp.apple.XCFramework

plugins {
    alias(libs.plugins.androidKotlinMultiplatformLibrary)
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.kotlinSerialization)
    alias(libs.plugins.atomicfu)
    alias(libs.plugins.mavenPublish)
    signing
}

kotlin {
    android {
        namespace = "com.blockstream.glsdk"
        compileSdk = 37
        minSdk = 24

        compilerOptions { jvmTarget.set(JvmTarget.JVM_1_8) }

        withHostTestBuilder {}.configure {}
        withDeviceTestBuilder {
            sourceSetTreeName = "test"
        }.configure {
            instrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        }

        optimization {
            consumerKeepRules.publish = true
            consumerKeepRules.file("consumer-rules.pro")
        }
    }

    jvm()

    val xcf = XCFramework()

    listOf(
        iosArm64(),
        iosSimulatorArm64()
    ).forEach {

        it.binaries.framework {
            baseName = "glsdk"
            xcf.add(this)
        }

        val platform = when (it.targetName) {
            "iosSimulatorArm64" -> "ios_simulator_arm64"
            "iosArm64" -> "ios_arm64"
            else -> error("Unsupported target $name")
        }

        it.compilations["main"].cinterops {
            create("glsdkCInterop") {
                defFile(project.file("src/nativeInterop/cinterop/glsdk.def"))
                includeDirs(
                    project.file("src/nativeInterop/cinterop/headers/glsdk/"),
                    project.file("src/libs/$platform")
                )
            }
        }
    }

    compilerOptions.freeCompilerArgs.add("-Xexpect-actual-classes")

    sourceSets {
        commonMain.dependencies {
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.kotlinx.serialization.core)
            implementation(libs.kotlinx.serialization.json)
            implementation(libs.annotation)
            implementation(libs.okio)
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
        }
        androidMain.dependencies {
            implementation(libs.jna.get()) {
                artifact { type = "aar" }
            }
        }
        jvmMain.dependencies {
            implementation(libs.jna)
        }
        getByName("androidHostTest").dependencies {
            implementation(libs.junit)
        }
        getByName("androidDeviceTest").dependencies {
            implementation(libs.junit.ext)
            implementation(libs.espresso.core)
        }
    }
}

val libraryVersion: String by project

mavenPublishing {
    coordinates(groupId = "com.blockstream", artifactId = "glsdk", version = libraryVersion)

    pom {
        name = "glsdk"
        description = "GreenlightSDK"
        url = "https://blockstream.com"
        licenses {
            license {
                name = "BSD-MIT"
                url = "https://github.com/blockstream/glsdk/blob/main/LICENSE"
            }
        }
        developers {
            developer {
                id = "cdecker"
                name = "Christian Decker"
                email = "decker@blockstream.com"
            }
        }
        scm {
            connection = "scm:git:github.com/blockstream/glsdk.git"
            developerConnection = "scm:git:ssh://github.com/blockstream/glsdk.git"
            url = "https://github.com/blockstream/glsdk"
        }
    }

    publishToMavenCentral()
    signAllPublications()
}

extensions.configure<SigningExtension> {
    useGpgCmd()
}

// Do not require signing when publishing to Maven Local
// Allows `./gradlew publishToMavenLocal` (or `publishToLocalMaven`) without GPG setup
tasks.withType<Sign>().configureEach {
    onlyIf {
        val taskNames = gradle.startParameter.taskNames
        // Skip signing if the build is targeting the local Maven repository
        taskNames.none { it.contains("publishToMavenLocal", ignoreCase = true) || it.contains("publishToLocalMaven", ignoreCase = true) }
    }
}
