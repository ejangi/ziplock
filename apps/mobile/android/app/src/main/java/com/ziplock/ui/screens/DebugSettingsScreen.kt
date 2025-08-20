package com.ziplock.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.BugReport
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

import com.ziplock.ffi.ZipLockNative
import com.ziplock.ui.theme.ZipLockColors
import com.ziplock.utils.DebugUtils

/**
 * Debug settings screen for ZipLock Android app
 *
 * Provides controls for:
 * - Enabling/disabling debug logging
 * - Testing the logging system
 * - Viewing debug information
 * - Running debug tests
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DebugSettingsScreen(
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current

    // State for debug settings
    var debugLoggingEnabled by remember { mutableStateOf(DebugUtils.isDebugLoggingEnabled(context)) }
    var verboseFfiLogging by remember { mutableStateOf(DebugUtils.isVerboseFfiLoggingEnabled(context)) }
    var performanceLogging by remember { mutableStateOf(DebugUtils.isPerformanceLoggingEnabled(context)) }

    // State for debug info and tests
    var debugInfo by remember { mutableStateOf<com.ziplock.utils.DebugInfo?>(null) }
    var testResult by remember { mutableStateOf<com.ziplock.utils.DebugTestResult?>(null) }
    var isRunningTests by remember { mutableStateOf(false) }

    // Load debug info on first composition
    LaunchedEffect(Unit) {
        debugInfo = DebugUtils.getDebugInfo(context)
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "Debug Settings",
                        color = ZipLockColors.DarkText
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.Default.ArrowBack,
                            contentDescription = "Back",
                            tint = ZipLockColors.LogoPurple
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = ZipLockColors.LightBackground
                )
            )
        },
        containerColor = ZipLockColors.LightBackground
    ) { paddingValues ->
        Column(
            modifier = modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp)
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Warning for debug builds only
            val isDebugBuild = try {
                val buildConfigClass = Class.forName("${context.packageName}.BuildConfig")
                val debugField = buildConfigClass.getField("DEBUG")
                debugField.getBoolean(null)
            } catch (e: Exception) {
                false
            }

            if (!isDebugBuild) {
                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = ZipLockColors.ErrorRed.copy(alpha = 0.1f)
                    ),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Row(
                        modifier = Modifier.padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            imageVector = Icons.Default.Info,
                            contentDescription = null,
                            tint = ZipLockColors.ErrorRed
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = "Debug features are limited in release builds",
                            color = ZipLockColors.ErrorRed,
                            style = MaterialTheme.typography.bodyMedium
                        )
                    }
                }
            }

            // Logging Controls Section
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(
                    containerColor = ZipLockColors.White
                )
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            imageVector = Icons.Default.Settings,
                            contentDescription = null,
                            tint = ZipLockColors.LogoPurple
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = "Logging Controls",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                            color = ZipLockColors.DarkText
                        )
                    }

                    Spacer(modifier = Modifier.height(16.dp))

                    // Debug Logging Toggle
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "Debug Logging",
                                style = MaterialTheme.typography.bodyLarge,
                                color = ZipLockColors.DarkText
                            )
                            Text(
                                text = "Enable detailed logging from native library",
                                style = MaterialTheme.typography.bodySmall,
                                color = ZipLockColors.LightGrayText
                            )
                        }
                        Switch(
                            checked = debugLoggingEnabled,
                            onCheckedChange = { enabled ->
                                debugLoggingEnabled = enabled
                                DebugUtils.setDebugLoggingEnabled(context, enabled)
                                // Refresh debug info
                                debugInfo = DebugUtils.getDebugInfo(context)
                            },
                            colors = SwitchDefaults.colors(
                                checkedThumbColor = ZipLockColors.LogoPurple,
                                checkedTrackColor = ZipLockColors.LogoPurpleLight
                            )
                        )
                    }

                    Divider(
                        modifier = Modifier.padding(vertical = 12.dp),
                        color = ZipLockColors.LightGrayText
                    )

                    // Verbose FFI Logging Toggle
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "Verbose FFI Logging",
                                style = MaterialTheme.typography.bodyLarge,
                                color = ZipLockColors.DarkText
                            )
                            Text(
                                text = "Log all FFI function calls (very verbose)",
                                style = MaterialTheme.typography.bodySmall,
                                color = ZipLockColors.LightGrayText
                            )
                        }
                        Switch(
                            checked = verboseFfiLogging,
                            onCheckedChange = { enabled ->
                                verboseFfiLogging = enabled
                                DebugUtils.setVerboseFfiLogging(context, enabled)
                            },
                            colors = SwitchDefaults.colors(
                                checkedThumbColor = ZipLockColors.LogoPurple,
                                checkedTrackColor = ZipLockColors.LogoPurpleLight
                            )
                        )
                    }

                    Divider(
                        modifier = Modifier.padding(vertical = 12.dp),
                        color = ZipLockColors.LightGrayText
                    )

                    // Performance Logging Toggle
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "Performance Logging",
                                style = MaterialTheme.typography.bodyLarge,
                                color = ZipLockColors.DarkText
                            )
                            Text(
                                text = "Log performance metrics and timing",
                                style = MaterialTheme.typography.bodySmall,
                                color = ZipLockColors.LightGrayText
                            )
                        }
                        Switch(
                            checked = performanceLogging,
                            onCheckedChange = { enabled ->
                                performanceLogging = enabled
                                DebugUtils.setPerformanceLogging(context, enabled)
                            },
                            colors = SwitchDefaults.colors(
                                checkedThumbColor = ZipLockColors.LogoPurple,
                                checkedTrackColor = ZipLockColors.LogoPurpleLight
                            )
                        )
                    }
                }
            }

            // Test Actions Section
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(
                    containerColor = ZipLockColors.White
                )
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            imageVector = Icons.Default.BugReport,
                            contentDescription = null,
                            tint = ZipLockColors.LogoPurple
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = "Test Actions",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                            color = ZipLockColors.DarkText
                        )
                    }

                    Spacer(modifier = Modifier.height(16.dp))

                    // Test Logging Button
                    OutlinedButton(
                        onClick = {
                            ZipLockNative.testLogging("Manual test from Debug Settings")
                        },
                        modifier = Modifier.fillMaxWidth(),
                        colors = ButtonDefaults.outlinedButtonColors(
                            contentColor = ZipLockColors.LogoPurple
                        )
                    ) {
                        Icon(
                            imageVector = Icons.Default.PlayArrow,
                            contentDescription = null
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Test Logging System")
                    }

                    Spacer(modifier = Modifier.height(8.dp))

                    // Run Debug Tests Button
                    Button(
                        onClick = {
                            isRunningTests = true
                            // Run tests in a coroutine to avoid blocking UI
                            Thread {
                                try {
                                    testResult = DebugUtils.runDebugTests(context)
                                } finally {
                                    isRunningTests = false
                                }
                            }.start()
                        },
                        modifier = Modifier.fillMaxWidth(),
                        enabled = !isRunningTests,
                        colors = ButtonDefaults.buttonColors(
                            containerColor = ZipLockColors.LogoPurple
                        )
                    ) {
                        if (isRunningTests) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(16.dp),
                                color = ZipLockColors.White,
                                strokeWidth = 2.dp
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("Running Tests...")
                        } else {
                            Icon(
                                imageVector = Icons.Default.PlayArrow,
                                contentDescription = null
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("Run Debug Tests")
                        }
                    }
                }
            }

            // Debug Information Section
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(
                    containerColor = ZipLockColors.White
                )
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            imageVector = Icons.Default.Info,
                            contentDescription = null,
                            tint = ZipLockColors.LogoPurple
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = "Debug Information",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                            color = ZipLockColors.DarkText
                        )
                    }

                    Spacer(modifier = Modifier.height(16.dp))

                    debugInfo?.let { info ->
                        Column(
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            DebugInfoRow("Build Type", info.buildType)
                            DebugInfoRow("Android Debug Logging", if (info.debugLoggingEnabled) "Enabled" else "Disabled")
                            DebugInfoRow("Native Debug Logging", if (info.nativeDebugLoggingEnabled) "Enabled" else "Disabled")
                            DebugInfoRow("Verbose FFI Logging", if (info.verboseFfiLogging) "Enabled" else "Disabled")
                            DebugInfoRow("Performance Logging", if (info.performanceLogging) "Enabled" else "Disabled")
                            DebugInfoRow("Native Library Version", info.nativeLibraryVersion)

                            if (info.lastError != null) {
                                Spacer(modifier = Modifier.height(8.dp))
                                Text(
                                    text = "Last Error:",
                                    style = MaterialTheme.typography.bodyMedium,
                                    fontWeight = FontWeight.Bold,
                                    color = ZipLockColors.ErrorRed
                                )
                                Text(
                                    text = info.lastError,
                                    style = MaterialTheme.typography.bodySmall,
                                    fontFamily = FontFamily.Monospace,
                                    color = ZipLockColors.ErrorRed
                                )
                            }
                        }
                    }

                    Spacer(modifier = Modifier.height(16.dp))

                    OutlinedButton(
                        onClick = {
                            debugInfo = DebugUtils.getDebugInfo(context)
                        },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text("Refresh Debug Info")
                    }
                }
            }

            // Test Results Section
            testResult?.let { result ->
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = if (result.allTestsPassed) {
                            ZipLockColors.SuccessGreen.copy(alpha = 0.1f)
                        } else {
                            ZipLockColors.ErrorRed.copy(alpha = 0.1f)
                        }
                    )
                ) {
                    Column(
                        modifier = Modifier.padding(16.dp)
                    ) {
                        Text(
                            text = "Test Results",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                            color = if (result.allTestsPassed) {
                                ZipLockColors.SuccessGreen
                            } else {
                                ZipLockColors.ErrorRed
                            }
                        )

                        Spacer(modifier = Modifier.height(8.dp))

                        Text(
                            text = "Overall: ${if (result.allTestsPassed) "PASSED" else "FAILED"}",
                            style = MaterialTheme.typography.bodyMedium,
                            fontWeight = FontWeight.Bold,
                            color = if (result.allTestsPassed) {
                                ZipLockColors.SuccessGreen
                            } else {
                                ZipLockColors.ErrorRed
                            }
                        )

                        Spacer(modifier = Modifier.height(8.dp))

                        result.testResults.forEach { testResult ->
                            Text(
                                text = testResult,
                                style = MaterialTheme.typography.bodySmall,
                                fontFamily = FontFamily.Monospace,
                                color = ZipLockColors.DarkText
                            )
                        }

                        Spacer(modifier = Modifier.height(8.dp))

                        Text(
                            text = "Completed: ${java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss", java.util.Locale.getDefault()).format(java.util.Date(result.timestamp))}",
                            style = MaterialTheme.typography.bodySmall,
                            color = ZipLockColors.LightGrayText
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun DebugInfoRow(
    label: String,
    value: String,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium,
            color = ZipLockColors.LightGrayText,
            modifier = Modifier.weight(1f)
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
            fontFamily = FontFamily.Monospace,
            color = ZipLockColors.DarkText,
            textAlign = TextAlign.End,
            modifier = Modifier.weight(1f)
        )
    }
}
