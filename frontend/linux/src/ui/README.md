# ZipLock UI System Documentation

This document explains how to use the shared UI system in the ZipLock Linux frontend, including themes, components, and styling conventions.

## Overview

The UI system is organized into several modules:

- **`theme`** - Shared color palette, button styles, and utility functions
- **`components`** - Reusable UI components that use the shared theme
- **`views`** - Complete application views (wizard, main app, etc.)

## Using the Shared Theme

### Color Palette

The theme provides brand colors from the design specification:

```rust
use crate::ui::theme::{LOGO_PURPLE, SUCCESS_GREEN, ERROR_RED, LIGHT_BACKGROUND, DARK_TEXT};

// Use brand colors consistently across views
text("Welcome to ZipLock!")
    .style(|theme| text::Appearance {
        color: Some(DARK_TEXT)
    })
```

### Button Styles

Use the shared button styles for consistency:

```rust
use crate::ui::button_styles;

// Primary action button (logo purple background)
button("Save")
    .on_press(Message::Save)
    .padding(utils::button_padding())
    .style(button_styles::primary())

// Secondary button (logo purple border)
button("Cancel")
    .on_press(Message::Cancel)
    .padding(utils::button_padding())
    .style(button_styles::secondary())

// Destructive action button (red background)
button("Delete")
    .on_press(Message::Delete)
    .padding(utils::button_padding())
    .style(button_styles::destructive())
```

### Progress Bars

Use consistent progress bar styling:

```rust
use crate::ui::progress_bar_styles;

progress_bar(0.0..=1.0, progress_value)
    .height(Length::Fixed(20.0))
    .style(progress_bar_styles::primary())
```

### Utility Functions

Use utility functions for consistent spacing and sizing:

```rust
use crate::ui::utils;

// Standard button padding
button("Click me")
    .padding(utils::button_padding())  // [8, 16]

// Small button padding for compact layouts
button("×")
    .padding(utils::small_button_padding())  // [4, 8]

// Standard spacing between elements
column![
    element1,
    Space::with_height(Length::Fixed(utils::standard_spacing().into())), // 20px
    element2,
]

// Consistent border radius
container(content)
    .style(move |theme| container::Appearance {
        border: iced::Border {
            radius: utils::border_radius().into(), // 6.0px
            ..Default::default()
        },
        ..Default::default()
    })
```

## Creating a New View

When creating a new view, follow this pattern:

```rust
//! My New View
//!
//! Description of what this view does.

use iced::{
    widget::{button, column, text, Space},
    Alignment, Command, Element, Length,
};

use crate::ui::{button_styles, theme, utils};

#[derive(Debug, Clone)]
pub enum MyViewMessage {
    // Define your messages here
    DoSomething,
    Cancel,
}

#[derive(Debug)]
pub struct MyView {
    // Your view state here
}

impl MyView {
    pub fn new() -> Self {
        Self {
            // Initialize your state
        }
    }

    pub fn update(&mut self, message: MyViewMessage) -> Command<MyViewMessage> {
        match message {
            MyViewMessage::DoSomething => {
                // Handle the message
                Command::none()
            }
            MyViewMessage::Cancel => {
                // Handle cancel
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<MyViewMessage> {
        column![
            text("My New View")
                .size(24)
                .style(|theme| text::Appearance {
                    color: Some(theme::DARK_TEXT)
                }),
            
            Space::with_height(Length::Fixed(utils::standard_spacing().into())),
            
            button("Primary Action")
                .on_press(MyViewMessage::DoSomething)
                .padding(utils::button_padding())
                .style(button_styles::primary()),
                
            button("Cancel")
                .on_press(MyViewMessage::Cancel)
                .padding(utils::button_padding())
                .style(button_styles::secondary()),
        ]
        .padding(30)
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
    }
}
```

## Using Reusable Components

The components module provides pre-configured UI elements:

```rust
use crate::ui::components::{primary_button, secondary_button, destructive_button};

// Simple usage
let save_btn = primary_button("Save", Some(Message::Save));
let cancel_btn = secondary_button("Cancel", Some(Message::Cancel));
let delete_btn = destructive_button("Delete", Some(Message::Delete));

// Using preset buttons
use crate::ui::components::presets;

let save_btn = presets::save_button(Some(Message::Save));
let cancel_btn = presets::cancel_button(Some(Message::Cancel));
let delete_btn = presets::delete_button(Some(Message::Delete));
```

## Theming Best Practices

### 1. Always Use Theme Colors

Don't hardcode colors. Always use the theme constants:

```rust
// ✅ Good - uses theme colors
text_color: theme::LOGO_PURPLE

// ❌ Bad - hardcoded color
text_color: Color::from_rgb(0.514, 0.220, 0.925)
```

### 2. Use Consistent Spacing

Use the utility functions for consistent spacing:

```rust
// ✅ Good - consistent spacing
.padding(utils::button_padding())
Space::with_height(Length::Fixed(utils::standard_spacing().into()))

// ❌ Bad - arbitrary spacing
.padding([12, 20])
Space::with_height(Length::Fixed(25.0))
```

### 3. Follow Button Hierarchy

Use appropriate button styles based on the action's importance:

- **Primary** - Main actions (Save, Create, Login)
- **Secondary** - Supporting actions (Cancel, Edit, Browse)
- **Destructive** - Dangerous actions (Delete, Remove, Logout)

### 4. Maintain Visual Consistency

Use similar layouts and patterns across views:

```rust
// Standard view layout pattern
column![
    view_header(),
    Space::with_height(Length::Fixed(utils::standard_spacing().into())),
    view_content(),
    Space::with_height(Length::Fixed(utils::standard_spacing().into())),
    view_footer(),
]
.padding(30)
.spacing(10)
```

## Integration with Main Application

To use your new view in the main application:

1. Add your view module to `views/mod.rs`
2. Export your view types from the UI module
3. Update the main application state and message handling
4. Use `create_ziplock_theme()` in your application's theme method

```rust
// In views/mod.rs
pub mod my_view;
pub use my_view::{MyView, MyViewMessage};

// In main.rs
use ui::{create_ziplock_theme, views::{MyView, MyViewMessage}};

impl Application for MyApp {
    fn theme(&self) -> Theme {
        create_ziplock_theme()
    }
    
    // ... rest of implementation
}
```

This ensures all views use the same consistent theme and styling throughout the application.