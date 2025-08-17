package com.ziplock.ui.theme

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp

/**
 * ZipLock Button Styles
 * Provides consistent button styling across the app
 */
enum class ZipLockButtonStyle {
    Primary,
    Secondary,
    Destructive,
    Disabled,
    PasswordToggle
}

/**
 * ZipLock Text Input Styles
 * Provides consistent text input styling
 */
enum class ZipLockTextInputStyle {
    Standard,
    Valid,
    Invalid,
    Neutral,
    Title
}

/**
 * Custom ZipLock Button
 * Matches the styling from Linux theme.rs button styles
 */
@Composable
fun ZipLockButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    style: ZipLockButtonStyle = ZipLockButtonStyle.Primary,
    enabled: Boolean = true,
    icon: ImageVector? = null,
    contentPadding: PaddingValues = PaddingValues(ZipLockSpacing.ButtonPadding)
) {
    val colors = getButtonColors(style, enabled)
    val shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)

    Button(
        onClick = onClick,
        modifier = modifier.heightIn(min = ZipLockDimensions.MinButtonHeight),
        enabled = enabled,
        colors = ButtonDefaults.buttonColors(
            containerColor = colors.background,
            contentColor = colors.content,
            disabledContainerColor = colors.disabledBackground,
            disabledContentColor = colors.disabledContent
        ),
        shape = shape,
        contentPadding = contentPadding,
        border = colors.border?.let { BorderStroke(1.dp, it) }
    ) {
        Row(
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically
        ) {
            icon?.let {
                Icon(
                    imageVector = it,
                    contentDescription = null,
                    modifier = Modifier.size(ZipLockDimensions.IconSize)
                )
                Spacer(modifier = Modifier.width(ZipLockSpacing.Small))
            }
            Text(
                text = text,
                style = ZipLockTypography.Medium
            )
        }
    }
}

/**
 * Custom ZipLock Text Input
 * Matches the styling from Linux theme.rs text input styles
 */
@Composable
fun ZipLockTextInput(
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    style: ZipLockTextInputStyle = ZipLockTextInputStyle.Standard,
    placeholder: String = "",
    isPassword: Boolean = false,
    enabled: Boolean = true,
    singleLine: Boolean = true,
    keyboardType: KeyboardType = KeyboardType.Text,
    imeAction: ImeAction = ImeAction.Next,
    keyboardActions: KeyboardActions = KeyboardActions.Default,
    leadingIcon: ImageVector? = null,
    trailingIcon: @Composable (() -> Unit)? = null
) {
    val colors = getTextInputColors(style, enabled)
    val shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)

    var passwordVisible by remember { mutableStateOf(!isPassword) }

    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        modifier = modifier
            .fillMaxWidth()
            .heightIn(min = ZipLockDimensions.TextInputHeight),
        enabled = enabled,
        placeholder = {
            Text(
                text = placeholder,
                style = ZipLockTypography.TextInput,
                color = colors.placeholder
            )
        },
        textStyle = ZipLockTypography.TextInput,
        singleLine = singleLine,
        visualTransformation = if (isPassword && !passwordVisible) {
            PasswordVisualTransformation()
        } else {
            VisualTransformation.None
        },
        keyboardOptions = KeyboardOptions(
            keyboardType = if (isPassword) KeyboardType.Password else keyboardType,
            imeAction = imeAction
        ),
        keyboardActions = keyboardActions,
        leadingIcon = leadingIcon?.let { icon ->
            {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    tint = colors.icon,
                    modifier = Modifier.size(ZipLockDimensions.IconSize)
                )
            }
        },
        trailingIcon = if (isPassword) {
            {
                ZipLockPasswordToggle(
                    visible = passwordVisible,
                    onToggle = { passwordVisible = !passwordVisible }
                )
            }
        } else {
            trailingIcon
        },
        colors = OutlinedTextFieldDefaults.colors(
            focusedTextColor = colors.text,
            unfocusedTextColor = colors.text,
            disabledTextColor = colors.disabledText,
            focusedBorderColor = colors.focusedBorder,
            unfocusedBorderColor = colors.unfocusedBorder,
            disabledBorderColor = colors.disabledBorder,
            focusedContainerColor = colors.background,
            unfocusedContainerColor = colors.background,
            disabledContainerColor = colors.disabledBackground
        ),
        shape = shape
    )
}

/**
 * Password visibility toggle button
 * Matches the Linux theme.rs password toggle implementation
 */
@Composable
fun ZipLockPasswordToggle(
    visible: Boolean,
    onToggle: () -> Unit,
    modifier: Modifier = Modifier
) {
    IconButton(
        onClick = onToggle,
        modifier = modifier.size(ZipLockDimensions.IconSize + ZipLockSpacing.Small)
    ) {
        Icon(
            imageVector = if (visible) ZipLockIcons.EyeOff else ZipLockIcons.Eye,
            contentDescription = if (visible) "Hide password" else "Show password",
            tint = ZipLockColors.MediumGray,
            modifier = Modifier.size(ZipLockDimensions.SmallIconSize)
        )
    }
}

/**
 * Alert Component
 * Matches the Linux theme.rs alert system
 */
@Composable
fun ZipLockAlert(
    level: AlertLevel,
    title: String? = null,
    message: String,
    dismissible: Boolean = true,
    onDismiss: (() -> Unit)? = null,
    modifier: Modifier = Modifier
) {
    val colors = getAlertColors(level)
    val icon = getAlertIcon(level)
    val shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)

    Card(
        modifier = modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = colors.background),
        shape = shape,
        border = BorderStroke(1.dp, colors.border)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(ZipLockSpacing.AlertPadding),
            verticalAlignment = Alignment.Top
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = colors.icon,
                modifier = Modifier.size(ZipLockDimensions.IconSize)
            )

            Spacer(modifier = Modifier.width(ZipLockSpacing.Medium))

            Column(
                modifier = Modifier.weight(1f)
            ) {
                title?.let {
                    Text(
                        text = it,
                        style = ZipLockTypography.Medium.copy(
                            color = colors.text
                        )
                    )
                    Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraSmall))
                }

                Text(
                    text = message,
                    style = ZipLockTypography.Normal.copy(
                        color = colors.text
                    )
                )
            }

            if (dismissible && onDismiss != null) {
                Spacer(modifier = Modifier.width(ZipLockSpacing.Small))
                IconButton(
                    onClick = onDismiss,
                    modifier = Modifier.size(ZipLockDimensions.IconSize)
                ) {
                    Icon(
                        imageVector = ZipLockIcons.Close,
                        contentDescription = "Dismiss",
                        tint = colors.icon,
                        modifier = Modifier.size(ZipLockDimensions.SmallIconSize)
                    )
                }
            }
        }
    }
}

/**
 * Loading Indicator
 */
@Composable
fun ZipLockLoadingIndicator(
    message: String? = null,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        CircularProgressIndicator(
            color = ZipLockColors.LogoPurple,
            strokeWidth = 3.dp,
            modifier = Modifier.size(ZipLockDimensions.LargeIconSize)
        )

        message?.let {
            Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))
            Text(
                text = it,
                style = ZipLockTypography.Normal,
                color = ZipLockColors.LightGrayText,
                textAlign = TextAlign.Center
            )
        }
    }
}

/**
 * File Picker Button
 */
@Composable
fun ZipLockFilePicker(
    selectedFileName: String?,
    onFileSelect: () -> Unit,
    modifier: Modifier = Modifier,
    placeholder: String = "Select archive file..."
) {
    Box(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(min = ZipLockDimensions.TextInputHeight)
            .border(
                width = 1.dp,
                color = ZipLockColors.LightGrayBorder,
                shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)
            )
            .clip(RoundedCornerShape(ZipLockSpacing.BorderRadius))
            .clickable { onFileSelect() }
            .background(ZipLockColors.White)
            .padding(ZipLockSpacing.TextInputPadding),
        contentAlignment = Alignment.CenterStart
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = ZipLockIcons.Folder,
                contentDescription = null,
                tint = ZipLockColors.MediumGray,
                modifier = Modifier.size(ZipLockDimensions.IconSize)
            )

            Spacer(modifier = Modifier.width(ZipLockSpacing.Medium))

            Text(
                text = selectedFileName ?: placeholder,
                style = ZipLockTypography.TextInput,
                color = if (selectedFileName != null) {
                    ZipLockColors.DarkText
                } else {
                    ZipLockColors.LightGrayText
                },
                modifier = Modifier.weight(1f)
            )

            Icon(
                imageVector = ZipLockIcons.FolderOpen,
                contentDescription = "Browse files",
                tint = ZipLockColors.LogoPurple,
                modifier = Modifier.size(ZipLockDimensions.IconSize)
            )
        }
    }
}

/**
 * Data classes for component styling
 */
private data class ButtonColors(
    val background: Color,
    val content: Color,
    val disabledBackground: Color,
    val disabledContent: Color,
    val border: Color? = null
)

private data class TextInputColors(
    val background: Color,
    val text: Color,
    val placeholder: Color,
    val icon: Color,
    val focusedBorder: Color,
    val unfocusedBorder: Color,
    val disabledBorder: Color,
    val disabledBackground: Color,
    val disabledText: Color
)

private data class AlertColors(
    val background: Color,
    val border: Color,
    val text: Color,
    val icon: Color
)

/**
 * Helper functions for component styling
 */
private fun getButtonColors(style: ZipLockButtonStyle, @Suppress("UNUSED_PARAMETER") enabled: Boolean): ButtonColors {
    return when (style) {
        ZipLockButtonStyle.Primary -> ButtonColors(
            background = ZipLockColors.LogoPurple,
            content = ZipLockColors.White,
            disabledBackground = ZipLockColors.DisabledBackground,
            disabledContent = ZipLockColors.DisabledText
        )

        ZipLockButtonStyle.Secondary -> ButtonColors(
            background = ZipLockColors.White,
            content = ZipLockColors.LogoPurple,
            disabledBackground = ZipLockColors.DisabledBackground,
            disabledContent = ZipLockColors.DisabledText,
            border = ZipLockColors.LogoPurple
        )

        ZipLockButtonStyle.Destructive -> ButtonColors(
            background = ZipLockColors.ErrorRed,
            content = ZipLockColors.White,
            disabledBackground = ZipLockColors.DisabledBackground,
            disabledContent = ZipLockColors.DisabledText
        )

        ZipLockButtonStyle.Disabled -> ButtonColors(
            background = ZipLockColors.DisabledBackground,
            content = ZipLockColors.DisabledText,
            disabledBackground = ZipLockColors.DisabledBackground,
            disabledContent = ZipLockColors.DisabledText
        )

        ZipLockButtonStyle.PasswordToggle -> ButtonColors(
            background = Color.Transparent,
            content = ZipLockColors.MediumGray,
            disabledBackground = Color.Transparent,
            disabledContent = ZipLockColors.DisabledText
        )
    }
}

private fun getTextInputColors(style: ZipLockTextInputStyle, @Suppress("UNUSED_PARAMETER") enabled: Boolean): TextInputColors {
    return when (style) {
        ZipLockTextInputStyle.Standard -> TextInputColors(
            background = ZipLockColors.White,
            text = ZipLockColors.DarkText,
            placeholder = ZipLockColors.LightGrayText,
            icon = ZipLockColors.MediumGray,
            focusedBorder = ZipLockColors.LogoPurple,
            unfocusedBorder = ZipLockColors.LightGrayBorder,
            disabledBorder = ZipLockColors.DisabledBorder,
            disabledBackground = ZipLockColors.DisabledBackground,
            disabledText = ZipLockColors.DisabledText
        )

        ZipLockTextInputStyle.Valid -> TextInputColors(
            background = ZipLockColors.White,
            text = ZipLockColors.DarkText,
            placeholder = ZipLockColors.LightGrayText,
            icon = ZipLockColors.SuccessGreen,
            focusedBorder = ZipLockColors.SuccessGreen,
            unfocusedBorder = ZipLockColors.SuccessGreen,
            disabledBorder = ZipLockColors.DisabledBorder,
            disabledBackground = ZipLockColors.DisabledBackground,
            disabledText = ZipLockColors.DisabledText
        )

        ZipLockTextInputStyle.Invalid -> TextInputColors(
            background = ZipLockColors.White,
            text = ZipLockColors.DarkText,
            placeholder = ZipLockColors.LightGrayText,
            icon = ZipLockColors.ErrorRed,
            focusedBorder = ZipLockColors.ErrorRed,
            unfocusedBorder = ZipLockColors.ErrorRed,
            disabledBorder = ZipLockColors.DisabledBorder,
            disabledBackground = ZipLockColors.DisabledBackground,
            disabledText = ZipLockColors.DisabledText
        )

        ZipLockTextInputStyle.Neutral -> TextInputColors(
            background = ZipLockColors.VeryLightGray,
            text = ZipLockColors.DarkText,
            placeholder = ZipLockColors.LightGrayText,
            icon = ZipLockColors.MediumGray,
            focusedBorder = ZipLockColors.MediumGray,
            unfocusedBorder = ZipLockColors.LightGrayBorder,
            disabledBorder = ZipLockColors.DisabledBorder,
            disabledBackground = ZipLockColors.DisabledBackground,
            disabledText = ZipLockColors.DisabledText
        )

        ZipLockTextInputStyle.Title -> TextInputColors(
            background = ZipLockColors.White,
            text = ZipLockColors.DarkText,
            placeholder = ZipLockColors.LightGrayText,
            icon = ZipLockColors.LogoPurple,
            focusedBorder = ZipLockColors.LogoPurple,
            unfocusedBorder = ZipLockColors.LogoPurple,
            disabledBorder = ZipLockColors.DisabledBorder,
            disabledBackground = ZipLockColors.DisabledBackground,
            disabledText = ZipLockColors.DisabledText
        )
    }
}

private fun getAlertColors(level: AlertLevel): AlertColors {
    return when (level) {
        AlertLevel.Error -> AlertColors(
            background = Color(0xFFFFF5F5),
            border = ZipLockColors.ErrorRed,
            text = Color(0xFF991B1B),
            icon = ZipLockColors.ErrorRed
        )

        AlertLevel.Warning -> AlertColors(
            background = Color(0xFFFFFBEB),
            border = ZipLockColors.WarningYellow,
            text = Color(0xFF92400E),
            icon = ZipLockColors.WarningYellow
        )

        AlertLevel.Success -> AlertColors(
            background = Color(0xFFF0FDF4),
            border = ZipLockColors.SuccessGreen,
            text = Color(0xFF166534),
            icon = ZipLockColors.SuccessGreen
        )

        AlertLevel.Info -> AlertColors(
            background = Color(0xFFF0F9FF),
            border = ZipLockColors.LogoPurple,
            text = Color(0xFF1E40AF),
            icon = ZipLockColors.LogoPurple
        )
    }
}
