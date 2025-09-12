//! Reusable Button Components for ZipLock Linux App
//!
//! This module provides pre-configured button components that use the shared theme system.
//! These components can be used across different views for consistency and proper styling.

use iced::{widget::button, Element};

use crate::ui::{theme, utils};

/// A primary action button with consistent styling using theme.rs styles
#[allow(dead_code)] // Reusable component for future use
pub fn primary_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::button_padding())
        .style(theme::button_styles::primary());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A secondary action button with consistent styling using theme.rs styles
#[allow(dead_code)] // Reusable component for future use
pub fn secondary_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::button_padding())
        .style(theme::button_styles::secondary());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A destructive action button with consistent styling using theme.rs styles
#[allow(dead_code)] // Reusable component for future use
pub fn destructive_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::button_padding())
        .style(theme::button_styles::destructive());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A small button with reduced padding for compact layouts using primary style
#[allow(dead_code)] // Reusable component for future use
pub fn small_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::small_button_padding())
        .style(theme::button_styles::primary());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A small secondary button with reduced padding for compact layouts
#[allow(dead_code)] // Reusable component for future use
pub fn small_secondary_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::small_button_padding())
        .style(theme::button_styles::secondary());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A toolbar button optimized for header/toolbar usage using secondary style
#[allow(dead_code)] // Reusable component for future use
pub fn toolbar_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    small_secondary_button(text, on_press)
}

/// A danger toolbar button for destructive actions in toolbars
#[allow(dead_code)] // Reusable component for future use
pub fn danger_toolbar_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::small_button_padding())
        .style(theme::button_styles::destructive());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// An icon button using secondary styling for toolbar/UI actions
#[allow(dead_code)] // Reusable component for future use
pub fn icon_button<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(content)
        .padding(12)
        .style(theme::button_styles::secondary());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A primary icon button for important actions
#[allow(dead_code)] // Reusable component for future use
pub fn primary_icon_button<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(content)
        .padding(12)
        .style(theme::button_styles::primary());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// Button component configuration for different contexts
pub mod presets {
    use super::*;

    /// Create a "Save" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn save_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        primary_button("Save", on_press)
    }

    /// Create a "Cancel" button with secondary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn cancel_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        secondary_button("Cancel", on_press)
    }

    /// Create a "Delete" button with destructive styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn delete_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        destructive_button("Delete", on_press)
    }

    /// Create an "Add" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn add_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        primary_button("Add", on_press)
    }

    /// Create an "Edit" button with secondary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn edit_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        secondary_button("Edit", on_press)
    }

    /// Create a "Close" button with secondary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn close_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        secondary_button("Close", on_press)
    }

    /// Create an "OK" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn ok_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        primary_button("OK", on_press)
    }

    /// Create a "Yes" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn yes_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        primary_button("Yes", on_press)
    }

    /// Create a "No" button with secondary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn no_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        secondary_button("No", on_press)
    }

    /// Create a "Browse..." button with secondary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn browse_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        secondary_button("Browse...", on_press)
    }

    /// Create a "Try Again" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn try_again_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        primary_button("Try Again", on_press)
    }

    /// Create a "Back" button with secondary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn back_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        secondary_button("Back", on_press)
    }

    /// Create a "Next" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn next_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        primary_button("Next", on_press)
    }

    /// Create a "Clear" button with secondary styling for small layouts
    #[allow(dead_code)] // Reusable component for future use
    pub fn clear_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        small_secondary_button("Clear", on_press)
    }

    /// Create an "Open Repository" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn open_repository_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        primary_button("Open Repository", on_press)
    }

    /// Create a "Start Using ZipLock" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn start_using_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        primary_button("Start Using ZipLock", on_press)
    }

    /// Create a "Get Started" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn get_started_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        primary_button("Get Started", on_press)
    }

    /// Create a "Create Repository" button with primary styling
    #[allow(dead_code)] // Reusable component for future use
    pub fn create_repository_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        primary_button("Create Repository", on_press)
    }
}

/// Example usage and documentation
#[cfg(doc)]
mod examples {
    use super::*;

    /// Example of how to use the button components in a view
    #[allow(dead_code)]
    fn example_usage() {
        // Using individual button functions with proper styling
        let _save_btn = primary_button("Save", Some(()));
        let _cancel_btn = secondary_button("Cancel", Some(()));
        let _delete_btn = destructive_button("Delete", Some(()));

        // Using preset buttons
        let _preset_save = presets::save_button(Some(()));
        let _preset_cancel = presets::cancel_button(Some(()));
        let _preset_delete = presets::delete_button(Some(()));

        // Toolbar buttons
        let _toolbar_btn = toolbar_button("Settings", Some(()));
        let _danger_toolbar = danger_toolbar_button("Remove", Some(()));

        // Icon buttons
        let _icon_btn = icon_button("Icon Content", Some(()));
        let _primary_icon_btn = primary_icon_button("Primary Icon", Some(()));
    }
}
