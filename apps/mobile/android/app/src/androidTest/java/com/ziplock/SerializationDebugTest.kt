package com.ziplock

import android.util.Log
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.ziplock.ffi.ZipLockMobileFFI
import kotlinx.serialization.encodeToString
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import org.junit.Test
import org.junit.runner.RunWith
import org.junit.Assert.*

/**
 * Test to debug and verify the serialization fix for credential save issue
 */
@RunWith(AndroidJUnit4::class)
class SerializationDebugTest {

    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
        explicitNulls = false
    }

    @Test
    fun testCredentialRecordSerialization() {
        // Create a credential record that matches the Rust structure exactly
        val currentTime = System.currentTimeMillis()
        val credential = ZipLockMobileFFI.CredentialRecord(
            id = "test-id-123",
            title = "Test Login",
            credentialType = "login",
            fields = mapOf(
                "username" to ZipLockMobileFFI.CredentialField(
                    value = "testuser",
                    fieldType = ZipLockMobileFFI.FieldType.Username,
                    sensitive = false,
                    label = null,
                    metadata = emptyMap()
                ),
                "password" to ZipLockMobileFFI.CredentialField(
                    value = "testpass123",
                    fieldType = ZipLockMobileFFI.FieldType.Password,
                    sensitive = true,
                    label = null,
                    metadata = emptyMap()
                )
            ),
            tags = listOf("test"),
            notes = null,
            createdAt = currentTime,
            updatedAt = currentTime,
            accessedAt = currentTime,
            favorite = false,
            folderPath = null
        )

        // Test serialization to JSON
        var jsonString: String? = null
        try {
            jsonString = json.encodeToString(credential)
            Log.d("SerializationDebugTest", "Serialized JSON: $jsonString")
            println("✅ Serialization successful!")
            println("JSON Length: ${jsonString.length}")
            println("JSON Content: $jsonString")
        } catch (e: Exception) {
            println("❌ Serialization failed: ${e.message}")
            e.printStackTrace()
            fail("Serialization failed: ${e.message}")
        }

        assertNotNull("JSON should not be null", jsonString)
        assertTrue("JSON should not be empty", jsonString!!.isNotEmpty())
        assertTrue("JSON should contain credential_type", jsonString.contains("credential_type"))
        assertTrue("JSON should contain field_type", jsonString.contains("field_type"))
        assertTrue("JSON should contain updated_at", jsonString.contains("updated_at"))

        // Test deserialization back
        try {
            val parsed = json.decodeFromString<ZipLockMobileFFI.CredentialRecord>(jsonString)
            assertEquals("Title should match", credential.title, parsed.title)
            assertEquals("Credential type should match", credential.credentialType, parsed.credentialType)
            assertEquals("Field count should match", credential.fields.size, parsed.fields.size)
            println("✅ Deserialization successful!")
        } catch (e: Exception) {
            println("❌ Deserialization failed: ${e.message}")
            e.printStackTrace()
            fail("Deserialization failed: ${e.message}")
        }
    }

    @Test
    fun testFieldTypeSerialization() {
        // Test that each FieldType enum value serializes correctly
        val testCases = mapOf(
            ZipLockMobileFFI.FieldType.Text to "Text",
            ZipLockMobileFFI.FieldType.Password to "Password",
            ZipLockMobileFFI.FieldType.Email to "Email",
            ZipLockMobileFFI.FieldType.Username to "Username",
            ZipLockMobileFFI.FieldType.Url to "Url"
        )

        testCases.forEach { (fieldType, expectedString) ->
            val field = ZipLockMobileFFI.CredentialField(
                value = "test",
                fieldType = fieldType,
                sensitive = false,
                label = null,
                metadata = emptyMap()
            )

            val jsonString = json.encodeToString(field)
            println("FieldType $fieldType serialized as: $jsonString")
            assertTrue("JSON should contain expected string", jsonString.contains(expectedString))

            // Verify it can be deserialized back
            try {
                val parsed = json.decodeFromString<ZipLockMobileFFI.CredentialField>(jsonString)
                assertEquals("Field type should match", fieldType, parsed.fieldType)
                println("✅ $fieldType: serialization roundtrip successful")
            } catch (e: Exception) {
                println("❌ $fieldType: serialization roundtrip failed - ${e.message}")
                fail("Failed to serialize/deserialize $fieldType: ${e.message}")
            }
        }
    }

    @Test
    fun testDirectFFICredentialCreation() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext

        // Create FFI instance and initialize
        val ffi = ZipLockMobileFFI()
        val initResult = ffi.initializeRepository()
        assertTrue("FFI should initialize", initResult)

        // Create credential matching expected format
        val currentTime = System.currentTimeMillis()
        val credential = ZipLockMobileFFI.CredentialRecord(
            id = "", // Let FFI generate ID
            title = "Test Login via FFI",
            credentialType = "login",
            fields = mapOf(
                "username" to ZipLockMobileFFI.CredentialField(
                    value = "ffi_user",
                    fieldType = ZipLockMobileFFI.FieldType.Username,
                    sensitive = false,
                    label = null,
                    metadata = emptyMap()
                ),
                "password" to ZipLockMobileFFI.CredentialField(
                    value = "ffi_pass123",
                    fieldType = ZipLockMobileFFI.FieldType.Password,
                    sensitive = true,
                    label = null,
                    metadata = emptyMap()
                )
            ),
            tags = listOf("test", "ffi-direct"),
            notes = null,
            createdAt = currentTime,
            updatedAt = currentTime,
            accessedAt = currentTime,
            favorite = false,
            folderPath = null
        )

        println("About to test credential addition via FFI...")

        // First, let's see what JSON would be sent
        val jsonString = json.encodeToString(credential)
        println("JSON that will be sent to FFI:")
        println(jsonString)

        // Now try adding via FFI
        val addResult = ffi.addCredential(credential)
        if (addResult) {
            println("✅ FFI credential addition successful!")

            // Try to list credentials to verify
            val credentials = ffi.listCredentials()
            println("Total credentials after add: ${credentials.size}")
            credentials.forEach { cred ->
                println("- ${cred.title} (${cred.credentialType})")
            }
        } else {
            println("❌ FFI credential addition failed!")
            // This would be the error we're trying to debug
        }

        assertTrue("Credential should be added successfully via FFI", addResult)
    }
}
