package com.ziplock.utils

import android.util.Log

/**
 * Utility class to test XZ library availability and diagnose ClassNotFoundException issues
 */
object XZTestUtils {
    private const val TAG = "XZTestUtils"

    /**
     * Test if XZ library classes are available at runtime
     */
    fun testXZAvailability(): TestResult {
        val results = mutableListOf<String>()
        var success = true

        // Test 1: Check if LZMA2Options class is available
        try {
            val clazz = Class.forName("org.tukaani.xz.LZMA2Options")
            results.add("✅ LZMA2Options class found: ${clazz.name}")
            Log.d(TAG, "LZMA2Options class loaded successfully")
        } catch (e: ClassNotFoundException) {
            results.add("❌ LZMA2Options class NOT found: ${e.message}")
            Log.e(TAG, "LZMA2Options class not found", e)
            success = false
        }

        // Test 2: Check if LZMAInputStream is available
        try {
            val clazz = Class.forName("org.tukaani.xz.LZMAInputStream")
            results.add("✅ LZMAInputStream class found: ${clazz.name}")
            Log.d(TAG, "LZMAInputStream class loaded successfully")
        } catch (e: ClassNotFoundException) {
            results.add("❌ LZMAInputStream class NOT found: ${e.message}")
            Log.e(TAG, "LZMAInputStream class not found", e)
            success = false
        }

        // Test 3: Check if XZInputStream is available
        try {
            val clazz = Class.forName("org.tukaani.xz.XZInputStream")
            results.add("✅ XZInputStream class found: ${clazz.name}")
            Log.d(TAG, "XZInputStream class loaded successfully")
        } catch (e: ClassNotFoundException) {
            results.add("❌ XZInputStream class NOT found: ${e.message}")
            Log.e(TAG, "XZInputStream class not found", e)
            success = false
        }

        // Test 4: Try to instantiate LZMA2Options
        if (success) {
            try {
                val constructor = Class.forName("org.tukaani.xz.LZMA2Options").getDeclaredConstructor()
                val instance = constructor.newInstance()
                results.add("✅ LZMA2Options instantiation successful")
                Log.d(TAG, "LZMA2Options created successfully: $instance")
            } catch (e: Exception) {
                results.add("❌ LZMA2Options instantiation failed: ${e.message}")
                Log.e(TAG, "LZMA2Options instantiation failed", e)
                success = false
            }
        }

        return TestResult(success, results)
    }

    /**
     * Test Apache Commons Compress classes availability
     */
    fun testCommonsCompressAvailability(): TestResult {
        val results = mutableListOf<String>()
        var success = true

        // Test 1: Check if SevenZFile is available
        try {
            val clazz = Class.forName("org.apache.commons.compress.archivers.sevenz.SevenZFile")
            results.add("✅ SevenZFile class found: ${clazz.name}")
            Log.d(TAG, "SevenZFile class loaded successfully")
        } catch (e: ClassNotFoundException) {
            results.add("❌ SevenZFile class NOT found: ${e.message}")
            Log.e(TAG, "SevenZFile class not found", e)
            success = false
        }

        // Test 2: Check if LZMADecoder is available
        try {
            val clazz = Class.forName("org.apache.commons.compress.archivers.sevenz.LZMADecoder")
            results.add("✅ LZMADecoder class found: ${clazz.name}")
            Log.d(TAG, "LZMADecoder class loaded successfully")
        } catch (e: ClassNotFoundException) {
            results.add("❌ LZMADecoder class NOT found: ${e.message}")
            Log.e(TAG, "LZMADecoder class not found", e)
            success = false
        }

        // Test 3: Check if Coders is available
        try {
            val clazz = Class.forName("org.apache.commons.compress.archivers.sevenz.Coders")
            results.add("✅ Coders class found: ${clazz.name}")
            Log.d(TAG, "Coders class loaded successfully")
        } catch (e: ClassNotFoundException) {
            results.add("❌ Coders class NOT found: ${e.message}")
            Log.e(TAG, "Coders class not found", e)
            success = false
        }

        return TestResult(success, results)
    }

    /**
     * Comprehensive dependency test
     */
    fun runComprehensiveTest(): ComprehensiveTestResult {
        Log.i(TAG, "Starting comprehensive XZ and Commons Compress dependency test...")

        val xzResult = testXZAvailability()
        val commonsResult = testCommonsCompressAvailability()

        val overallSuccess = xzResult.success && commonsResult.success

        Log.i(TAG, "Test completed. Overall success: $overallSuccess")

        return ComprehensiveTestResult(
            overallSuccess = overallSuccess,
            xzTest = xzResult,
            commonsCompressTest = commonsResult
        )
    }

    /**
     * Get detailed class loader information for debugging
     */
    fun getClassLoaderInfo(): String {
        val sb = StringBuilder()
        val classLoader = XZTestUtils::class.java.classLoader

        sb.appendLine("=== ClassLoader Debug Info ===")
        sb.appendLine("Current ClassLoader: ${classLoader?.javaClass?.name}")
        sb.appendLine("Parent ClassLoader: ${classLoader?.parent?.javaClass?.name}")

        // Try to get system class path
        try {
            val systemClassPath = System.getProperty("java.class.path")
            sb.appendLine("System ClassPath: $systemClassPath")
        } catch (e: Exception) {
            sb.appendLine("Could not get system class path: ${e.message}")
        }

        // Check if we can access the specific problematic class
        try {
            val resource = classLoader?.getResource("org/tukaani/xz/LZMA2Options.class")
            if (resource != null) {
                sb.appendLine("✅ LZMA2Options.class resource found at: $resource")
            } else {
                sb.appendLine("❌ LZMA2Options.class resource NOT found")
            }
        } catch (e: Exception) {
            sb.appendLine("Error checking LZMA2Options.class resource: ${e.message}")
        }

        sb.appendLine("==============================")

        val result = sb.toString()
        Log.d(TAG, result)
        return result
    }

    data class TestResult(
        val success: Boolean,
        val details: List<String>
    )

    data class ComprehensiveTestResult(
        val overallSuccess: Boolean,
        val xzTest: TestResult,
        val commonsCompressTest: TestResult
    ) {
        fun getFormattedReport(): String {
            val sb = StringBuilder()

            sb.appendLine("=== Dependency Test Report ===")
            sb.appendLine("Overall Status: ${if (overallSuccess) "✅ PASS" else "❌ FAIL"}")
            sb.appendLine("")

            sb.appendLine("XZ Library Test:")
            xzTest.details.forEach { sb.appendLine("  $it") }
            sb.appendLine("")

            sb.appendLine("Commons Compress Test:")
            commonsCompressTest.details.forEach { sb.appendLine("  $it") }
            sb.appendLine("")

            if (!overallSuccess) {
                sb.appendLine("Recommendations:")
                if (!xzTest.success) {
                    sb.appendLine("  • Add org.tukaani:xz dependency to build.gradle")
                    sb.appendLine("  • Check ProGuard rules for XZ library exclusion")
                }
                if (!commonsCompressTest.success) {
                    sb.appendLine("  • Verify org.apache.commons:commons-compress dependency")
                    sb.appendLine("  • Check for dependency conflicts")
                }
            }

            sb.appendLine("===============================")

            return sb.toString()
        }
    }
}
