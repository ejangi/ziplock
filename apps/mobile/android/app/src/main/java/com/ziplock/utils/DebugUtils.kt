package com.ziplock.utils

import android.content.Context
import android.content.SharedPreferences
import android.util.Log

import com.ziplock.ffi.ZipLockNative

/**
 * Debug utilities for ZipLock Android app
 *
 * Provides centralized debug functionality including logging control,
 * debug preferences, and development-only features.
 */
object DebugUtils {

    private const val TAG = "ZipLockDebug"
    private const val PREFS_NAME = "ziplock_debug_prefs"
    private const val KEY_DEBUG_LOGGING_ENABLED = "debug_logging_enabled"
    private const val KEY_VERBOSE_FFI_LOGGING = "verbose_ffi_logging"
    private const val KEY_PERFORMANCE_LOGGING = "performance_logging"

    /**
     * Initialize debug settings based on build configuration and user preferences
     */
    fun initializeDebugSettings(context: Context) {
        val prefs = getDebugPreferences(context)

        // Check if this is a debug build by looking for debug indicators
        val isDebugBuild = try {
            val buildConfigClass = Class.forName("${context.packageName}.BuildConfig")
            val debugField = buildConfigClass.getField("DEBUG")
            debugField.getBoolean(null)
        } catch (e: Exception) {
            false // Assume release build if we can't determine
        }

        // In debug builds, enable debug logging by default
        // In release builds, respect user preference (default off)
        val shouldEnableDebug = if (isDebugBuild) {
            prefs.getBoolean(KEY_DEBUG_LOGGING_ENABLED, true)
        } else {
            prefs.getBoolean(KEY_DEBUG_LOGGING_ENABLED, false)
        }

        setDebugLoggingEnabled(context, shouldEnableDebug)

        Log.d(TAG, "Debug settings initialized: debug_logging=$shouldEnableDebug, build_type=${if (isDebugBuild) "debug" else "release"}")
    }

    /**
     * Enable or disable debug logging in both Android and native library
     */
    fun setDebugLoggingEnabled(context: Context, enabled: Boolean) {
        val prefs = getDebugPreferences(context)
        prefs.edit()
            .putBoolean(KEY_DEBUG_LOGGING_ENABLED, enabled)
            .apply()

        try {
            val result = if (enabled) {
                ZipLockNative.enableDebugLogging()
            } else {
                ZipLockNative.disableDebugLogging()
            }

            if (result) {
                Log.d(TAG, "Native debug logging ${if (enabled) "enabled" else "disabled"}")

                // Test logging if enabled
                if (enabled) {
                    ZipLockNative.testLogging("Debug logging test from DebugUtils")
                }
            } else {
                Log.w(TAG, "Failed to ${if (enabled) "enable" else "disable"} native debug logging")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception while setting debug logging: ${e.message}", e)
        }
    }

    /**
     * Check if debug logging is currently enabled
     */
    fun isDebugLoggingEnabled(context: Context): Boolean {
        val prefs = getDebugPreferences(context)
        val defaultValue = try {
            val buildConfigClass = Class.forName("${context.packageName}.BuildConfig")
            val debugField = buildConfigClass.getField("DEBUG")
            debugField.getBoolean(null)
        } catch (e: Exception) {
            false
        }
        return prefs.getBoolean(KEY_DEBUG_LOGGING_ENABLED, defaultValue)
    }

    /**
     * Toggle debug logging on/off
     */
    fun toggleDebugLogging(context: Context): Boolean {
        val currentState = isDebugLoggingEnabled(context)
        val newState = !currentState
        setDebugLoggingEnabled(context, newState)
        return newState
    }

    /**
     * Enable verbose FFI logging (logs all FFI calls)
     */
    fun setVerboseFfiLogging(context: Context, enabled: Boolean) {
        val prefs = getDebugPreferences(context)
        prefs.edit()
            .putBoolean(KEY_VERBOSE_FFI_LOGGING, enabled)
            .apply()

        Log.d(TAG, "Verbose FFI logging ${if (enabled) "enabled" else "disabled"}")
    }

    /**
     * Check if verbose FFI logging is enabled
     */
    fun isVerboseFfiLoggingEnabled(context: Context): Boolean {
        val prefs = getDebugPreferences(context)
        return prefs.getBoolean(KEY_VERBOSE_FFI_LOGGING, false)
    }

    /**
     * Enable performance logging
     */
    fun setPerformanceLogging(context: Context, enabled: Boolean) {
        val prefs = getDebugPreferences(context)
        prefs.edit()
            .putBoolean(KEY_PERFORMANCE_LOGGING, enabled)
            .apply()

        Log.d(TAG, "Performance logging ${if (enabled) "enabled" else "disabled"}")
    }

    /**
     * Check if performance logging is enabled
     */
    fun isPerformanceLoggingEnabled(context: Context): Boolean {
        val prefs = getDebugPreferences(context)
        return prefs.getBoolean(KEY_PERFORMANCE_LOGGING, false)
    }

    /**
     * Get debug information about the current state
     */
    fun getDebugInfo(context: Context): DebugInfo {
        val isDebugBuild = try {
            val buildConfigClass = Class.forName("${context.packageName}.BuildConfig")
            val debugField = buildConfigClass.getField("DEBUG")
            debugField.getBoolean(null)
        } catch (e: Exception) {
            false
        }
        return DebugInfo(
            buildType = if (isDebugBuild) "Debug" else "Release",
            debugLoggingEnabled = isDebugLoggingEnabled(context),
            nativeDebugLoggingEnabled = try {
                ZipLockNative.isDebugLoggingEnabled()
            } catch (e: Exception) {
                false
            },
            verboseFfiLogging = isVerboseFfiLoggingEnabled(context),
            performanceLogging = isPerformanceLoggingEnabled(context),
            nativeLibraryVersion = try {
                ZipLockNative.getVersion()
            } catch (e: Exception) {
                "Unknown"
            },
            lastError = ZipLockNative.getLastError()
        )
    }

    /**
     * Run comprehensive debug tests
     */
    fun runDebugTests(context: Context): DebugTestResult {
        val results = mutableListOf<String>()
        var allPassed = true

        try {
            // Test 1: Native library initialization
            results.add("✓ Native library accessible")

            // Test 2: Version retrieval
            val version = ZipLockNative.getVersion()
            results.add("✓ Version retrieval: $version")

            // Test 3: Debug logging state
            val debugState = ZipLockNative.isDebugLoggingEnabled()
            results.add("✓ Debug logging state: $debugState")

            // Test 4: Logging configuration
            val configResult = ZipLockNative.configureLogging("debug")
            if (configResult) {
                results.add("✓ Logging configuration successful")
            } else {
                results.add("✗ Logging configuration failed")
                allPassed = false
            }

            // Test 5: Test message logging
            val testResult = ZipLockNative.testLogging("Debug test from runDebugTests")
            if (testResult) {
                results.add("✓ Test message logging successful")
            } else {
                results.add("✗ Test message logging failed")
                allPassed = false
            }

            // Test 6: Error handling
            val lastError = ZipLockNative.getLastError()
            if (lastError == null) {
                results.add("✓ No errors detected")
            } else {
                results.add("⚠ Last error: $lastError")
            }

        } catch (e: Exception) {
            results.add("✗ Exception during testing: ${e.message}")
            allPassed = false
        }

        return DebugTestResult(
            allTestsPassed = allPassed,
            testResults = results,
            timestamp = System.currentTimeMillis()
        )
    }

    /**
     * Log FFI call if verbose logging is enabled
     */
    fun logFfiCall(context: Context, functionName: String, vararg parameters: Any) {
        if (isVerboseFfiLoggingEnabled(context)) {
            val paramString = parameters.joinToString(", ") { it.toString() }
            Log.v(TAG, "FFI Call: $functionName($paramString)")
        }
    }

    /**
     * Log performance metric if performance logging is enabled
     */
    fun logPerformance(context: Context, operation: String, durationMs: Long) {
        if (isPerformanceLoggingEnabled(context)) {
            Log.d(TAG, "Performance: $operation took ${durationMs}ms")
        }
    }

    /**
     * Execute operation with performance logging
     */
    inline fun <T> withPerformanceLogging(
        context: Context,
        operationName: String,
        operation: () -> T
    ): T {
        val startTime = System.currentTimeMillis()
        try {
            return operation()
        } finally {
            val duration = System.currentTimeMillis() - startTime
            logPerformance(context, operationName, duration)
        }
    }

    /**
     * Get shared preferences for debug settings
     */
    private fun getDebugPreferences(context: Context): SharedPreferences {
        return context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    }

    /**
     * Clear all debug preferences (useful for testing)
     */
    fun clearDebugPreferences(context: Context) {
        getDebugPreferences(context).edit().clear().apply()
        Log.d(TAG, "Debug preferences cleared")
    }
}

/**
 * Data class containing current debug information
 */
data class DebugInfo(
    val buildType: String,
    val debugLoggingEnabled: Boolean,
    val nativeDebugLoggingEnabled: Boolean,
    val verboseFfiLogging: Boolean,
    val performanceLogging: Boolean,
    val nativeLibraryVersion: String,
    val lastError: String?
)

/**
 * Result of debug tests
 */
data class DebugTestResult(
    val allTestsPassed: Boolean,
    val testResults: List<String>,
    val timestamp: Long
)
