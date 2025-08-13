//
//  android-example.kt
//  ZipLock Mobile FFI Example
//
//  Example demonstrating how to use ZipLock's C API from Android Kotlin applications.
//  This file shows the complete integration pattern including error handling,
//  memory management, and proper Kotlin idioms.
//

package com.example.ziplock

import android.app.Application
import android.content.Context
import android.util.Log
import kotlinx.coroutines.*
import java.util.concurrent.ConcurrentHashMap

// MARK: - Error Types

sealed class ZipLockError : Exception() {
    object InitializationFailed : ZipLockError()
    object InvalidPointer : ZipLockError()
    object InvalidString : ZipLockError()
    data class FieldError(val message: String) : ZipLockError()
    data class ValidationFailed(val message: String) : ZipLockError()
    data class InternalError(val code: Int, val message: String = "") : ZipLockError()

    override val message: String
        get() = when (this) {
            is InitializationFailed -> "Failed to initialize ZipLock library"
            is InvalidPointer -> "Invalid pointer passed to ZipLock function"
            is InvalidString -> "Invalid string encoding"
            is FieldError -> "Field error: $message"
            is ValidationFailed -> "Validation failed: $message"
            is InternalError -> "Internal ZipLock error (code $code): $message"
        }

    companion object {
        fun fromCode(code: Int): ZipLockError = when (code) {
            -1 -> InvalidPointer
            -2 -> InvalidString
            -3 -> FieldError("Invalid field")
            -4 -> ValidationFailed("Validation failed")
            else -> InternalError(code)
        }
    }
}

// MARK: - Field Types

enum class ZipLockFieldType(val value: Int) {
    TEXT(0),
    PASSWORD(1),
    EMAIL(2),
    URL(3),
    USERNAME(4),
    PHONE(5),
    CREDIT_CARD_NUMBER(6),
    EXPIRY_DATE(7),
    CVV(8),
    TOTP_SECRET(9),
    TEXT_AREA(10),
    NUMBER(11),
    DATE(12),
    CUSTOM(13);

    val displayName: String
        get() = when (this) {
            TEXT -> "Text"
            PASSWORD -> "Password"
            EMAIL -> "Email"
            URL -> "URL"
            USERNAME -> "Username"
            PHONE -> "Phone"
            CREDIT_CARD_NUMBER -> "Credit Card"
            EXPIRY_DATE -> "Expiry Date"
            CVV -> "CVV"
            TOTP_SECRET -> "TOTP Secret"
            TEXT_AREA -> "Text Area"
            NUMBER -> "Number"
            DATE -> "Date"
            CUSTOM -> "Custom"
        }

    val isSensitiveByDefault: Boolean
        get() = when (this) {
            PASSWORD, CVV, TOTP_SECRET -> true
            else -> false
        }

    companion object {
        fun fromValue(value: Int): ZipLockFieldType? = values().find { it.value == value }
    }
}

// MARK: - Password Strength

data class PasswordStrength(
    val level: Level,
    val score: UInt,
    val description: String
) {
    enum class Level(val value: Int) {
        VERY_WEAK(0),
        WEAK(1),
        FAIR(2),
        GOOD(3),
        STRONG(4);

        val description: String
            get() = when (this) {
                VERY_WEAK -> "Very Weak"
                WEAK -> "Weak"
                FAIR -> "Fair"
                GOOD -> "Good"
                STRONG -> "Strong"
            }

        val color: String
            get() = when (this) {
                VERY_WEAK -> "#FF4444"
                WEAK -> "#FF8800"
                FAIR -> "#FFBB00"
                GOOD -> "#88BB00"
                STRONG -> "#44BB44"
            }

        companion object {
            fun fromValue(value: Int): Level? = values().find { it.value == value }
        }
    }
}

// MARK: - Native Interface

object ZipLockNative {
    private const val TAG = "ZipLockNative"

    init {
        try {
            System.loadLibrary("ziplock_shared")
            Log.d(TAG, "ZipLock native library loaded successfully")
        } catch (e: UnsatisfiedLinkError) {
            Log.e(TAG, "Failed to load ZipLock native library", e)
            throw ZipLockError.InitializationFailed
        }
    }

    // Library management
    external fun ziplock_init(): Int
    external fun ziplock_get_version(): String?
    external fun ziplock_get_last_error(): String?
    external fun ziplock_string_free(ptr: Long)

    // Credential management
    external fun ziplock_credential_new(title: String, type: String): Long
    external fun ziplock_credential_from_template(template: String, title: String): Long
    external fun ziplock_credential_free(handle: Long)
    external fun ziplock_credential_add_field(
        handle: Long,
        name: String,
        fieldType: Int,
        value: String,
        label: String?,
        sensitive: Int
    ): Int
    external fun ziplock_credential_get_field(handle: Long, name: String): String?
    external fun ziplock_credential_remove_field(handle: Long, name: String): Int
    external fun ziplock_credential_add_tag(handle: Long, tag: String): Int
    external fun ziplock_credential_remove_tag(handle: Long, tag: String): Int
    external fun ziplock_credential_has_tag(handle: Long, tag: String): Int
    external fun ziplock_credential_validate(handle: Long): Long

    // Password utilities
    external fun ziplock_password_generate(
        length: Int,
        includeUppercase: Int,
        includeLowercase: Int,
        includeNumbers: Int,
        includeSymbols: Int
    ): String?
    external fun ziplock_password_validate(password: String): Long
    external fun ziplock_password_strength_free(handle: Long)

    // Validation
    external fun ziplock_email_validate(email: String): Int
    external fun ziplock_url_validate(url: String): Int
    external fun ziplock_validation_result_free(handle: Long)

    // Utilities
    external fun ziplock_credit_card_format(cardNumber: String): String?
    external fun ziplock_totp_generate(secret: String, timeStep: Int): String?
    external fun ziplock_test_echo(input: String): String?

    // Debug
    external fun ziplock_debug_logging(enabled: Int): Int
}

// MARK: - Core Library Manager

class ZipLockCore private constructor() {
    companion object {
        @Volatile
        private var INSTANCE: ZipLockCore? = null

        fun getInstance(): ZipLockCore {
            return INSTANCE ?: synchronized(this) {
                INSTANCE ?: ZipLockCore().also { INSTANCE = it }
            }
        }
    }

    private val isInitialized: Boolean

    init {
        val result = ZipLockNative.ziplock_init()
        isInitialized = result == 0
        if (!isInitialized) {
            val error = ZipLockNative.ziplock_get_last_error() ?: "Unknown error"
            throw ZipLockError.InternalError(result, error)
        }
        Log.d("ZipLockCore", "ZipLock library initialized successfully")
    }

    val version: String
        get() = ZipLockNative.ziplock_get_version() ?: "Unknown"

    fun enableDebugLogging(enabled: Boolean) {
        ZipLockNative.ziplock_debug_logging(if (enabled) 1 else 0)
    }

    fun getLastError(): String? = ZipLockNative.ziplock_get_last_error()
}

// MARK: - Credential Management

class ZipLockCredential private constructor(private val handle: Long) : AutoCloseable {
    private var isClosed = false

    companion object {
        fun create(title: String, type: String): ZipLockCredential {
            val handle = ZipLockNative.ziplock_credential_new(title, type)
            if (handle == 0L) {
                throw ZipLockError.InternalError(-1, "Failed to create credential")
            }
            return ZipLockCredential(handle)
        }

        fun fromTemplate(template: String, title: String): ZipLockCredential {
            val handle = ZipLockNative.ziplock_credential_from_template(template, title)
            if (handle == 0L) {
                throw ZipLockError.InternalError(-1, "Failed to create credential from template")
            }
            return ZipLockCredential(handle)
        }
    }

    fun addField(
        name: String,
        type: ZipLockFieldType,
        value: String,
        label: String? = null,
        sensitive: Boolean? = null
    ) {
        checkNotClosed()
        val isSensitive = sensitive ?: type.isSensitiveByDefault
        val result = ZipLockNative.ziplock_credential_add_field(
            handle, name, type.value, value, label, if (isSensitive) 1 else 0
        )
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }

    fun getField(name: String): String? {
        checkNotClosed()
        return ZipLockNative.ziplock_credential_get_field(handle, name)
    }

    fun removeField(name: String) {
        checkNotClosed()
        val result = ZipLockNative.ziplock_credential_remove_field(handle, name)
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }

    fun addTag(tag: String) {
        checkNotClosed()
        val result = ZipLockNative.ziplock_credential_add_tag(handle, tag)
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }

    fun removeTag(tag: String) {
        checkNotClosed()
        val result = ZipLockNative.ziplock_credential_remove_tag(handle, tag)
        if (result != 0) {
            throw ZipLockError.fromCode(result)
        }
    }

    fun hasTag(tag: String): Boolean {
        checkNotClosed()
        return ZipLockNative.ziplock_credential_has_tag(handle, tag) == 1
    }

    fun validate() {
        checkNotClosed()
        val validationHandle = ZipLockNative.ziplock_credential_validate(handle)
        if (validationHandle == 0L) {
            throw ZipLockError.InternalError(-1, "Failed to validate credential")
        }

        // Note: In a real implementation, you would parse the validation result
        // For now, we just free the handle
        ZipLockNative.ziplock_validation_result_free(validationHandle)
    }

    private fun checkNotClosed() {
        if (isClosed) {
            throw IllegalStateException("Credential has been closed")
        }
    }

    override fun close() {
        if (!isClosed) {
            ZipLockNative.ziplock_credential_free(handle)
            isClosed = true
        }
    }
}

// MARK: - Password Utilities

object ZipLockPassword {
    fun generate(
        length: Int = 16,
        includeUppercase: Boolean = true,
        includeLowercase: Boolean = true,
        includeNumbers: Boolean = true,
        includeSymbols: Boolean = true
    ): String? {
        return ZipLockNative.ziplock_password_generate(
            length,
            if (includeUppercase) 1 else 0,
            if (includeLowercase) 1 else 0,
            if (includeNumbers) 1 else 0,
            if (includeSymbols) 1 else 0
        )
    }

    fun validate(password: String): PasswordStrength? {
        val handle = ZipLockNative.ziplock_password_validate(password)
        if (handle == 0L) return null

        // Note: In a real implementation, you would parse the C struct
        // For this example, we'll return a mock result
        ZipLockNative.ziplock_password_strength_free(handle)

        // Mock implementation - in reality, you'd parse the actual result
        val score = when {
            password.length < 8 -> 20u
            password.length < 12 -> 40u
            password.any { it.isDigit() } && password.any { it.isLetter() } -> 80u
            else -> 60u
        }

        val level = when (score.toInt()) {
            in 0..20 -> PasswordStrength.Level.VERY_WEAK
            in 21..40 -> PasswordStrength.Level.WEAK
            in 41..60 -> PasswordStrength.Level.FAIR
            in 61..80 -> PasswordStrength.Level.GOOD
            else -> PasswordStrength.Level.STRONG
        }

        return PasswordStrength(level, score, level.description)
    }
}

// MARK: - Validation Utilities

object ZipLockValidation {
    fun isValidEmail(email: String): Boolean {
        return ZipLockNative.ziplock_email_validate(email) == 1
    }

    fun isValidURL(url: String): Boolean {
        return ZipLockNative.ziplock_url_validate(url) == 1
    }
}

// MARK: - Utility Functions

object ZipLockUtils {
    fun formatCreditCard(cardNumber: String): String? {
        return ZipLockNative.ziplock_credit_card_format(cardNumber)
    }

    fun generateTOTP(secret: String, timeStep: Int = 30): String? {
        return ZipLockNative.ziplock_totp_generate(secret, timeStep)
    }

    fun testEcho(input: String): String? {
        return ZipLockNative.ziplock_test_echo(input)
    }
}

// MARK: - Example Usage

class ZipLockExample {
    companion object {
        private const val TAG = "ZipLockExample"

        fun runExamples() {
            Log.d(TAG, "ZipLock Android FFI Example")
            Log.d(TAG, "===========================")
            Log.d(TAG, "Library Version: ${ZipLockCore.getInstance().version}")

            // Test basic functionality
            testBasicFunctionality()

            // Test credential management
            testCredentialManagement()

            // Test password utilities
            testPasswordUtilities()

            // Test validation
            testValidation()

            // Test utility functions
            testUtilities()
        }

        private fun testBasicFunctionality() {
            Log.d(TAG, "1. Testing Basic Functionality")
            Log.d(TAG, "------------------------------")

            // Test echo function
            val echo = ZipLockUtils.testEcho("Hello, ZipLock!")
            if (echo != null) {
                Log.d(TAG, "✓ Echo test: $echo")
            } else {
                Log.e(TAG, "✗ Echo test failed")
            }
        }

        private fun testCredentialManagement() {
            Log.d(TAG, "2. Testing Credential Management")
            Log.d(TAG, "--------------------------------")

            try {
                ZipLockCredential.create("Example Login", "login").use { credential ->
                    Log.d(TAG, "✓ Created credential")

                    // Add fields
                    credential.addField("username", ZipLockFieldType.USERNAME, "user@example.com")
                    credential.addField("password", ZipLockFieldType.PASSWORD, "SuperSecure123!")
                    credential.addField("website", ZipLockFieldType.URL, "https://example.com")
                    Log.d(TAG, "✓ Added fields")

                    // Add tags
                    credential.addTag("work")
                    credential.addTag("important")
                    Log.d(TAG, "✓ Added tags")

                    // Retrieve field values
                    val username = credential.getField("username")
                    if (username != null) {
                        Log.d(TAG, "✓ Retrieved username: $username")
                    }

                    // Check tags
                    if (credential.hasTag("work")) {
                        Log.d(TAG, "✓ Has 'work' tag")
                    }

                    // Validate credential
                    credential.validate()
                    Log.d(TAG, "✓ Credential validation passed")
                }
            } catch (e: ZipLockError) {
                Log.e(TAG, "✗ Credential management test failed: ${e.message}")
            } catch (e: Exception) {
                Log.e(TAG, "✗ Unexpected error in credential management test", e)
            }
        }

        private fun testPasswordUtilities() {
            Log.d(TAG, "3. Testing Password Utilities")
            Log.d(TAG, "-----------------------------")

            // Generate password
            val password = ZipLockPassword.generate(length = 12, includeSymbols = false)
            if (password != null) {
                Log.d(TAG, "✓ Generated password: $password")

                // Validate password strength
                val strength = ZipLockPassword.validate(password)
                if (strength != null) {
                    Log.d(TAG, "✓ Password strength: ${strength.level.description} (Score: ${strength.score})")
                } else {
                    Log.e(TAG, "✗ Password strength validation failed")
                }
            } else {
                Log.e(TAG, "✗ Password generation failed")
            }

            // Test with a known weak password
            val weakStrength = ZipLockPassword.validate("123456")
            if (weakStrength != null) {
                Log.d(TAG, "✓ Weak password strength: ${weakStrength.level.description} (Score: ${weakStrength.score})")
            }
        }

        private fun testValidation() {
            Log.d(TAG, "4. Testing Validation")
            Log.d(TAG, "--------------------")

            // Test email validation
            val emails = listOf(
                "user@example.com" to true,
                "invalid-email" to false,
                "test@domain.co.uk" to true,
                "@invalid.com" to false
            )

            for ((email, expected) in emails) {
                val isValid = ZipLockValidation.isValidEmail(email)
                val status = if (isValid == expected) "✓" else "✗"
                Log.d(TAG, "$status Email '$email': ${if (isValid) "valid" else "invalid"}")
            }

            // Test URL validation
            val urls = listOf(
                "https://example.com" to true,
                "http://test.org" to true,
                "not-a-url" to false,
                "ftp://files.com" to false
            )

            for ((url, expected) in urls) {
                val isValid = ZipLockValidation.isValidURL(url)
                val status = if (isValid == expected) "✓" else "✗"
                Log.d(TAG, "$status URL '$url': ${if (isValid) "valid" else "invalid"}")
            }
        }

        private fun testUtilities() {
            Log.d(TAG, "5. Testing Utility Functions")
            Log.d(TAG, "----------------------------")

            // Test credit card formatting
            val cardNumbers = listOf(
                "1234567890123456",
                "4111-1111-1111-1111",
                "1234"
            )

            for (cardNumber in cardNumbers) {
                val formatted = ZipLockUtils.formatCreditCard(cardNumber)
                if (formatted != null) {
                    Log.d(TAG, "✓ Credit card '$cardNumber' formatted as: $formatted")
                } else {
                    Log.e(TAG, "✗ Failed to format credit card: $cardNumber")
                }
            }

            // Test TOTP generation (with example secret)
            val totpSecret = "JBSWY3DPEHPK3PXP"  // Example base32 secret
            val totp = ZipLockUtils.generateTOTP(totpSecret)
            if (totp != null) {
                Log.d(TAG, "✓ Generated TOTP: $totp")
            } else {
                Log.e(TAG, "✗ TOTP generation failed")
            }
        }
    }
}

// MARK: - Template Helper Functions

object ZipLockTemplates {
    fun createLoginCredential(
        title: String,
        username: String,
        password: String,
        website: String
    ): ZipLockCredential {
        val credential = ZipLockCredential.fromTemplate("login", title)
        credential.addField("username", ZipLockFieldType.USERNAME, username)
        credential.addField("password", ZipLockFieldType.PASSWORD, password)
        credential.addField("website", ZipLockFieldType.URL, website)
        return credential
    }

    fun createCreditCardCredential(
        title: String,
        cardNumber: String,
        expiryDate: String,
        cvv: String,
        cardholderName: String
    ): ZipLockCredential {
        val credential = ZipLockCredential.fromTemplate("credit_card", title)
        credential.addField("card_number", ZipLockFieldType.CREDIT_CARD_NUMBER, cardNumber)
        credential.addField("expiry_date", ZipLockFieldType.EXPIRY_DATE, expiryDate)
        credential.addField("cvv", ZipLockFieldType.CVV, cvv)
        credential.addField("cardholder_name", ZipLockFieldType.TEXT, cardholderName)
        return credential
    }

    fun createSecureNoteCredential(title: String, content: String): ZipLockCredential {
        val credential = ZipLockCredential.fromTemplate("secure_note", title)
        credential.addField("content", ZipLockFieldType.TEXT_AREA, content)
        return credential
    }
}

// MARK: - Coroutine Extensions

object ZipLockAsync {
    suspend fun generatePasswordAsync(
        length: Int = 16,
        includeUppercase: Boolean = true,
        includeLowercase: Boolean = true,
        includeNumbers: Boolean = true,
        includeSymbols: Boolean = true
    ): String? = withContext(Dispatchers.IO) {
        ZipLockPassword.generate(length, includeUppercase, includeLowercase, includeNumbers, includeSymbols)
    }

    suspend fun validatePasswordAsync(password: String): PasswordStrength? = withContext(Dispatchers.IO) {
        ZipLockPassword.validate(password)
    }

    suspend fun validateEmailAsync(email: String): Boolean = withContext(Dispatchers.IO) {
        ZipLockValidation.isValidEmail(email)
    }

    suspend fun validateURLAsync(url: String): Boolean = withContext(Dispatchers.IO) {
        ZipLockValidation.isValidURL(url)
    }
}

// MARK: - Android Application Class

class ZipLockApplication : Application() {
    override fun onCreate() {
        super.onCreate()

        try {
            // Initialize ZipLock Core
            ZipLockCore.getInstance()
            Log.d("ZipLockApplication", "ZipLock library initialized successfully")

            // Enable debug logging in debug builds
            if (BuildConfig.DEBUG) {
                ZipLockCore.getInstance().enableDebugLogging(true)
            }

            // Run examples
            ZipLockExample.runExamples()

        } catch (e: ZipLockError) {
            Log.e("ZipLockApplication", "Failed to initialize ZipLock: ${e.message}", e)
        }
    }
}

// MARK: - Android Activity Example

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            ZipLockExampleTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    ZipLockExampleScreen()
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ZipLockExampleScreen() {
    var password by remember { mutableStateOf("") }
    var passwordStrength by remember { mutableStateOf<PasswordStrength?>(null) }
    var email by remember { mutableStateOf("") }
    var isEmailValid by remember { mutableStateOf(false) }
    var generatedPassword by remember { mutableStateOf("") }

    val scope = rememberCoroutineScope()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Header
        Card(
            modifier = Modifier.fillMaxWidth()
        ) {
            Column(
                modifier = Modifier.padding(16.dp)
            ) {
                Text(
                    text = "ZipLock FFI Demo",
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = "Version: ${ZipLockCore.getInstance().version}",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }

        // Password Testing Section
        Card(
            modifier = Modifier.fillMaxWidth()
        ) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Text(
                    text = "Password Testing",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                OutlinedTextField(
                    value = password,
                    onValueChange = { newPassword ->
                        password = newPassword
                        scope.launch {
                            passwordStrength = ZipLockAsync.validatePasswordAsync(newPassword)
                        }
                    },
                    label = { Text("Enter password") },
                    visualTransformation = PasswordVisualTransformation(),
                    modifier = Modifier.fillMaxWidth()
                )

                passwordStrength?.let { strength ->
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = "Strength: ${strength.level.description}",
                            color = Color(android.graphics.Color.parseColor(strength.level.color))
                        )
                        Text(
                            text = "${strength.score}/100",
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                }

                Button(
                    onClick = {
                        scope.launch {
                            val generated = ZipLockAsync.generatePasswordAsync()
                            if (generated != null) {
                                generatedPassword = generated
                                password = generated
                            }
                        }
                    },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text("Generate Password")
                }

                if (generatedPassword.isNotEmpty()) {
                    Text(
                        text = "Generated: $generatedPassword",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }

        // Email Validation Section
        Card(
            modifier = Modifier.fillMaxWidth()
        ) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Text(
                    text = "Email Validation",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                OutlinedTextField(
                    value = email,
                    onValueChange = { newEmail ->
                        email = newEmail
                        scope.launch {
                            isEmailValid = ZipLockAsync.validateEmailAsync(newEmail)
                        }
                    },
                    label = { Text("Enter email") },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                    modifier = Modifier.fillMaxWidth()
                )

                Row(
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Valid: ")
                    Text(
                        text = if (isEmailValid) "✓" else "✗",
                        color = if (isEmailValid) Color.Green else Color.Red,
                        fontWeight = FontWeight.Bold
                    )
                }
            }
        }

        // Test Section
        Card(
            modifier = Modifier.fillMaxWidth()
        ) {
            Column(
                modifier = Modifier.padding(16.dp)
            ) {
                Text(
                    text = "Test Functions",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )

                Spacer(modifier = Modifier.height(8.dp))

                Button(
                    onClick = {
                        scope.launch {
                            ZipLockExample.runExamples()
                        }
                    },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text("Run All Tests")
                }
            }
        }
    }
}

@Composable
fun ZipLockExampleTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        content = content
    )
}

@Preview(showBackground = true)
@Composable
fun ZipLockExampleScreenPreview() {
    ZipLockExampleTheme {
        ZipLockExampleScreen()
    }
}
