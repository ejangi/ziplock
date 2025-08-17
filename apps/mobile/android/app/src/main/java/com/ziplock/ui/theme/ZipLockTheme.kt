package com.ziplock.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * ZipLock Color Palette
 * Matches the Linux theme.rs color definitions
 */
@Immutable
object ZipLockColors {
    // Brand Colors
    val LogoPurple = Color(0xFF8338EC)
    val LogoPurpleHover = Color(0xFF9F5FFF)
    val LogoPurplePressed = Color(0xFF6B2BC4)
    val LogoPurpleLight = Color(0xFFB085FF)
    val LogoPurpleMedium = Color(0xFF9F5FFF)
    val LogoPurpleSubtle = Color(0xFFE8D5FF)

    // Validation Colors
    val SuccessGreen = Color(0xFF06D6A0)
    val ErrorRed = Color(0xFFEF476F)
    val ErrorRedHover = Color(0xFFFF6B8A)
    val ErrorRedPressed = Color(0xFFD63660)
    val WarningYellow = Color(0xFFFCBF49)

    // Background Colors
    val LightBackground = Color(0xFFF8F9FA)
    val White = Color(0xFFFFFFFF)
    val Transparent = Color(0x00000000)

    // Text Colors
    val DarkText = Color(0xFF212529)
    val LightGrayText = Color(0xFF6C757D)

    // Disabled States
    val DisabledBackground = Color(0xFFE9ECEF)
    val DisabledText = Color(0xFF6C757D)
    val DisabledBorder = Color(0xFFDEE2E6)

    // Gray Shades
    val LightGrayBorder = Color(0xFFDEE2E6)
    val MediumGray = Color(0xFFADB5BD)
    val VeryLightGray = Color(0xFFF8F9FA)
    val ExtraLightGray = Color(0xFFFAFBFC)

    // Shadow
    val ShadowColor = Color(0x1A000000)
}

/**
 * ZipLock Typography System
 * Provides consistent text styling across the app
 */
@Immutable
object ZipLockTypography {
    private val fontFamily = FontFamily.Default

    val ExtraLarge = TextStyle(
        fontFamily = fontFamily,
        fontWeight = FontWeight.Bold,
        fontSize = 32.sp,
        lineHeight = 40.sp
    )

    val Large = TextStyle(
        fontFamily = fontFamily,
        fontWeight = FontWeight.SemiBold,
        fontSize = 24.sp,
        lineHeight = 32.sp
    )

    val Header = TextStyle(
        fontFamily = fontFamily,
        fontWeight = FontWeight.SemiBold,
        fontSize = 20.sp,
        lineHeight = 28.sp
    )

    val Medium = TextStyle(
        fontFamily = fontFamily,
        fontWeight = FontWeight.Medium,
        fontSize = 16.sp,
        lineHeight = 24.sp
    )

    val Normal = TextStyle(
        fontFamily = fontFamily,
        fontWeight = FontWeight.Normal,
        fontSize = 14.sp,
        lineHeight = 20.sp
    )

    val Small = TextStyle(
        fontFamily = fontFamily,
        fontWeight = FontWeight.Normal,
        fontSize = 12.sp,
        lineHeight = 16.sp
    )

    val TextInput = TextStyle(
        fontFamily = fontFamily,
        fontWeight = FontWeight.Normal,
        fontSize = 16.sp,
        lineHeight = 24.sp
    )

    val TitleInput = TextStyle(
        fontFamily = fontFamily,
        fontWeight = FontWeight.SemiBold,
        fontSize = 18.sp,
        lineHeight = 28.sp
    )
}

/**
 * ZipLock Spacing System
 * Provides consistent spacing values throughout the app
 */
@Immutable
object ZipLockSpacing {
    val None: Dp = 0.dp
    val ExtraSmall: Dp = 4.dp
    val Small: Dp = 8.dp
    val Medium: Dp = 12.dp
    val Standard: Dp = 16.dp
    val Large: Dp = 20.dp
    val ExtraLarge: Dp = 24.dp
    val Huge: Dp = 32.dp
    val Massive: Dp = 48.dp

    // Specific component spacing
    val ButtonPadding: Dp = 16.dp
    val SmallButtonPadding: Dp = 12.dp
    val StandardButtonPadding: Dp = 16.dp
    val RepositoryButtonPadding: Dp = 20.dp
    val SetupButtonPadding: Dp = 24.dp

    val TextInputPadding: Dp = 16.dp
    val TitleInputPadding: Dp = 20.dp
    val ToastDismissPadding: Dp = 8.dp
    val SmallElementPadding: Dp = 8.dp

    val LogoContainerPadding: Dp = 24.dp
    val MainContentPadding: Dp = 24.dp
    val SearchBarPadding: Dp = 16.dp
    val AddCredentialButtonPadding: Dp = 20.dp
    val ListPadding: Dp = 16.dp
    val ErrorContainerPadding: Dp = 16.dp
    val CompletionButtonPadding: Dp = 24.dp
    val AlertPadding: Dp = 16.dp
    val PasswordTogglePadding: Dp = 8.dp

    val BorderRadius: Dp = 8.dp
}

/**
 * ZipLock Dimensions
 * Standard dimensions for UI elements
 */
@Immutable
object ZipLockDimensions {
    val MinButtonHeight: Dp = 48.dp
    val StandardButtonHeight: Dp = 56.dp
    val LargeButtonHeight: Dp = 64.dp

    val TextInputHeight: Dp = 56.dp
    val TitleInputHeight: Dp = 64.dp

    val IconSize: Dp = 24.dp
    val SmallIconSize: Dp = 16.dp
    val LargeIconSize: Dp = 32.dp

    val LogoSize: Dp = 120.dp
    val SmallLogoSize: Dp = 64.dp

    val ToastWidth: Dp = 320.dp
    val ToastHeight: Dp = 80.dp

    val CardElevation: Dp = 4.dp
    val ModalElevation: Dp = 8.dp
}

/**
 * ZipLock Theme Data Class
 * Contains all theme-related properties
 */
@Immutable
data class ZipLockTheme(
    val colors: ZipLockColors = ZipLockColors,
    val typography: ZipLockTypography = ZipLockTypography,
    val spacing: ZipLockSpacing = ZipLockSpacing,
    val dimensions: ZipLockDimensions = ZipLockDimensions
)

/**
 * Default ZipLock theme instance
 */
val DefaultZipLockTheme = ZipLockTheme()

/**
 * Utility function to get credential type emoji
 * Matches the Linux implementation's get_credential_type_icon function
 */
fun getCredentialTypeEmoji(credentialType: String): String {
    return when (credentialType.lowercase()) {
        "login", "website", "web" -> "🌐"
        "credit_card", "card", "payment" -> "💳"
        "note", "secure_note" -> "📝"
        "identity", "personal" -> "👤"
        "document" -> "📄"
        "bank", "banking" -> "🏦"
        "wallet", "crypto" -> "💼"
        "database", "server" -> "🗄️"
        "license", "software" -> "🔑"
        else -> "🔒"
    }
}
