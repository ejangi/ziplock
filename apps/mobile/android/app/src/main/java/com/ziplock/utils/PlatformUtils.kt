package com.ziplock.utils

import android.os.Build
import android.util.Log
import com.ziplock.ffi.ZipLockNative

/**
 * Platform detection utilities for ZipLock Android app
 * Helps identify emulator types and potential compatibility issues
 */
object PlatformUtils {
    private const val TAG = "PlatformUtils"

    /**
     * Check if running on Android emulator
     */
    fun isEmulator(): Boolean {
        return Build.FINGERPRINT.startsWith("generic") ||
                Build.FINGERPRINT.startsWith("unknown") ||
                Build.MODEL.contains("google_sdk") ||
                Build.MODEL.contains("Emulator") ||
                Build.MODEL.contains("Android SDK built for") ||
                Build.MANUFACTURER.contains("Genymotion") ||
                Build.BRAND.startsWith("generic") && Build.DEVICE.startsWith("generic") ||
                "google_sdk" == Build.PRODUCT
    }

    /**
     * Check if running on x86/x86_64 emulator (potentially problematic for 7z operations)
     */
    fun isX86Emulator(): Boolean {
        return isEmulator() && Build.SUPPORTED_ABIS.any {
            it.contains("x86")
        }
    }

    /**
     * Check if running in Android emulator using native library detection
     * Falls back to Java detection if native library is not available
     */
    fun isEmulatorNative(): Boolean {
        return try {
            ZipLockNative.isAndroidEmulator()
        } catch (e: Exception) {
            Log.w(TAG, "Native emulator detection failed, falling back to Java detection", e)
            isEmulator()
        }
    }

    /**
     * Check if running on any Android emulator (all have potential 7z library issues)
     */
    fun isAnyEmulator(): Boolean {
        return isEmulator()
    }

    /**
     * Check if running on ARM emulator (recommended for development)
     */
    fun isArmEmulator(): Boolean {
        return isEmulator() && Build.SUPPORTED_ABIS.any {
            it.contains("arm")
        }
    }

    /**
     * Get primary ABI architecture
     */
    fun getPrimaryAbi(): String {
        return Build.SUPPORTED_ABIS.firstOrNull() ?: "unknown"
    }

    /**
     * Get all supported ABIs
     */
    fun getAllAbis(): List<String> {
        return Build.SUPPORTED_ABIS.toList()
    }

    /**
     * Check if platform has known issues with archive operations
     */
    fun hasKnownArchiveIssues(): Boolean {
        return try {
            ZipLockNative.hasArchiveCompatibilityIssues()
        } catch (e: Exception) {
            Log.w(TAG, "Native compatibility check failed, falling back to emulator detection", e)
            isEmulator() // All Android emulators have sevenz_rust2 issues
        }
    }

    /**
     * Get platform-specific recommendations for archive operations
     */
    fun getArchiveCompatibilityMessage(): String? {
        // Try to get native platform warning first
        try {
            val nativeWarning = ZipLockNative.getPlatformCompatibilityWarning()
            if (nativeWarning != null) {
                return nativeWarning
            }
        } catch (e: Exception) {
            Log.w(TAG, "Native platform warning failed, falling back to Java detection", e)
        }

        // Fallback to Java-based detection
        return when {
            isX86Emulator() -> {
                "⚠️ Running on x86 emulator. Archive operations WILL crash due to sevenz_rust2 library issues. " +
                "Use real Android device for archive operations."
            }
            isArmEmulator() -> {
                "⚠️ Running on ARM emulator. Archive operations may crash due to sevenz_rust2 library issues. " +
                "Use real Android device for reliable archive operations."
            }
            isEmulator() -> {
                "⚠️ Running on Android emulator. Archive operations may crash due to native library compatibility. " +
                "Use real Android device for archive operations."
            }
            !isEmulator() -> {
                "✅ Running on real device. Full compatibility expected."
            }
            else -> {
                "ℹ️ Unknown platform type. Monitor for potential issues."
            }
        }
    }

    /**
     * Log detailed platform information for debugging
     */
    fun logPlatformInfo() {
        Log.i(TAG, "=== Platform Information ===")
        Log.i(TAG, "Device: ${Build.MANUFACTURER} ${Build.MODEL}")
        Log.i(TAG, "Android: ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})")
        Log.i(TAG, "Build: ${Build.FINGERPRINT}")
        Log.i(TAG, "Product: ${Build.PRODUCT}")
        Log.i(TAG, "Brand: ${Build.BRAND}")
        Log.i(TAG, "Device: ${Build.DEVICE}")
        Log.i(TAG, "Supported ABIs: ${getAllAbis().joinToString(", ")}")
        Log.i(TAG, "Primary ABI: ${getPrimaryAbi()}")
        Log.i(TAG, "Is Emulator (Java): ${isEmulator()}")
        Log.i(TAG, "Is Emulator (Native): ${isEmulatorNative()}")
        Log.i(TAG, "Is x86 Emulator: ${isX86Emulator()}")
        Log.i(TAG, "Is ARM Emulator: ${isArmEmulator()}")
        Log.i(TAG, "Has Known Archive Issues: ${hasKnownArchiveIssues()}")

        // Try to get native platform description
        try {
            val nativeDescription = ZipLockNative.getAndroidPlatformDescription()
            Log.i(TAG, "Platform (Native): $nativeDescription")
        } catch (e: Exception) {
            Log.w(TAG, "Native platform description failed", e)
        }

        getArchiveCompatibilityMessage()?.let { message ->
            Log.i(TAG, "Archive Compatibility: $message")
        }
        Log.i(TAG, "==========================")
    }

    /**
     * Get a user-friendly platform description
     */
    fun getPlatformDescription(): String {
        return when {
            isX86Emulator() -> "Android x86 Emulator (Incompatible)"
            isArmEmulator() -> "Android ARM Emulator (May Crash)"
            isEmulator() -> "Android Emulator (Archive Issues)"
            else -> "Android Device (${Build.MODEL})"
        }
    }

    /**
     * Check if current platform is recommended for archive operations
     */
    fun isRecommendedForArchiveOps(): Boolean {
        return !hasKnownArchiveIssues()
    }

    /**
     * Get emoji indicator for platform compatibility
     */
    fun getCompatibilityIndicator(): String {
        return when {
            isX86Emulator() -> "❌"
            isArmEmulator() -> "⚠️"
            isEmulator() -> "⚠️"
            !isEmulator() -> "✅"
            else -> "❓"
        }
    }
}
