package com.ziplock.ffi

import android.util.Log
import kotlinx.serialization.Serializable

/**
 * Field template alias for compatibility
 */
typealias FieldTemplate = ZipLockNativeHelper.TemplateField

/**
 * ZipLockNativeHelper - Compatibility Helper Class
 *
 * This class provides helper methods and data structures that were previously
 * part of the legacy architecture. It serves as a compatibility layer while
 * the codebase transitions to the unified architecture.
 *
 * This class provides:
 * - Credential template definitions
 * - Helper methods for credential operations
 * - Compatibility wrappers for legacy code
 */
object ZipLockNativeHelper {

    private const val TAG = "ZipLockNativeHelper"

    /**
     * Credential template data structure
     */
    @Serializable
    data class CredentialTemplate(
        val id: String,
        val name: String,
        val credentialType: String,
        val description: String = "",
        val fields: List<TemplateField> = emptyList(),
        val category: String = "general"
    )

    /**
     * Template field definition
     */
    @Serializable
    data class TemplateField(
        val id: String,
        val name: String,
        val fieldType: String,
        val required: Boolean = false,
        val sensitive: Boolean = false,
        val placeholder: String = "",
        val validation: String? = null,
        val options: List<String> = emptyList(),
        val label: String = name
    )

    /**
     * Predefined credential templates
     */
    private val predefinedTemplates = listOf(
        CredentialTemplate(
            id = "login",
            name = "Login",
            credentialType = "login",
            description = "Website or application login credentials",
            category = "web",
            fields = listOf(
                TemplateField("username", "Username", "text", required = false, placeholder = "Enter username"),
                TemplateField("password", "Password", "password", required = false, sensitive = true, placeholder = "Enter password"),
                TemplateField("url", "Website URL", "url", placeholder = "https://example.com"),
                TemplateField("notes", "Notes", "textarea", placeholder = "Additional notes")
            )
        ),
        CredentialTemplate(
            id = "credit_card",
            name = "Credit Card",
            credentialType = "credit_card",
            description = "Credit card information",
            category = "finance",
            fields = listOf(
                TemplateField("cardholder", "Cardholder Name", "text", required = true, placeholder = "Name on card"),
                TemplateField("number", "Card Number", "text", required = true, sensitive = true, placeholder = "1234 5678 9012 3456"),
                TemplateField("expiry", "Expiry Date", "text", required = true, placeholder = "MM/YY"),
                TemplateField("cvv", "CVV", "text", required = true, sensitive = true, placeholder = "123"),
                TemplateField("notes", "Notes", "textarea", placeholder = "Additional notes")
            )
        ),
        CredentialTemplate(
            id = "secure_note",
            name = "Secure Note",
            credentialType = "secure_note",
            description = "Encrypted note or document",
            category = "documents",
            fields = listOf(
                TemplateField("title", "Title", "text", required = true, placeholder = "Note title"),
                TemplateField("content", "Content", "textarea", required = true, sensitive = true, placeholder = "Note content")
            )
        ),
        CredentialTemplate(
            id = "bank_account",
            name = "Bank Account",
            credentialType = "bank_account",
            description = "Bank account information",
            category = "finance",
            fields = listOf(
                TemplateField("bank_name", "Bank Name", "text", required = true, placeholder = "Bank name"),
                TemplateField("account_holder", "Account Holder", "text", required = true, placeholder = "Account holder name"),
                TemplateField("account_number", "Account Number", "text", required = true, sensitive = true, placeholder = "Account number"),
                TemplateField("routing_number", "Routing Number", "text", placeholder = "Routing number"),
                TemplateField("notes", "Notes", "textarea", placeholder = "Additional notes")
            )
        ),
        CredentialTemplate(
            id = "wifi_password",
            name = "WiFi Password",
            credentialType = "wifi",
            description = "WiFi network credentials",
            category = "network",
            fields = listOf(
                TemplateField("ssid", "Network Name (SSID)", "text", required = true, placeholder = "WiFi network name"),
                TemplateField("password", "Password", "password", required = false, sensitive = true, placeholder = "WiFi password"),
                TemplateField("security", "Security Type", "select", options = listOf("WPA2", "WPA3", "WEP", "Open")),
                TemplateField("notes", "Notes", "textarea", placeholder = "Additional notes")
            )
        ),
        CredentialTemplate(
            id = "identity",
            name = "Identity",
            credentialType = "identity",
            description = "Personal identity information",
            category = "personal",
            fields = listOf(
                TemplateField("first_name", "First Name", "text", required = true, placeholder = "First name"),
                TemplateField("last_name", "Last Name", "text", required = true, placeholder = "Last name"),
                TemplateField("email", "Email", "email", placeholder = "email@example.com"),
                TemplateField("phone", "Phone", "tel", placeholder = "Phone number"),
                TemplateField("address", "Address", "textarea", placeholder = "Full address"),
                TemplateField("notes", "Notes", "textarea", placeholder = "Additional notes")
            )
        )
    )

    /**
     * Get all available credential templates
     */
    fun getAvailableTemplates(): List<CredentialTemplate> {
        return predefinedTemplates
    }

    /**
     * Get a specific template by ID
     */
    fun getTemplate(templateId: String): CredentialTemplate? {
        return predefinedTemplates.find { it.id == templateId }
    }

    /**
     * Get template for a specific credential type
     */
    fun getTemplateForType(credentialType: String): CredentialTemplate {
        return predefinedTemplates.find { it.credentialType == credentialType }
            ?: createGenericTemplate(credentialType)
    }

    /**
     * Create a generic template for unknown credential types
     */
    private fun createGenericTemplate(credentialType: String): CredentialTemplate {
        return CredentialTemplate(
            id = "generic_$credentialType",
            name = "Generic ${credentialType.capitalize()}",
            credentialType = credentialType,
            description = "Generic template for $credentialType credentials",
            fields = listOf(
                TemplateField("title", "Title", "text", required = true, placeholder = "Credential title"),
                TemplateField("value", "Value", "text", sensitive = true, placeholder = "Credential value"),
                TemplateField("notes", "Notes", "textarea", placeholder = "Additional notes")
            )
        )
    }

    /**
     * Validate credential data against template
     */
    fun validateCredential(template: CredentialTemplate, fields: Map<String, String>): ValidationResult {
        val errors = mutableListOf<String>()

        // Check required fields
        template.fields.filter { it.required }.forEach { templateField ->
            val value = fields[templateField.id]
            if (value.isNullOrBlank()) {
                errors.add("${templateField.name} is required")
            }
        }

        // Field-specific validation
        template.fields.forEach { templateField ->
            val value = fields[templateField.id]
            if (!value.isNullOrBlank()) {
                when (templateField.fieldType) {
                    "email" -> {
                        if (!android.util.Patterns.EMAIL_ADDRESS.matcher(value).matches()) {
                            errors.add("${templateField.name} must be a valid email address")
                        }
                    }
                    "url" -> {
                        if (!android.util.Patterns.WEB_URL.matcher(value).matches()) {
                            errors.add("${templateField.name} must be a valid URL")
                        }
                    }
                }
            }
        }

        return ValidationResult(
            isValid = errors.isEmpty(),
            errors = errors
        )
    }

    /**
     * Convert template field to credential field format
     */
    fun templateFieldToCredentialField(templateField: TemplateField, value: String): ZipLockNative.FieldValue {
        return ZipLockNative.FieldValue(
            value = value,
            fieldType = templateField.fieldType,
            label = templateField.name,
            sensitive = templateField.sensitive
        )
    }

    /**
     * Create credential from template and field values
     */
    fun createCredentialFromTemplate(
        template: CredentialTemplate,
        title: String,
        fieldValues: Map<String, String>,
        tags: List<String> = emptyList()
    ): ZipLockNative.Credential {
        val fields = mutableMapOf<String, ZipLockNative.FieldValue>()

        // Convert template fields to credential fields
        template.fields.forEach { templateField ->
            val value = fieldValues[templateField.id] ?: ""
            if (value.isNotBlank() || templateField.required) {
                fields[templateField.id] = templateFieldToCredentialField(templateField, value)
            }
        }

        return ZipLockNative.Credential(
            id = java.util.UUID.randomUUID().toString(),
            title = title,
            credentialType = template.credentialType,
            fields = fields,
            createdAt = System.currentTimeMillis(),
            updatedAt = System.currentTimeMillis(),
            tags = tags
        )
    }

    /**
     * Validation result data class
     */
    data class ValidationResult(
        val isValid: Boolean,
        val errors: List<String>
    )

    /**
     * Get templates by category
     */
    fun getTemplatesByCategory(category: String): List<CredentialTemplate> {
        return predefinedTemplates.filter { it.category == category }
    }

    /**
     * Get all available categories
     */
    fun getAvailableCategories(): List<String> {
        return predefinedTemplates.map { it.category }.distinct().sorted()
    }

    /**
     * Helper method to generate secure passwords
     */
    fun generateSecurePassword(
        length: Int = 16,
        includeUppercase: Boolean = true,
        includeLowercase: Boolean = true,
        includeNumbers: Boolean = true,
        includeSymbols: Boolean = true
    ): String {
        val uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        val lowercase = "abcdefghijklmnopqrstuvwxyz"
        val numbers = "0123456789"
        val symbols = "!@#$%^&*()_+-=[]{}|;:,.<>?"

        val characters = StringBuilder()
        if (includeUppercase) characters.append(uppercase)
        if (includeLowercase) characters.append(lowercase)
        if (includeNumbers) characters.append(numbers)
        if (includeSymbols) characters.append(symbols)

        if (characters.isEmpty()) {
            return ""
        }

        val random = java.security.SecureRandom()
        return (1..length)
            .map { characters[random.nextInt(characters.length)] }
            .joinToString("")
    }

    /**
     * Log helper method
     */
    private fun logDebug(message: String) {
        Log.d(TAG, message)
    }

    /**
     * Log error helper method
     */
    private fun logError(message: String, throwable: Throwable? = null) {
        Log.e(TAG, message, throwable)
    }

    /**
     * Default tags for credentials
     */
    val defaultTags: List<String> = emptyList()

    // Note: typealias moved to top level for compatibility
}
