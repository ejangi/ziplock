//! Shared theme and styling for ZipLock Linux App
//!
//! This module contains the custom theme implementation using the ZipLock brand colors
//! as specified in the design.md file. It provides consistent styling across all views.
//!
//! ## Icon Attribution
//! Icons used in this application are from Iconoir (https://iconoir.com/),
//! a beautiful collection of free SVG icons by Luca Burgio and contributors.
//! Licensed under MIT License.

use iced::{
    widget::button, widget::progress_bar, widget::svg, Background, Border, Color, Shadow, Theme,
};

/// Embedded ZipLock logo SVG for use across all views
pub const ZIPLOCK_LOGO_SVG: &[u8] = include_bytes!("../../resources/icons/ziplock-logo.svg");

/// Embedded eye icon SVG for password visibility toggle (from Iconoir)
pub const EYE_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/eye-solid.svg");

/// Embedded eye-off icon SVG for password visibility toggle (from Iconoir)
pub const EYE_OFF_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/eye-off.svg");

/// Embedded alert icon SVG for info/general alerts (from Iconoir)
pub const ALERT_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/alert.svg");

/// Embedded check icon SVG for success alerts (from Iconoir)
pub const CHECK_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/check.svg");

/// Embedded error icon SVG for error alerts (from Iconoir)
pub const ERROR_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/error.svg");

/// Embedded warning icon SVG for warning alerts (from Iconoir)
pub const WARNING_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/warning.svg");

/// Embedded refresh icon SVG for refresh button (from Iconoir style)
pub const REFRESH_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/refresh.svg");

/// Embedded plus icon SVG for add button (from Iconoir style)
pub const PLUS_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/plus.svg");

/// Embedded settings icon SVG for settings button (from Iconoir style)
pub const SETTINGS_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/settings.svg");

/// Embedded lock icon SVG for lock button (from Iconoir style)
pub const LOCK_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/lock.svg");

/// Helper function to create an SVG handle from the embedded ZipLock logo
pub fn ziplock_logo() -> svg::Handle {
    svg::Handle::from_memory(ZIPLOCK_LOGO_SVG)
}

/// Helper function to create an SVG handle from the embedded eye icon
pub fn eye_icon() -> svg::Handle {
    svg::Handle::from_memory(EYE_ICON_SVG)
}

/// Helper function to create an SVG handle from the embedded eye-off icon
pub fn eye_off_icon() -> svg::Handle {
    svg::Handle::from_memory(EYE_OFF_ICON_SVG)
}

/// Helper function to create an SVG handle from the embedded alert icon
pub fn alert_icon() -> svg::Handle {
    svg::Handle::from_memory(ALERT_ICON_SVG)
}

/// Helper function to create an SVG handle from the embedded check icon
pub fn check_icon() -> svg::Handle {
    svg::Handle::from_memory(CHECK_ICON_SVG)
}

/// Helper function to create an SVG handle from the embedded error icon
pub fn error_icon() -> svg::Handle {
    svg::Handle::from_memory(ERROR_ICON_SVG)
}

/// Helper function to create an SVG handle from the embedded warning icon
pub fn warning_icon() -> svg::Handle {
    svg::Handle::from_memory(WARNING_ICON_SVG)
}

/// Helper function to create an SVG handle from the embedded refresh icon
pub fn refresh_icon() -> svg::Handle {
    svg::Handle::from_memory(REFRESH_ICON_SVG)
}

/// Helper function to create an SVG handle from the embedded plus icon
pub fn plus_icon() -> svg::Handle {
    svg::Handle::from_memory(PLUS_ICON_SVG)
}

/// Helper function to create an SVG handle from the embedded settings icon
pub fn settings_icon() -> svg::Handle {
    svg::Handle::from_memory(SETTINGS_ICON_SVG)
}

/// Helper function to create an SVG handle from the embedded lock icon
pub fn lock_icon() -> svg::Handle {
    svg::Handle::from_memory(LOCK_ICON_SVG)
}

/// Logo purple color from design.md (#8338ec)
pub const LOGO_PURPLE: Color = Color::from_rgb(0.514, 0.220, 0.925);

/// Logo purple hover state (slightly darker)
pub const LOGO_PURPLE_HOVER: Color = Color::from_rgb(0.45, 0.18, 0.82);

/// Logo purple pressed state (even darker)
pub const LOGO_PURPLE_PRESSED: Color = Color::from_rgb(0.40, 0.15, 0.75);

/// Logo purple with low opacity for hover backgrounds
pub const LOGO_PURPLE_LIGHT: Color = Color::from_rgba(0.514, 0.220, 0.925, 0.1);

/// Logo purple with medium opacity for pressed backgrounds
pub const LOGO_PURPLE_MEDIUM: Color = Color::from_rgba(0.514, 0.220, 0.925, 0.2);

/// Logo purple with very light opacity for subtle backgrounds
pub const LOGO_PURPLE_SUBTLE: Color = Color::from_rgba(0.514, 0.220, 0.925, 0.05);

/// Success/Valid color from design.md (#06d6a0)
pub const SUCCESS_GREEN: Color = Color::from_rgb(0.024, 0.839, 0.627);

/// Error/Invalid color from design.md (#ef476f)
pub const ERROR_RED: Color = Color::from_rgb(0.937, 0.278, 0.435);

/// Error red hover state (slightly darker)
pub const ERROR_RED_HOVER: Color = Color::from_rgb(0.85, 0.25, 0.40);

/// Error red pressed state (even darker)
pub const ERROR_RED_PRESSED: Color = Color::from_rgb(0.80, 0.22, 0.35);

/// Warning color from design.md (#fcbf49)
pub const WARNING_YELLOW: Color = Color::from_rgb(0.988, 0.749, 0.286);

/// Light background color from design.md (#F8F9FA)
pub const LIGHT_BACKGROUND: Color = Color::from_rgb(0.97, 0.976, 0.98);

/// Dark text color from design.md (#212529)
pub const DARK_TEXT: Color = Color::from_rgb(0.129, 0.145, 0.161);

/// White color constant
pub const WHITE: Color = Color::WHITE;

/// Transparent color constant
pub const TRANSPARENT: Color = Color::TRANSPARENT;

/// Disabled background color (light gray)
pub const DISABLED_BACKGROUND: Color = Color::from_rgb(0.8, 0.8, 0.8);

/// Disabled text color (medium gray)
pub const DISABLED_TEXT: Color = Color::from_rgb(0.5, 0.5, 0.5);

/// Disabled border color (darker gray)
pub const DISABLED_BORDER: Color = Color::from_rgb(0.7, 0.7, 0.7);

/// Standard shadow color (black with low opacity)
pub const SHADOW_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.1);

/// Light gray text color for help text
pub const LIGHT_GRAY_TEXT: Color = Color::from_rgb(0.6, 0.6, 0.6);

/// Light gray border color for text inputs
pub const LIGHT_GRAY_BORDER: Color = Color::from_rgb(0.8, 0.8, 0.8);

/// Medium gray color for icons and placeholders
pub const MEDIUM_GRAY: Color = Color::from_rgb(0.5, 0.5, 0.5);

/// Very light gray background for disabled inputs
pub const VERY_LIGHT_GRAY: Color = Color::from_rgb(0.95, 0.95, 0.95);

/// Extra light gray border for disabled elements
pub const EXTRA_LIGHT_GRAY: Color = Color::from_rgb(0.9, 0.9, 0.9);

/// Creates the ZipLock custom theme with brand colors
pub fn create_ziplock_theme() -> Theme {
    Theme::custom(
        "ZipLock".to_string(),
        iced::theme::Palette {
            background: LIGHT_BACKGROUND,
            text: DARK_TEXT,
            primary: LOGO_PURPLE,
            success: SUCCESS_GREEN,
            danger: ERROR_RED,
        },
    )
}

/// Custom button style functions for consistent styling across views
pub mod button_styles {
    use super::*;

    /// Primary button style using logo purple
    pub fn primary() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(PrimaryButtonStyle))
    }

    /// Secondary button style with logo purple border
    pub fn secondary() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(SecondaryButtonStyle))
    }

    /// Destructive button style using error red
    pub fn destructive() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(DestructiveButtonStyle))
    }

    /// Disabled button style
    pub fn disabled() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(DisabledButtonStyle))
    }

    /// Password toggle button style for inactive state (password hidden)
    pub fn password_toggle_inactive() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(PasswordToggleInactiveStyle))
    }

    /// Password toggle button style for active state (password shown)
    pub fn password_toggle_active() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(PasswordToggleActiveStyle))
    }

    /// Text field style button for copyable TOTP codes
    pub fn text_field_like() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(TextFieldLikeButtonStyle))
    }

    // Style implementations
    struct PrimaryButtonStyle;
    struct SecondaryButtonStyle;
    struct DestructiveButtonStyle;
    struct DisabledButtonStyle;
    struct PasswordToggleInactiveStyle;
    struct PasswordToggleActiveStyle;
    struct TextFieldLikeButtonStyle;

    impl button::StyleSheet for PrimaryButtonStyle {
        type Style = Theme;

        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(LOGO_PURPLE.into()),
                text_color: WHITE,
                border: iced::Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                shadow_offset: iced::Vector::new(0.0, 2.0),
            }
        }

        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(LOGO_PURPLE_HOVER.into()),
                ..active
            }
        }

        fn pressed(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(LOGO_PURPLE_PRESSED.into()),
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 1.0),
                ..active
            }
        }

        fn disabled(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(DISABLED_BACKGROUND.into()),
                text_color: DISABLED_TEXT,
                border: iced::Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
    }

    impl button::StyleSheet for SecondaryButtonStyle {
        type Style = Theme;

        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(TRANSPARENT.into()),
                text_color: LOGO_PURPLE,
                border: iced::Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }

        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(LOGO_PURPLE_LIGHT.into()),
                ..active
            }
        }

        fn pressed(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(LOGO_PURPLE_MEDIUM.into()),
                ..active
            }
        }

        fn disabled(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(TRANSPARENT.into()),
                text_color: DISABLED_TEXT,
                border: iced::Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
    }

    impl button::StyleSheet for DestructiveButtonStyle {
        type Style = Theme;

        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(ERROR_RED.into()),
                text_color: WHITE,
                border: iced::Border {
                    color: ERROR_RED,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                shadow_offset: iced::Vector::new(0.0, 2.0),
            }
        }

        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(ERROR_RED_HOVER.into()),
                ..active
            }
        }

        fn pressed(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(ERROR_RED_PRESSED.into()),
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 1.0),
                ..active
            }
        }

        fn disabled(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(DISABLED_BACKGROUND.into()),
                text_color: DISABLED_TEXT,
                border: iced::Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
    }

    impl button::StyleSheet for PasswordToggleInactiveStyle {
        type Style = Theme;

        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(WHITE.into()),
                text_color: LOGO_PURPLE,
                border: iced::Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }

        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(LOGO_PURPLE_SUBTLE.into()),
                ..active
            }
        }

        fn pressed(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(LOGO_PURPLE_LIGHT.into()),
                ..active
            }
        }

        fn disabled(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(WHITE.into()),
                text_color: DISABLED_TEXT,
                border: iced::Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
    }

    impl button::StyleSheet for PasswordToggleActiveStyle {
        type Style = Theme;

        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(LOGO_PURPLE.into()),
                text_color: WHITE,
                border: iced::Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }

        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(Color::from_rgb(0.45, 0.18, 0.82).into()), // Slightly darker purple
                ..active
            }
        }

        fn pressed(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(Color::from_rgb(0.40, 0.15, 0.75).into()), // Even darker purple
                ..active
            }
        }

        fn disabled(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(WHITE.into()),
                text_color: DISABLED_TEXT,
                border: iced::Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
    }

    impl button::StyleSheet for DisabledButtonStyle {
        type Style = Theme;

        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(DISABLED_BACKGROUND.into()),
                text_color: DISABLED_TEXT,
                border: iced::Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }

        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            self.active(style)
        }

        fn pressed(&self, style: &Self::Style) -> button::Appearance {
            self.active(style)
        }

        fn disabled(&self, style: &Self::Style) -> button::Appearance {
            self.active(style)
        }
    }

    impl button::StyleSheet for TextFieldLikeButtonStyle {
        type Style = Theme;

        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(WHITE.into()),
                text_color: DARK_TEXT,
                border: iced::Border {
                    color: DISABLED_BACKGROUND,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }

        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(Color::from_rgb(0.98, 0.98, 0.98).into()), // Slight gray tint on hover
                border: iced::Border {
                    color: LOGO_PURPLE, // Purple border on hover
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                ..active
            }
        }

        fn pressed(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(VERY_LIGHT_GRAY.into()),
                border: iced::Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                ..active
            }
        }

        fn disabled(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(VERY_LIGHT_GRAY.into()),
                text_color: DISABLED_TEXT,
                border: iced::Border {
                    color: EXTRA_LIGHT_GRAY,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: iced::Shadow::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
    }
}

/// Custom text input styles
pub mod text_input_styles {
    use super::*;

    /// Standard text input style
    pub fn standard() -> iced::theme::TextInput {
        iced::theme::TextInput::Custom(Box::new(StandardTextInputStyle))
    }

    /// Valid text input style (green border)
    pub fn valid() -> iced::theme::TextInput {
        iced::theme::TextInput::Custom(Box::new(ValidTextInputStyle))
    }

    /// Invalid text input style (red border)
    pub fn invalid() -> iced::theme::TextInput {
        iced::theme::TextInput::Custom(Box::new(InvalidTextInputStyle))
    }

    /// Neutral text input style (purple border for focused state)
    pub fn neutral() -> iced::theme::TextInput {
        iced::theme::TextInput::Custom(Box::new(NeutralTextInputStyle))
    }

    // Style implementations
    struct StandardTextInputStyle;
    struct ValidTextInputStyle;
    struct InvalidTextInputStyle;
    struct NeutralTextInputStyle;

    impl iced::widget::text_input::StyleSheet for StandardTextInputStyle {
        type Style = iced::Theme;

        fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: Color::WHITE.into(),
                border: iced::Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }

        fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: WHITE.into(),
                border: iced::Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }

        fn placeholder_color(&self, _style: &Self::Style) -> Color {
            MEDIUM_GRAY
        }

        fn value_color(&self, _style: &Self::Style) -> Color {
            DARK_TEXT
        }

        fn disabled_color(&self, _style: &Self::Style) -> Color {
            MEDIUM_GRAY
        }

        fn selection_color(&self, _style: &Self::Style) -> Color {
            LOGO_PURPLE
        }

        fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: VERY_LIGHT_GRAY.into(),
                border: iced::Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }
    }

    impl iced::widget::text_input::StyleSheet for ValidTextInputStyle {
        type Style = iced::Theme;

        fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: WHITE.into(),
                border: iced::Border {
                    color: SUCCESS_GREEN,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }

        fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: WHITE.into(),
                border: iced::Border {
                    color: SUCCESS_GREEN,
                    width: 3.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }

        fn placeholder_color(&self, _style: &Self::Style) -> Color {
            MEDIUM_GRAY
        }

        fn value_color(&self, _style: &Self::Style) -> Color {
            DARK_TEXT
        }

        fn disabled_color(&self, _style: &Self::Style) -> Color {
            MEDIUM_GRAY
        }

        fn selection_color(&self, _style: &Self::Style) -> Color {
            SUCCESS_GREEN
        }

        fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: VERY_LIGHT_GRAY.into(),
                border: iced::Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }
    }

    impl iced::widget::text_input::StyleSheet for InvalidTextInputStyle {
        type Style = iced::Theme;

        fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: WHITE.into(),
                border: iced::Border {
                    color: ERROR_RED,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }

        fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: WHITE.into(),
                border: iced::Border {
                    color: ERROR_RED,
                    width: 3.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }

        fn placeholder_color(&self, _style: &Self::Style) -> Color {
            MEDIUM_GRAY
        }

        fn value_color(&self, _style: &Self::Style) -> Color {
            DARK_TEXT
        }

        fn disabled_color(&self, _style: &Self::Style) -> Color {
            MEDIUM_GRAY
        }

        fn selection_color(&self, _style: &Self::Style) -> Color {
            ERROR_RED
        }

        fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: VERY_LIGHT_GRAY.into(),
                border: iced::Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }
    }

    impl iced::widget::text_input::StyleSheet for NeutralTextInputStyle {
        type Style = iced::Theme;

        fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: WHITE.into(),
                border: iced::Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }

        fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: WHITE.into(),
                border: iced::Border {
                    color: LOGO_PURPLE,
                    width: 3.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }

        fn placeholder_color(&self, _style: &Self::Style) -> Color {
            MEDIUM_GRAY
        }

        fn value_color(&self, _style: &Self::Style) -> Color {
            DARK_TEXT
        }

        fn disabled_color(&self, _style: &Self::Style) -> Color {
            MEDIUM_GRAY
        }

        fn selection_color(&self, _style: &Self::Style) -> Color {
            LOGO_PURPLE
        }

        fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
            iced::widget::text_input::Appearance {
                background: VERY_LIGHT_GRAY.into(),
                border: iced::Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon_color: MEDIUM_GRAY,
            }
        }
    }
}

/// Custom progress bar styles
pub mod progress_bar_styles {
    use super::*;

    /// Primary progress bar style using logo purple
    pub fn primary() -> iced::theme::ProgressBar {
        iced::theme::ProgressBar::Custom(Box::new(PrimaryProgressBarStyle))
    }

    struct PrimaryProgressBarStyle;

    impl progress_bar::StyleSheet for PrimaryProgressBarStyle {
        type Style = Theme;

        fn appearance(&self, _style: &Self::Style) -> progress_bar::Appearance {
            progress_bar::Appearance {
                background: Color::from_rgb(0.9, 0.9, 0.9).into(),
                bar: LOGO_PURPLE.into(),
                border_radius: utils::border_radius().into(),
            }
        }
    }
}

/// Custom container styles for error displays and alerts
pub mod container_styles {
    use super::*;
    use iced::widget::container;

    /// Error alert container style with red border and light red background
    pub fn error_alert() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(ErrorAlertStyle))
    }

    /// Warning alert container style with yellow border and light yellow background
    pub fn warning_alert() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(WarningAlertStyle))
    }

    /// Success alert container style with green border and light green background
    pub fn success_alert() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(SuccessAlertStyle))
    }

    /// Info alert container style with purple border and light purple background
    pub fn info_alert() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(InfoAlertStyle))
    }

    /// Sidebar container style with light gray background
    pub fn sidebar() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(SidebarStyle))
    }

    // Style implementations
    struct ErrorAlertStyle;
    struct WarningAlertStyle;
    struct SuccessAlertStyle;
    struct InfoAlertStyle;
    struct SidebarStyle;

    impl container::StyleSheet for ErrorAlertStyle {
        type Style = iced::Theme;

        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            container::Appearance {
                background: Some(iced::Color::from_rgba(0.937, 0.278, 0.435, 0.1).into()),
                border: iced::Border {
                    color: ERROR_RED,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                text_color: Some(ERROR_RED),
                shadow: iced::Shadow::default(),
            }
        }
    }

    impl container::StyleSheet for WarningAlertStyle {
        type Style = iced::Theme;

        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            let warning_color = iced::Color::from_rgb(0.988, 0.749, 0.286); // Yellow
            container::Appearance {
                background: Some(iced::Color::from_rgba(0.988, 0.749, 0.286, 0.1).into()),
                border: iced::Border {
                    color: warning_color,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                text_color: Some(iced::Color::from_rgb(0.8, 0.6, 0.0)),
                shadow: iced::Shadow::default(),
            }
        }
    }

    impl container::StyleSheet for SuccessAlertStyle {
        type Style = iced::Theme;

        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            container::Appearance {
                background: Some(iced::Color::from_rgba(0.024, 0.839, 0.627, 0.1).into()),
                border: iced::Border {
                    color: SUCCESS_GREEN,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                text_color: Some(SUCCESS_GREEN),
                shadow: iced::Shadow::default(),
            }
        }
    }

    impl container::StyleSheet for InfoAlertStyle {
        type Style = iced::Theme;

        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.96, 0.9, 1.0))),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
                text_color: None,
            }
        }
    }

    impl container::StyleSheet for SidebarStyle {
        type Style = iced::Theme;

        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                border: Border {
                    color: Color::from_rgb(0.85, 0.85, 0.85),
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
                text_color: None,
            }
        }
    }
}

/// Utility functions for common UI patterns
pub mod utils {
    /// Creates a consistent spacing value for UI elements
    pub fn standard_spacing() -> u16 {
        20
    }

    /// Creates a consistent padding value for buttons
    pub fn button_padding() -> [u16; 2] {
        [20, 20]
    }

    /// Creates a consistent padding value for small buttons
    pub fn small_button_padding() -> [u16; 2] {
        [4, 8]
    }

    /// Creates a consistent padding value for standard UI buttons
    pub fn standard_button_padding() -> [u16; 2] {
        [10, 20]
    }

    /// Creates a consistent padding value for repository buttons
    pub fn repository_button_padding() -> [u16; 2] {
        [15, 20]
    }

    /// Creates a consistent padding value for setup buttons
    pub fn setup_button_padding() -> [u16; 2] {
        [15, 30]
    }

    /// Creates a consistent padding value for text inputs
    pub fn text_input_padding() -> u16 {
        10
    }

    /// Creates a consistent padding value for toast dismiss buttons
    pub fn toast_dismiss_padding() -> [u16; 2] {
        [2, 6]
    }

    /// Creates a consistent padding value for small elements
    pub fn small_element_padding() -> [u16; 2] {
        [0, 5]
    }

    /// Creates a consistent padding value for logo containers
    pub fn logo_container_padding() -> [u16; 4] {
        [20, 0, 30, 0]
    }

    /// Creates a consistent padding value for main content areas
    pub fn main_content_padding() -> [u16; 4] {
        [0, 30, 30, 30]
    }

    /// Creates a consistent padding value for search bars
    pub fn search_bar_padding() -> [u16; 2] {
        [8, 12]
    }

    /// Creates a consistent padding value for add credential buttons
    pub fn add_credential_button_padding() -> [u16; 2] {
        [12, 24]
    }

    /// Creates a consistent padding value for scrollable lists
    pub fn list_padding() -> [u16; 2] {
        [10, 0]
    }

    /// Creates a consistent padding value for error containers
    pub fn error_container_padding() -> [u16; 2] {
        [12, 16]
    }

    /// Creates a consistent padding value for completion buttons
    pub fn completion_button_padding() -> [u16; 2] {
        [12, 24]
    }

    /// Creates a consistent border radius for UI elements
    pub fn border_radius() -> f32 {
        10.0
    }

    /// Creates a consistent padding for alert components
    pub fn alert_padding() -> [u16; 2] {
        [12, 16]
    }

    /// Creates a consistent padding for password visibility toggle buttons
    pub fn password_toggle_padding() -> [u16; 2] {
        [8, 12]
    }

    /// Creates a password visibility toggle button with eye icon
    pub fn password_visibility_toggle<'a, Message: Clone + 'a>(
        show_password: bool,
        on_toggle: Message,
    ) -> iced::widget::Button<'a, Message> {
        use iced::widget::{button, svg};

        let (icon, style) = if show_password {
            // Password is shown, display eye icon to represent current visible state and use active (purple) style
            (
                super::eye_icon(),
                super::button_styles::password_toggle_active(),
            )
        } else {
            // Password is hidden, display eye-off icon to represent current obscured state and use inactive (white with purple outline) style
            (
                super::eye_off_icon(),
                super::button_styles::password_toggle_inactive(),
            )
        };

        button(
            svg(icon)
                .width(iced::Length::Fixed(16.0))
                .height(iced::Length::Fixed(16.0)),
        )
        .on_press(on_toggle)
        .style(style)
        .padding(password_toggle_padding())
    }
}

/// Alert component utilities and types
pub mod alerts {
    use super::*;
    use iced::{
        widget::{button, column, container, row, svg, text, Space},
        Alignment, Element, Length,
    };

    /// Alert severity levels
    #[derive(Debug, Clone, PartialEq)]
    pub enum AlertLevel {
        Error,
        Warning,
        Success,
        Info,
    }

    /// Alert message structure
    #[derive(Debug, Clone)]
    pub struct AlertMessage {
        pub level: AlertLevel,
        pub title: Option<String>,
        pub message: String,
        pub dismissible: bool,
    }

    impl AlertMessage {
        /// Create a new error alert
        pub fn error(message: impl Into<String>) -> Self {
            Self {
                level: AlertLevel::Error,
                title: Some("Error".to_string()),
                message: message.into(),
                dismissible: true,
            }
        }

        /// Create a new error alert with custom title
        pub fn error_with_title(title: impl Into<String>, message: impl Into<String>) -> Self {
            Self {
                level: AlertLevel::Error,
                title: Some(title.into()),
                message: message.into(),
                dismissible: true,
            }
        }

        /// Create a new warning alert
        pub fn warning(message: impl Into<String>) -> Self {
            Self {
                level: AlertLevel::Warning,
                title: Some("Warning".to_string()),
                message: message.into(),
                dismissible: true,
            }
        }

        /// Create a new success alert
        pub fn success(message: impl Into<String>) -> Self {
            Self {
                level: AlertLevel::Success,
                title: Some("Success".to_string()),
                message: message.into(),
                dismissible: true,
            }
        }

        /// Create a new info alert
        pub fn info(message: impl Into<String>) -> Self {
            Self {
                level: AlertLevel::Info,
                title: Some("Information".to_string()),
                message: message.into(),
                dismissible: true,
            }
        }

        /// Create an IPC error alert with a specific message
        pub fn ipc_error(message: impl Into<String>) -> Self {
            Self {
                level: AlertLevel::Error,
                title: Some("Connection Error".to_string()),
                message: message.into(),
                dismissible: true,
            }
        }
    }

    /// Renders an alert component
    pub fn render_alert<Message: Clone + 'static>(
        alert: &AlertMessage,
        on_dismiss: Option<Message>,
    ) -> Element<'_, Message> {
        let container_style = match alert.level {
            AlertLevel::Error => container_styles::error_alert(),
            AlertLevel::Warning => container_styles::warning_alert(),
            AlertLevel::Success => container_styles::success_alert(),
            AlertLevel::Info => container_styles::info_alert(),
        };

        let icon_svg = match alert.level {
            AlertLevel::Error => error_icon(),
            AlertLevel::Warning => warning_icon(),
            AlertLevel::Success => check_icon(),
            AlertLevel::Info => alert_icon(),
        };

        let mut content = row![svg(icon_svg).width(16).height(16)];

        let mut text_column = column![];

        if let Some(title) = &alert.title {
            let title_color = match alert.level {
                AlertLevel::Error => ERROR_RED,
                AlertLevel::Warning => iced::Color::from_rgb(0.8, 0.6, 0.0),
                AlertLevel::Success => SUCCESS_GREEN,
                AlertLevel::Info => LOGO_PURPLE,
            };
            text_column = text_column.push(
                text(title)
                    .size(14)
                    .style(iced::theme::Text::Color(title_color)),
            );
        }

        text_column = text_column.push(
            text(&alert.message)
                .size(12)
                .style(iced::theme::Text::Color(DARK_TEXT)),
        );

        content = content
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(text_column.width(Length::Fill).spacing(4));

        if alert.dismissible {
            if let Some(dismiss_msg) = on_dismiss {
                content = content.push(Space::with_width(Length::Fixed(10.0))).push(
                    button("✕")
                        .on_press(dismiss_msg)
                        .padding(utils::toast_dismiss_padding())
                        .style(button_styles::secondary()),
                );
            }
        }

        container(content.align_items(Alignment::Center))
            .padding(utils::alert_padding())
            .width(Length::Fill)
            .style(container_style)
            .into()
    }
}
