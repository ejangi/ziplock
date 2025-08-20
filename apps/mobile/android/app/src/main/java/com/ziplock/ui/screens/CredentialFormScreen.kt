package com.ziplock.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusDirection
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.ziplock.ffi.ZipLockNative
import com.ziplock.ffi.ZipLockNativeHelper
import com.ziplock.ui.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CredentialFormScreen(
    template: ZipLockNativeHelper.CredentialTemplate,
    existingCredential: ZipLockNative.Credential? = null,
    onSave: (title: String, fields: Map<String, String>, tags: List<String>) -> Unit,
    onCancel: () -> Unit,
    isSaving: Boolean = false,
    errorMessage: String? = null,
    modifier: Modifier = Modifier
) {
    // Form state
    var title by remember { mutableStateOf(existingCredential?.title ?: "") }
    var fieldValues by remember {
        mutableStateOf<Map<String, String>>(
            template.fields.associate { field: ZipLockNativeHelper.FieldTemplate ->
                field.name to (getExistingFieldValue(existingCredential, field.name) ?: field.defaultValue ?: "")
            }
        )
    }
    var tags by remember {
        mutableStateOf<String>(
            existingCredential?.tags?.joinToString(", ") ?: template.defaultTags.joinToString(", ")
        )
    }

    // Validation state
    val isFormValid = remember(title, fieldValues) {
        title.isNotBlank() && template.fields.filter { field: ZipLockNativeHelper.FieldTemplate -> field.required }.all { field: ZipLockNativeHelper.FieldTemplate ->
            val value = fieldValues[field.name]
            !value.isNullOrBlank()
        }
    }

    val focusManager = LocalFocusManager.current
    val isEditing = existingCredential != null

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(ZipLockColors.LightBackground)
    ) {
        // Header
        CredentialFormHeader(
            title = if (isEditing) "Edit ${formatTemplateName(template.name)}" else "New ${formatTemplateName(template.name)}",
            onCancel = onCancel,
            onSave = {
                if (isFormValid && !isSaving) {
                    val tagsList = tags.split(",").map { it.trim() }.filter { it.isNotBlank() }
                    onSave(title, fieldValues, tagsList)
                }
            },
            canSave = isFormValid && !isSaving,
            isSaving = isSaving,
            modifier = Modifier.fillMaxWidth()
        )

        // Error message
        errorMessage?.let { error ->
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = ZipLockSpacing.Standard),
                colors = CardDefaults.cardColors(containerColor = ZipLockColors.ErrorRed.copy(alpha = 0.1f)),
                shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(ZipLockSpacing.Standard),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = ZipLockIcons.ErrorCircle,
                        contentDescription = "Error",
                        tint = ZipLockColors.ErrorRed,
                        modifier = Modifier.size(20.dp)
                    )
                    Spacer(modifier = Modifier.width(ZipLockSpacing.Small))
                    Text(
                        text = error,
                        style = ZipLockTypography.Normal,
                        color = ZipLockColors.ErrorRed
                    )
                }
            }
            Spacer(modifier = Modifier.height(ZipLockSpacing.Small))
        }

        // Form content
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(
                horizontal = ZipLockSpacing.Standard,
                vertical = ZipLockSpacing.Small
            ),
            verticalArrangement = Arrangement.spacedBy(ZipLockSpacing.Standard)
        ) {
            // Title field
            item {
                Card(
                    colors = CardDefaults.cardColors(containerColor = ZipLockColors.White),
                    elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
                    shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(ZipLockSpacing.Standard)
                    ) {
                        Text(
                            text = "Title *",
                            style = ZipLockTypography.Medium,
                            color = ZipLockColors.DarkText,
                            fontWeight = FontWeight.Medium
                        )

                        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

                        ZipLockTextInput(
                            value = title,
                            onValueChange = { title = it },
                            placeholder = "Enter a title for this credential",
                            imeAction = ImeAction.Next,
                            keyboardActions = KeyboardActions(
                                onNext = { focusManager.moveFocus(FocusDirection.Down) }
                            ),
                            modifier = Modifier.fillMaxWidth()
                        )
                    }
                }
            }

            // Template fields
            items(template.fields) { field ->
                CredentialFieldInput(
                    field = field,
                    value = fieldValues[field.name] ?: "",
                    onValueChange = { newValue ->
                        fieldValues = fieldValues + (field.name to newValue)
                    },
                    isLastField = field == template.fields.last(),
                    onNext = { focusManager.moveFocus(FocusDirection.Down) },
                    onDone = { focusManager.clearFocus() }
                )
            }

            // Tags field
            item {
                Card(
                    colors = CardDefaults.cardColors(containerColor = ZipLockColors.White),
                    elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
                    shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(ZipLockSpacing.Standard)
                    ) {
                        Text(
                            text = "Tags",
                            style = ZipLockTypography.Medium,
                            color = ZipLockColors.DarkText,
                            fontWeight = FontWeight.Medium
                        )

                        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

                        Text(
                            text = "Separate multiple tags with commas",
                            style = ZipLockTypography.Small,
                            color = ZipLockColors.LightGrayText
                        )

                        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

                        ZipLockTextInput(
                            value = tags,
                            onValueChange = { tags = it },
                            placeholder = "e.g., work, personal, important",
                            imeAction = ImeAction.Done,
                            keyboardActions = KeyboardActions(
                                onDone = { focusManager.clearFocus() }
                            ),
                            modifier = Modifier.fillMaxWidth()
                        )
                    }
                }
            }

            // Bottom padding
            item {
                Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CredentialFormHeader(
    title: String,
    onCancel: () -> Unit,
    onSave: () -> Unit,
    canSave: Boolean,
    isSaving: Boolean,
    modifier: Modifier = Modifier
) {
    Surface(
        modifier = modifier,
        color = ZipLockColors.White,
        shadowElevation = 2.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(ZipLockSpacing.Standard),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Cancel button
            TextButton(onClick = onCancel) {
                Text(
                    text = "Cancel",
                    style = ZipLockTypography.Medium,
                    color = ZipLockColors.LightGrayText
                )
            }

            // Title
            Text(
                text = title,
                style = ZipLockTypography.Header,
                color = ZipLockColors.DarkText,
                fontWeight = FontWeight.SemiBold,
                textAlign = TextAlign.Center,
                modifier = Modifier.weight(1f)
            )

            // Save button
            if (isSaving) {
                CircularProgressIndicator(
                    color = ZipLockColors.LogoPurple,
                    modifier = Modifier.size(24.dp),
                    strokeWidth = 2.dp
                )
            } else {
                TextButton(
                    onClick = onSave,
                    enabled = canSave
                ) {
                    Text(
                        text = "Save",
                        style = ZipLockTypography.Medium,
                        color = if (canSave) ZipLockColors.LogoPurple else ZipLockColors.LightGrayText,
                        fontWeight = FontWeight.Medium
                    )
                }
            }
        }
    }
}

@Composable
private fun CredentialFieldInput(
    field: ZipLockNativeHelper.FieldTemplate,
    value: String,
    onValueChange: (String) -> Unit,
    isLastField: Boolean,
    onNext: () -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier,
        colors = CardDefaults.cardColors(containerColor = ZipLockColors.White),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
        shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(ZipLockSpacing.Standard)
        ) {
            // Field label
            Row(
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = field.label,
                    style = ZipLockTypography.Medium,
                    color = ZipLockColors.DarkText,
                    fontWeight = FontWeight.Medium
                )

                if (field.required) {
                    Text(
                        text = " *",
                        style = ZipLockTypography.Medium,
                        color = ZipLockColors.ErrorRed,
                        fontWeight = FontWeight.Medium
                    )
                }
            }

            Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

            // Field input
            ZipLockTextInput(
                value = value,
                onValueChange = onValueChange,
                placeholder = getFieldPlaceholder(field),
                isPassword = field.sensitive && field.fieldType.lowercase() != "textarea",
                singleLine = field.fieldType.lowercase() != "textarea",
                keyboardType = getKeyboardType(field.fieldType),
                imeAction = if (isLastField) ImeAction.Done else ImeAction.Next,
                keyboardActions = KeyboardActions(
                    onNext = { onNext() },
                    onDone = { onDone() }
                ),
                modifier = Modifier.fillMaxWidth()
            )

            // Required field validation
            if (field.required && value.isBlank()) {
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "This field is required",
                    style = ZipLockTypography.Small,
                    color = ZipLockColors.ErrorRed
                )
            }
        }
    }
}

/**
 * Get placeholder text for a field
 */
private fun getFieldPlaceholder(field: ZipLockNativeHelper.FieldTemplate): String {
    return when (field.fieldType.lowercase()) {
        "email" -> "example@domain.com"
        "url" -> "https://example.com"
        "phone" -> "+1 (555) 123-4567"
        "username" -> "Enter username"
        "password" -> "Enter password"
        "text" -> "Enter ${field.label.lowercase()}"
        "textarea" -> "Enter your ${field.label.lowercase()} here..."
        "number" -> "Enter number"
        "date" -> "YYYY-MM-DD"
        else -> "Enter ${field.label.lowercase()}"
    }
}

/**
 * Get keyboard type for field type
 */
private fun getKeyboardType(fieldType: String): KeyboardType {
    return when (fieldType.lowercase()) {
        "email" -> KeyboardType.Email
        "url" -> KeyboardType.Uri
        "phone" -> KeyboardType.Phone
        "number" -> KeyboardType.Number
        "password" -> KeyboardType.Password
        else -> KeyboardType.Text
    }
}

/**
 * Get existing field value from credential
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
 * Format template name for display
 */
private fun formatTemplateName(name: String): String {
    return name.split("_")
        .joinToString(" ") { word ->
            word.replaceFirstChar {
                if (it.isLowerCase()) it.titlecase() else it.toString()
            }
        }
}
