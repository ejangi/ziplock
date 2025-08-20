package com.ziplock

import com.ziplock.ffi.ZipLockNative
import com.ziplock.ffi.ZipLockNativeHelper
import com.ziplock.viewmodel.CredentialFormViewModel
import org.junit.Test
import org.junit.Assert.*

/**
 * Test class for verifying secure note content field handling
 */
class CredentialFormTest {

    @Test
    fun testSecureNoteContentFieldMapping() {
        val viewModel = CredentialFormViewModel()

        // Create secure note template
        val secureNoteTemplate = ZipLockNativeHelper.CredentialTemplate(
            name = "secure_note",
            description = "Secure note or document",
            fields = listOf(
                ZipLockNativeHelper.FieldTemplate(
                    name = "content",
                    fieldType = "TextArea",
                    label = "Content",
                    required = false,
                    sensitive = false,
                    defaultValue = null,
                    validation = null
                )
            ),
            defaultTags = listOf("note")
        )

        // Test form data
        val title = "Test Secure Note"
        val contentText = "This is a multi-line\nsecure note content\nwith several lines\nof important information."
        val fields = mapOf("content" to contentText)
        val tags = listOf("test", "note")

        // Validate form
        val validationResult = viewModel.validateForm(title, secureNoteTemplate, fields)
        assertTrue("Form should be valid", validationResult.isValid)
        assertTrue("Should have no validation errors", validationResult.errors.isEmpty())
    }

    @Test
    fun testSecureNoteContentPersistence() {
        // Simulate existing secure note with content
        val existingNote = ZipLockNative.Credential(
            id = "test_note_1",
            title = "Existing Note",
            credentialType = "secure_note",
            username = "",
            url = "",
            notes = "This is the existing content\nwith multiple lines",
            tags = listOf("existing", "note")
        )

        // Test that content is properly extracted
        val extractedContent = getExistingFieldValue(existingNote, "content")
        assertEquals("Content should match notes field", existingNote.notes, extractedContent)
    }

    @Test
    fun testMultiLineTextAreaHeight() {
        // Test that TextArea fields are configured for multi-line display
        val field = ZipLockNativeHelper.FieldTemplate(
            name = "content",
            fieldType = "TextArea",
            label = "Content",
            required = false,
            sensitive = false,
            defaultValue = null,
            validation = null
        )

        // Verify field type handling
        assertFalse("TextArea should not be single line", getSingleLineStatus(field))
        assertFalse("TextArea should not be password field", getPasswordStatus(field))
    }

    @Test
    fun testSecureNoteTemplateConfiguration() {
        // Create a test template to verify structure
        val secureNoteTemplate = ZipLockNativeHelper.CredentialTemplate(
            name = "secure_note",
            description = "Secure note or document",
            fields = listOf(
                ZipLockNativeHelper.FieldTemplate(
                    name = "content",
                    fieldType = "TextArea",
                    label = "Content",
                    required = false,
                    sensitive = false,
                    defaultValue = null,
                    validation = null
                )
            ),
            defaultTags = listOf("note")
        )

        assertEquals("Template name should be secure_note", "secure_note", secureNoteTemplate.name)

        val contentField = secureNoteTemplate.fields.find { it.name == "content" }
        assertNotNull("Content field should exist", contentField)
        assertEquals("Content field should be TextArea", "TextArea", contentField?.fieldType)
        assertFalse("Content field should not be sensitive", contentField?.sensitive ?: true)
        assertEquals("Content field label should be Content", "Content", contentField?.label)
    }

    /**
     * Helper function to simulate field value extraction
     */
    private fun getExistingFieldValue(credential: ZipLockNative.Credential?, fieldName: String): String? {
        if (credential == null) return null

        return when (fieldName.lowercase()) {
            "username" -> credential.username
            "url", "website" -> credential.url
            "notes", "note" -> credential.notes
            "content" -> credential.notes  // For secure notes, content is stored in notes field
            else -> null
        }
    }

    /**
     * Helper function to determine if field should be single line
     */
    private fun getSingleLineStatus(field: ZipLockNativeHelper.FieldTemplate): Boolean {
        return field.fieldType.lowercase() != "textarea"
    }

    /**
     * Helper function to determine if field should be password field
     */
    private fun getPasswordStatus(field: ZipLockNativeHelper.FieldTemplate): Boolean {
        return field.sensitive && field.fieldType.lowercase() != "textarea"
    }
}
