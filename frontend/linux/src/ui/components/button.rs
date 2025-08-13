//! Reusable Button Components for ZipLock Linux Frontend
//!
//! This module provides pre-configured button components that use the shared theme system.
//! These components can be used across different views for consistency.

use iced::{widget::button, Element};

use crate::ui::{button_styles, utils};

/// A primary action button with consistent styling
pub fn primary_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::button_padding())
        .style(button_styles::primary());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A secondary action button with consistent styling
pub fn secondary_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::button_padding())
        .style(button_styles::secondary());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A destructive action button with consistent styling
pub fn destructive_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::button_padding())
        .style(button_styles::destructive());

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A small button with reduced padding for compact layouts
pub fn small_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
    style: iced::theme::Button,
) -> Element<'a, Message> {
    let mut btn = button(text)
        .padding(utils::small_button_padding())
        .style(style);

    if let Some(message) = on_press {
        btn = btn.on_press(message);
    }

    btn.into()
}

/// A toolbar button optimized for header/toolbar usage
pub fn toolbar_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    small_button(text, on_press, button_styles::secondary())
}

/// A danger toolbar button for destructive actions in toolbars
pub fn danger_toolbar_button<'a, Message: Clone + 'a>(
    text: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    small_button(text, on_press, button_styles::destructive())
}

/// Button component configuration for different contexts
pub mod presets {
    use super::*;

    /// Create a "Save" button with primary styling
    pub fn save_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        primary_button("Save", on_press)
    }

    /// Create a "Cancel" button with secondary styling
    pub fn cancel_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        secondary_button("Cancel", on_press)
    }

    /// Create a "Delete" button with destructive styling
    pub fn delete_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        destructive_button("Delete", on_press)
    }

    /// Create an "Add" button with primary styling
    pub fn add_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        primary_button("Add", on_press)
    }

    /// Create an "Edit" button with secondary styling
    pub fn edit_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        secondary_button("Edit", on_press)
    }

    /// Create a "Close" button with secondary styling
    pub fn close_button<'a, Message: Clone + 'a>(
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        secondary_button("Close", on_press)
    }

    /// Create an "OK" button with primary styling
    pub fn ok_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        primary_button("OK", on_press)
    }

    /// Create a "Yes" button with primary styling
    pub fn yes_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        primary_button("Yes", on_press)
    }

    /// Create a "No" button with secondary styling
    pub fn no_button<'a, Message: Clone + 'a>(on_press: Option<Message>) -> Element<'a, Message> {
        secondary_button("No", on_press)
    }
}

/// Example usage and documentation
#[cfg(doc)]
mod examples {
    use super::*;

    /// Example of how to use the button components in a view
    #[allow(dead_code)]
    fn example_usage() {
        // Using individual button functions
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
    }
}
