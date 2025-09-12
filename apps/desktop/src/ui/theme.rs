//! Comprehensive theme and styling for ZipLock Linux App
//!
//! This module contains the custom theme implementation using the ZipLock brand colors
//! as specified in the design.md file. It provides consistent styling across all views
//! with full Iced 0.13 compatibility.
//!
//! ## Icon Attribution
//! Icons used in this application are from Iconoir (https://iconoir.com/),
//! a beautiful collection of free SVG icons by Luca Burgio and contributors.
//! Licensed under MIT License.

use iced::{
    widget::{button, svg, text_editor, text_input},
    Background, Border, Color, Shadow, Theme,
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

/// Embedded xmark icon SVG for close/dismiss buttons (from Iconoir style)
pub const XMARK_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/xmark.svg");

/// Embedded credit card icon SVG for credit card credentials
pub const CREDIT_CARD_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/credit-card.svg");

/// Embedded note icon SVG for secure notes
pub const NOTE_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/note.svg");

/// Embedded user icon SVG for identity credentials
pub const USER_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/user.svg");

/// Embedded document icon SVG for document credentials
pub const DOCUMENT_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/document.svg");

/// Embedded bank icon SVG for bank account credentials
pub const BANK_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/bank.svg");

/// Embedded wallet icon SVG for crypto wallet credentials
pub const WALLET_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/wallet.svg");

/// Embedded database icon SVG for database credentials
pub const DATABASE_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/database.svg");

/// Embedded license icon SVG for software license credentials
pub const LICENSE_ICON_SVG: &[u8] = include_bytes!("../../resources/icons/license.svg");

// Icon helper functions
pub fn ziplock_logo() -> svg::Handle {
    svg::Handle::from_memory(ZIPLOCK_LOGO_SVG)
}

pub fn eye_icon() -> svg::Handle {
    svg::Handle::from_memory(EYE_ICON_SVG)
}

pub fn eye_off_icon() -> svg::Handle {
    svg::Handle::from_memory(EYE_OFF_ICON_SVG)
}

pub fn alert_icon() -> svg::Handle {
    svg::Handle::from_memory(ALERT_ICON_SVG)
}

pub fn check_icon() -> svg::Handle {
    svg::Handle::from_memory(CHECK_ICON_SVG)
}

pub fn error_icon() -> svg::Handle {
    svg::Handle::from_memory(ERROR_ICON_SVG)
}

pub fn warning_icon() -> svg::Handle {
    svg::Handle::from_memory(WARNING_ICON_SVG)
}

pub fn refresh_icon() -> svg::Handle {
    svg::Handle::from_memory(REFRESH_ICON_SVG)
}

pub fn plus_icon() -> svg::Handle {
    svg::Handle::from_memory(PLUS_ICON_SVG)
}

pub fn settings_icon() -> svg::Handle {
    svg::Handle::from_memory(SETTINGS_ICON_SVG)
}

pub fn lock_icon() -> svg::Handle {
    svg::Handle::from_memory(LOCK_ICON_SVG)
}

pub fn xmark_icon() -> svg::Handle {
    svg::Handle::from_memory(XMARK_ICON_SVG)
}

pub fn credit_card_icon() -> svg::Handle {
    svg::Handle::from_memory(CREDIT_CARD_ICON_SVG)
}

pub fn note_icon() -> svg::Handle {
    svg::Handle::from_memory(NOTE_ICON_SVG)
}

pub fn user_icon() -> svg::Handle {
    svg::Handle::from_memory(USER_ICON_SVG)
}

pub fn document_icon() -> svg::Handle {
    svg::Handle::from_memory(DOCUMENT_ICON_SVG)
}

pub fn bank_icon() -> svg::Handle {
    svg::Handle::from_memory(BANK_ICON_SVG)
}

pub fn wallet_icon() -> svg::Handle {
    svg::Handle::from_memory(WALLET_ICON_SVG)
}

pub fn database_icon() -> svg::Handle {
    svg::Handle::from_memory(DATABASE_ICON_SVG)
}

pub fn license_icon() -> svg::Handle {
    svg::Handle::from_memory(LICENSE_ICON_SVG)
}

// ZipLock Brand Colors - Restored Original Values
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

    /// Primary button style using logo purple - Iced 0.13 style function
    pub fn primary() -> impl Fn(&Theme, button::Status) -> button::Style {
        |_theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(LOGO_PURPLE)),
                text_color: WHITE,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_HOVER)),
                text_color: WHITE,
                border: Border {
                    color: LOGO_PURPLE_HOVER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_PRESSED)),
                text_color: WHITE,
                border: Border {
                    color: LOGO_PURPLE_PRESSED,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                },
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(DISABLED_BACKGROUND)),
                text_color: DISABLED_TEXT,
                border: Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
        }
    }

    /// Secondary button style with logo purple border
    pub fn secondary() -> impl Fn(&Theme, button::Status) -> button::Style {
        |_theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(TRANSPARENT)),
                text_color: LOGO_PURPLE,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_LIGHT)),
                text_color: LOGO_PURPLE,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_MEDIUM)),
                text_color: LOGO_PURPLE,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(TRANSPARENT)),
                text_color: DISABLED_TEXT,
                border: Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
        }
    }

    /// Destructive button style using error red
    pub fn destructive() -> impl Fn(&Theme, button::Status) -> button::Style {
        |_theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(ERROR_RED)),
                text_color: WHITE,
                border: Border {
                    color: ERROR_RED,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(ERROR_RED_HOVER)),
                text_color: WHITE,
                border: Border {
                    color: ERROR_RED_HOVER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(ERROR_RED_PRESSED)),
                text_color: WHITE,
                border: Border {
                    color: ERROR_RED_PRESSED,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                },
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(DISABLED_BACKGROUND)),
                text_color: DISABLED_TEXT,
                border: Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
        }
    }

    /// Password toggle button style for inactive state (password hidden)
    pub fn password_toggle_inactive() -> impl Fn(&Theme, button::Status) -> button::Style {
        |_theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(VERY_LIGHT_GRAY)),
                text_color: LOGO_PURPLE,
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_SUBTLE)),
                text_color: LOGO_PURPLE,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_LIGHT)),
                text_color: LOGO_PURPLE,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(DISABLED_BACKGROUND)),
                text_color: DISABLED_TEXT,
                border: Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
        }
    }

    /// Password toggle button style for active state (password shown)
    pub fn password_toggle_active() -> impl Fn(&Theme, button::Status) -> button::Style {
        |_theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(LOGO_PURPLE)),
                text_color: WHITE,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_HOVER)),
                text_color: WHITE,
                border: Border {
                    color: LOGO_PURPLE_HOVER,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_PRESSED)),
                text_color: WHITE,
                border: Border {
                    color: LOGO_PURPLE_PRESSED,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(DISABLED_BACKGROUND)),
                text_color: DISABLED_TEXT,
                border: Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
        }
    }

    /// Text field style button for copyable TOTP codes
    pub fn text_field_like() -> impl Fn(&Theme, button::Status) -> button::Style {
        |_theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(WHITE)),
                text_color: DARK_TEXT,
                border: Border {
                    color: DISABLED_BACKGROUND,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                text_color: DARK_TEXT,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(VERY_LIGHT_GRAY)),
                text_color: DARK_TEXT,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(VERY_LIGHT_GRAY)),
                text_color: DISABLED_TEXT,
                border: Border {
                    color: EXTRA_LIGHT_GRAY,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
        }
    }

    /// Toast close button style with white background and bold border
    pub fn toast_close_button() -> impl Fn(&Theme, button::Status) -> button::Style {
        |_theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(WHITE)),
                text_color: DARK_TEXT,
                border: Border {
                    color: WHITE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.9))),
                text_color: DARK_TEXT,
                border: Border {
                    color: WHITE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.8))),
                text_color: DARK_TEXT,
                border: Border {
                    color: WHITE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.5))),
                text_color: DISABLED_TEXT,
                border: Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
        }
    }

    /// Credential list item button style with white background and purple border
    pub fn credential_list_item() -> impl Fn(&Theme, button::Status) -> button::Style {
        |_theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(WHITE)),
                text_color: DARK_TEXT,
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_LIGHT)),
                text_color: DARK_TEXT,
                border: Border {
                    color: LOGO_PURPLE_HOVER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                },
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(LOGO_PURPLE_MEDIUM)),
                text_color: DARK_TEXT,
                border: Border {
                    color: LOGO_PURPLE_PRESSED,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow {
                    color: SHADOW_COLOR,
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 1.0,
                },
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(VERY_LIGHT_GRAY)),
                text_color: DISABLED_TEXT,
                border: Border {
                    color: DISABLED_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                shadow: Shadow::default(),
            },
        }
    }
}

/// Custom text input styles for validation states and different input types
pub mod text_input_styles {
    use super::*;

    /// Standard text input style
    pub fn standard() -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
        |_theme, status| match status {
            text_input::Status::Active => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_input::Status::Hovered => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_input::Status::Focused => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_input::Status::Disabled => text_input::Style {
                background: Background::Color(VERY_LIGHT_GRAY),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DISABLED_TEXT,
                selection: DISABLED_TEXT,
            },
        }
    }

    /// Valid text input style (green border)
    pub fn valid() -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
        |_theme, status| match status {
            text_input::Status::Active => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: SUCCESS_GREEN,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: SUCCESS_GREEN,
            },
            text_input::Status::Hovered => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: SUCCESS_GREEN,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: SUCCESS_GREEN,
            },
            text_input::Status::Focused => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: SUCCESS_GREEN,
                    width: 3.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: SUCCESS_GREEN,
            },
            text_input::Status::Disabled => text_input::Style {
                background: Background::Color(VERY_LIGHT_GRAY),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DISABLED_TEXT,
                selection: DISABLED_TEXT,
            },
        }
    }

    /// Invalid text input style (red border)
    pub fn invalid() -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
        |_theme, status| match status {
            text_input::Status::Active => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: ERROR_RED,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: ERROR_RED,
            },
            text_input::Status::Hovered => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: ERROR_RED,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: ERROR_RED,
            },
            text_input::Status::Focused => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: ERROR_RED,
                    width: 3.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: ERROR_RED,
            },
            text_input::Status::Disabled => text_input::Style {
                background: Background::Color(VERY_LIGHT_GRAY),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DISABLED_TEXT,
                selection: DISABLED_TEXT,
            },
        }
    }

    /// Neutral text input style (purple border for focused state)
    pub fn neutral() -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
        |_theme, status| match status {
            text_input::Status::Active => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_input::Status::Hovered => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_input::Status::Focused => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 3.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_input::Status::Disabled => text_input::Style {
                background: Background::Color(VERY_LIGHT_GRAY),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DISABLED_TEXT,
                selection: DISABLED_TEXT,
            },
        }
    }

    /// Title text input style (larger font and padding)
    pub fn title() -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
        |_theme, status| match status {
            text_input::Status::Active => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_input::Status::Hovered => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_input::Status::Focused => text_input::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_input::Status::Disabled => text_input::Style {
                background: Background::Color(VERY_LIGHT_GRAY),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DISABLED_TEXT,
                selection: DISABLED_TEXT,
            },
        }
    }
}

pub mod text_editor_styles {
    use super::*;

    /// Standard text editor style with white background
    pub fn standard() -> impl Fn(&Theme, text_editor::Status) -> text_editor::Style {
        |_theme, status| match status {
            text_editor::Status::Active => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_editor::Status::Hovered => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_editor::Status::Focused => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_editor::Status::Disabled => text_editor::Style {
                background: Background::Color(VERY_LIGHT_GRAY),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DISABLED_TEXT,
                selection: DISABLED_TEXT,
            },
        }
    }

    /// Valid text editor style with white background and green border
    pub fn valid() -> impl Fn(&Theme, text_editor::Status) -> text_editor::Style {
        |_theme, status| match status {
            text_editor::Status::Active => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: SUCCESS_GREEN,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: SUCCESS_GREEN,
            },
            text_editor::Status::Hovered => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: SUCCESS_GREEN,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: SUCCESS_GREEN,
            },
            text_editor::Status::Focused => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: SUCCESS_GREEN,
                    width: 3.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: SUCCESS_GREEN,
            },
            text_editor::Status::Disabled => text_editor::Style {
                background: Background::Color(VERY_LIGHT_GRAY),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DISABLED_TEXT,
                selection: DISABLED_TEXT,
            },
        }
    }

    /// Invalid text editor style with white background and red border
    pub fn invalid() -> impl Fn(&Theme, text_editor::Status) -> text_editor::Style {
        |_theme, status| match status {
            text_editor::Status::Active => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: ERROR_RED,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: ERROR_RED,
            },
            text_editor::Status::Hovered => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: ERROR_RED,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: ERROR_RED,
            },
            text_editor::Status::Focused => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: ERROR_RED,
                    width: 3.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: ERROR_RED,
            },
            text_editor::Status::Disabled => text_editor::Style {
                background: Background::Color(VERY_LIGHT_GRAY),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DISABLED_TEXT,
                selection: DISABLED_TEXT,
            },
        }
    }

    /// Neutral text editor style with white background and purple border
    pub fn neutral() -> impl Fn(&Theme, text_editor::Status) -> text_editor::Style {
        |_theme, status| match status {
            text_editor::Status::Active => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_editor::Status::Hovered => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 2.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_editor::Status::Focused => text_editor::Style {
                background: Background::Color(WHITE),
                border: Border {
                    color: LOGO_PURPLE,
                    width: 3.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DARK_TEXT,
                selection: LOGO_PURPLE,
            },
            text_editor::Status::Disabled => text_editor::Style {
                background: Background::Color(VERY_LIGHT_GRAY),
                border: Border {
                    color: LIGHT_GRAY_BORDER,
                    width: 1.0,
                    radius: utils::border_radius().into(),
                },
                icon: MEDIUM_GRAY,
                placeholder: MEDIUM_GRAY,
                value: DISABLED_TEXT,
                selection: DISABLED_TEXT,
            },
        }
    }
}

/// Utility functions for consistent spacing, sizing, and styling
pub mod utils {
    use iced::Padding;

    /// Creates a consistent spacing value for UI elements
    pub fn standard_spacing() -> u16 {
        20
    }

    /// Creates a consistent padding value for buttons
    pub fn button_padding() -> Padding {
        Padding::from([10, 20])
    }

    /// Creates a consistent padding value for small buttons
    pub fn small_button_padding() -> Padding {
        Padding::from([4, 8])
    }

    /// Creates a consistent padding value for standard UI buttons
    pub fn standard_button_padding() -> Padding {
        Padding::from([12, 24])
    }

    /// Creates a consistent padding value for repository buttons
    pub fn repository_button_padding() -> Padding {
        Padding::from([15, 20])
    }

    /// Creates a consistent padding value for setup buttons
    pub fn setup_button_padding() -> Padding {
        Padding::from([12, 32])
    }

    /// Creates a consistent padding value for text inputs
    pub fn text_input_padding() -> Padding {
        Padding::from([10, 15])
    }

    /// Creates a consistent padding value for title text inputs (larger)
    pub fn title_input_padding() -> Padding {
        Padding::from([15, 20])
    }

    /// Creates a consistent padding value for toast dismiss buttons
    pub fn toast_dismiss_padding() -> Padding {
        Padding::from([5, 8])
    }

    /// Creates a consistent padding value for small elements
    pub fn small_element_padding() -> Padding {
        Padding::from([8, 12])
    }

    /// Creates a consistent padding value for logo containers
    pub fn logo_container_padding() -> Padding {
        Padding::from([20, 40])
    }

    /// Creates a consistent padding value for main content areas
    pub fn main_content_padding() -> Padding {
        Padding::from([20, 30])
    }

    /// Creates a consistent padding value for search bars
    pub fn search_bar_padding() -> Padding {
        Padding::from([12, 16])
    }

    /// Creates a consistent padding value for add credential buttons
    pub fn add_credential_button_padding() -> Padding {
        Padding::from([10, 16])
    }

    /// Creates a consistent padding value for scrollable lists
    pub fn list_padding() -> Padding {
        Padding::from([15, 20])
    }

    /// Creates a consistent padding value for error containers
    pub fn error_container_padding() -> Padding {
        Padding::from([20, 25])
    }

    /// Creates a consistent padding value for completion buttons
    pub fn completion_button_padding() -> Padding {
        Padding::from([12, 20])
    }

    /// Creates a consistent border radius for UI elements
    pub fn border_radius() -> f32 {
        10.0
    }

    /// Creates a consistent padding for alert components
    pub fn alert_padding() -> Padding {
        Padding::from([15, 20])
    }

    /// Creates a consistent padding for password visibility toggle buttons
    pub fn password_toggle_padding() -> Padding {
        Padding::from([8, 12])
    }

    /// Creates a password visibility toggle button with eye icon and proper styling
    pub fn password_visibility_toggle<'a, Message: Clone + 'a>(
        show_password: bool,
        on_toggle: Message,
    ) -> iced::widget::Button<'a, Message> {
        use iced::widget::{button, svg};

        let icon = if show_password {
            super::eye_icon()
        } else {
            super::eye_off_icon()
        };

        button(
            svg(icon)
                .width(iced::Length::Fixed(16.0))
                .height(iced::Length::Fixed(16.0)),
        )
        .on_press(on_toggle)
        .style(move |theme, status| {
            if show_password {
                super::button_styles::password_toggle_active()(theme, status)
            } else {
                super::button_styles::password_toggle_inactive()(theme, status)
            }
        })
        .padding(password_toggle_padding())
    }

    /// Typography utilities for consistent font sizing
    pub mod typography {
        use std::sync::OnceLock;

        static FONT_SIZE: OnceLock<f32> = OnceLock::new();

        /// Initialize the global font size
        pub fn init_font_size(size: f32) {
            let _ = FONT_SIZE.set(size);
        }

        /// Get the base font size, defaulting to 14.0 if not set
        fn base_font_size() -> f32 {
            *FONT_SIZE.get().unwrap_or(&14.0)
        }

        /// Get normal text size
        pub fn normal_text_size() -> f32 {
            base_font_size()
        }

        /// Get text input size
        pub fn text_input_size() -> f32 {
            base_font_size()
        }

        /// Get medium text size (slightly larger than normal)
        pub fn medium_text_size() -> f32 {
            base_font_size() + 2.0
        }

        /// Get small text size (smaller than normal)
        pub fn small_text_size() -> f32 {
            base_font_size() - 2.0
        }

        /// Get header text size (larger than medium)
        pub fn header_text_size() -> f32 {
            base_font_size() + 4.0
        }

        /// Get large text size (larger than header)
        pub fn large_text_size() -> f32 {
            base_font_size() + 6.0
        }

        /// Get the icon SVG for a credential type
        pub fn get_credential_type_icon(credential_type: &str) -> iced::widget::svg::Handle {
            match credential_type {
                "login" => crate::ui::theme::lock_icon(),
                "credit_card" => crate::ui::theme::credit_card_icon(),
                "secure_note" => crate::ui::theme::note_icon(),
                "identity" => crate::ui::theme::user_icon(),
                "password" => crate::ui::theme::lock_icon(),
                "document" => crate::ui::theme::document_icon(),
                "ssh_key" => crate::ui::theme::settings_icon(),
                "bank_account" => crate::ui::theme::bank_icon(),
                "api_credentials" => crate::ui::theme::settings_icon(),
                "crypto_wallet" => crate::ui::theme::wallet_icon(),
                "database" => crate::ui::theme::database_icon(),
                "software_license" => crate::ui::theme::license_icon(),
                _ => crate::ui::theme::alert_icon(),
            }
        }

        /// Get extra large text size (largest size)
        pub fn extra_large_text_size() -> f32 {
            base_font_size() + 10.0
        }

        /// Get title input size (for larger title inputs)
        pub fn title_input_size() -> f32 {
            base_font_size() + 2.0
        }
    }
}

/// Alert system for user feedback
pub mod alerts {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum AlertLevel {
        Error,
        Warning,
        Success,
        Info,
    }

    #[derive(Debug, Clone)]
    pub struct AlertMessage {
        pub level: AlertLevel,
        pub title: Option<String>,
        pub message: String,
        pub dismissible: bool,
    }

    impl AlertMessage {
        pub fn error<S: Into<String>>(message: S) -> Self {
            Self {
                level: AlertLevel::Error,
                title: None,
                message: message.into(),
                dismissible: true,
            }
        }

        pub fn error_with_title<S1: Into<String>, S2: Into<String>>(
            title: S1,
            message: S2,
        ) -> Self {
            Self {
                level: AlertLevel::Error,
                title: Some(title.into()),
                message: message.into(),
                dismissible: true,
            }
        }

        pub fn warning<S: Into<String>>(message: S) -> Self {
            Self {
                level: AlertLevel::Warning,
                title: None,
                message: message.into(),
                dismissible: true,
            }
        }

        pub fn success<S: Into<String>>(message: S) -> Self {
            Self {
                level: AlertLevel::Success,
                title: None,
                message: message.into(),
                dismissible: true,
            }
        }

        pub fn info<S: Into<String>>(message: S) -> Self {
            Self {
                level: AlertLevel::Info,
                title: None,
                message: message.into(),
                dismissible: true,
            }
        }

        pub fn ipc_error<S: Into<String>>(message: S) -> Self {
            Self {
                level: AlertLevel::Error,
                title: Some("Backend Connection Error".to_string()),
                message: message.into(),
                dismissible: true,
            }
        }
    }

    /// Render an alert using proper styling with custom themes
    pub fn render_alert<'a, Message>(
        alert: &'a AlertMessage,
        on_dismiss: Option<Message>,
    ) -> iced::Element<'a, Message>
    where
        Message: 'a + Clone,
    {
        use iced::widget::{button, column, container, row, svg, text};
        use iced::{Alignment, Length};

        let icon = match alert.level {
            AlertLevel::Error => svg(error_icon()),
            AlertLevel::Warning => svg(warning_icon()),
            AlertLevel::Success => svg(check_icon()),
            AlertLevel::Info => svg(alert_icon()),
        };

        let mut content_column = column![].spacing(8);

        if let Some(ref title) = alert.title {
            content_column =
                content_column.push(text(title).size(utils::typography::medium_text_size()));
        }

        content_column =
            content_column.push(text(&alert.message).size(utils::typography::normal_text_size()));

        let mut main_row = row![icon.width(20).height(20), content_column,]
            .spacing(12)
            .align_y(Alignment::Center);

        if let Some(dismiss_message) = on_dismiss {
            if alert.dismissible {
                let dismiss_button = button(svg(xmark_icon()).width(16).height(16))
                    .on_press(dismiss_message)
                    .style(super::button_styles::toast_close_button())
                    .padding(utils::toast_dismiss_padding());

                main_row = main_row.push(dismiss_button);
            }
        }

        container(main_row)
            .padding(utils::alert_padding())
            .width(Length::Fill)
            .into()
    }
}
