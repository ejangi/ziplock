# Iced 0.12 to 0.13 Migration Guide for ZipLock

This document provides a comprehensive guide for migrating the ZipLock password manager from Iced 0.12 to 0.13. It covers all breaking changes, new features, and specific considerations for our custom theming system.

## Table of Contents

1. [Overview of Changes](#overview-of-changes)
2. [Critical Breaking Changes](#critical-breaking-changes)
3. [Linux App-Specific Considerations](#linux-app-specific-considerations)
4. [ZipLock-Specific Migration Steps](#ziplock-specific-migration-steps)
5. [New Features and Opportunities](#new-features-and-opportunities)
6. [Migration Checklist](#migration-checklist)
7. [Testing Strategy](#testing-strategy)
8. [Rollback Plan](#rollback-plan)

## Overview of Changes

Iced 0.13 represents a major evolution in the framework with significant API changes designed to improve developer experience and application performance. The most impactful changes for ZipLock include:

- **Complete removal of the `Sandbox` trait** (used in simple applications)
- **Introduction of the `Task` API** replacing `Command`s for async operations
- **New class-based theming system** with improved styling APIs
- **Enhanced widget ecosystem** with new components and helpers
- **Improved application lifecycle management**

## Critical Breaking Changes

### 1. Application Architecture Migration

**Current (0.12):**
```rust
use iced::{Application, Command, Element, Settings, Theme};

impl Application for ZipLockApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // Initialization
    }

    fn title(&self) -> String {
        "ZipLock Password Manager".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        // Update logic
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        // View logic
    }

    fn theme(&self) -> Self::Theme {
        create_ziplock_theme()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        // Subscriptions
    }
}

pub fn main() -> iced::Result {
    ZipLockApp::run(Settings::default())
}
```

**New (0.13):**
```rust
use iced::{Task, Element, Theme};

pub fn main() -> iced::Result {
    iced::application(ZipLockApp::new, ZipLockApp::update, ZipLockApp::view)
        .subscription(ZipLockApp::subscription)
        .theme(ZipLockApp::theme)
        .title(ZipLockApp::title)
        .window_size((1200.0, 800.0))
        .run()
}

impl ZipLockApp {
    fn new() -> (Self, Task<Message>) {
        // Initialization with Task instead of Command
        (
            Self::default(),
            Task::perform(Self::load_config(), Message::ConfigLoaded)
        )
    }

    fn title(&self) -> String {
        match &self.current_archive {
            Some(path) => format!("ZipLock - {}", path.display()),
            None => "ZipLock Password Manager".to_string(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        // Update logic with Task return type
    }

    fn view(&self) -> Element<'_, Message> {
        // View logic
    }

    fn theme(&self) -> Theme {
        create_ziplock_theme()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        // Subscriptions
    }
}
```

### 2. Command to Task Migration

**Key Changes:**
- `Command<T>` → `Task<T>`
- `Command::none()` → `Task::none()`
- `Command::perform()` → `Task::perform()`
- `Command::batch()` → `Task::batch()`

**ZipLock Archive Operations Example:**

```rust
// Before (0.12)
fn update(&mut self, message: Message) -> Command<Message> {
    match message {
        Message::OpenArchive(path) => {
            Command::perform(
                archive::open_archive(path.clone(), self.password.clone()),
                |result| Message::ArchiveOpened(result)
            )
        }
        Message::SaveCredential(credential) => {
            if let Some(ref mut archive) = self.archive {
                Command::perform(
                    archive::save_credential(archive.clone(), credential),
                    Message::CredentialSaved
                )
            } else {
                Command::none()
            }
        }
    }
}

// After (0.13)
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::OpenArchive(path) => {
            Task::perform(
                archive::open_archive(path.clone(), self.password.clone()),
                |result| Message::ArchiveOpened(result)
            )
        }
        Message::SaveCredential(credential) => {
            if let Some(ref mut archive) = self.archive {
                // Enhanced with abortable tasks for cancellable operations
                let (task, handle) = Task::perform(
                    archive::save_credential(archive.clone(), credential),
                    Message::CredentialSaved
                ).abortable();
                
                self.save_handle = Some(handle.abort_on_drop());
                task
            } else {
                Task::none()
            }
        }
    }
}
```

### 3. Theming System Migration

Our custom theme system in `apps/linux/src/ui/theme.rs` needs significant updates:

**Current Theme Creation:**
```rust
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
```

**New Class-Based Styling (0.13):**
```rust
// Enhanced theme with extended palette support
pub fn create_ziplock_theme() -> Theme {
    Theme::custom_with_fn("ZipLock".to_string(), 
        iced::theme::Palette {
            background: LIGHT_BACKGROUND,
            text: DARK_TEXT,
            primary: LOGO_PURPLE,
            success: SUCCESS_GREEN,
            danger: ERROR_RED,
        },
        |palette| {
            // Custom extended palette configuration
            iced::theme::Extended {
                background: iced::theme::Background {
                    base: palette.background,
                    weak: VERY_LIGHT_GRAY,
                    strong: MEDIUM_GRAY,
                    strongest: DARK_TEXT,
                },
                primary: iced::theme::Primary {
                    base: palette.primary,
                    weak: LOGO_PURPLE_LIGHT,
                    strong: LOGO_PURPLE_HOVER,
                    strongest: LOGO_PURPLE_PRESSED,
                },
                // ... other extended palette configurations
            }
        }
    )
}

// Simplified button styling using built-in classes
pub mod button_styles {
    use iced::widget::button;

    pub fn primary() -> button::Style {
        button::primary // Use built-in primary style
    }

    pub fn secondary() -> button::Style {
        button::secondary
    }

    pub fn destructive() -> button::Style {
        button::danger
    }

    // Custom styles can still be created using closures
    pub fn password_toggle() -> button::Style {
        button::Style::default().with(|style| {
            style.background = Some(iced::Background::Color(TRANSPARENT));
            style.text_color = LOGO_PURPLE;
            style.border = iced::Border::default().with_color(TRANSPARENT);
        })
    }
}
```

### 4. Widget Updates and New Components

**Alignment Changes:**
```rust
// Before (0.12)
column![/* content */]
    .align_items(Alignment::Center)
    .spacing(20)

// After (0.13)
column![/* content */]
    .align_x(iced::Center)  // More specific alignment
    .spacing(20)
```

**New Helper Functions:**
```rust
// Enhanced layout helpers
let main_content = center_x(
    column![
        ziplock_logo(),
        text("Welcome to ZipLock").size(24),
        credential_list
    ]
    .spacing(20)
    .max_width(800)
).padding(40);

// Stack widget for overlays (new in 0.13)
let with_modal = stack![
    main_content,
    if self.show_modal {
        Some(modal_overlay())
    } else {
        None
    }
];
```

## Linux App-Specific Considerations

The ZipLock Linux application has several complex systems that require special attention during the Iced 0.13 migration. These considerations are based on analysis of the current `apps/linux` implementation.

### 1. Complex Subscription System Migration

The Linux app has a sophisticated subscription system that combines multiple event sources:

**Current Implementation:**
```rust
fn subscription(&self) -> iced::Subscription<Message> {
    iced::Subscription::batch([
        close_subscription,           // Window close handling
        activity_subscription,        // User activity tracking
        toast_subscription,          // Toast auto-dismiss
        auto_lock_subscription,      // Auto-lock timers
        auto_update_subscription,    // Update checks
        view_subscription,           // View-specific events
    ])
}
```

**Migration Considerations:**
- Timer-based subscriptions using `iced::time::every()` may have API changes
- Event subscription APIs (`iced::event::listen_with`) likely updated
- Batched subscription handling might need adjustment
- View-specific subscription integration patterns may change

### 2. Extensive Custom Theming System

The Linux app has one of the most comprehensive custom theming systems:

**Current Theme Complexity:**
- **8+ custom button styles**: Primary, secondary, destructive, disabled, password toggles, etc.
- **5+ text input styles**: Standard, valid, invalid, neutral, title styles
- **Multiple container styles**: Error/warning/success/info alerts, toasts, modals, cards
- **Progress bar styles** with custom appearance
- **20+ named color constants** with complex color management

**Migration Impact:**
```rust
// Current StyleSheet implementation pattern
impl button::StyleSheet for PrimaryButtonStyle {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance { /* ... */ }
    fn hovered(&self, _style: &Self::Style) -> button::Appearance { /* ... */ }
    fn pressed(&self, _style: &Self::Style) -> button::Appearance { /* ... */ }
    fn disabled(&self, _style: &Self::Style) -> button::Appearance { /* ... */ }
}
```

All StyleSheet implementations need updates for Iced 0.13's class-based theming system.

### 3. Text Editor Integration Complexity

The credential form system makes heavy use of text editors:

**Current Usage:**
```rust
// Text editor content management
text_editor_content: HashMap<String, text_editor::Content>,

// Text editor actions in messages
TextEditorAction(String, text_editor::Action),
```

**Migration Requirements:**
- Text editor APIs may have changed significantly
- Content management patterns might need updates
- Action handling integration with the new Task system

### 4. Toast and Alert System Architecture

The app has a dual notification system with complex overlay rendering:

**Current Implementation:**
```rust
// Toast manager with auto-dismiss timing
toast_manager: ToastManager,

// Toast subscription for updates
let toast_subscription = if self.toast_manager.has_toasts() {
    time::every(Duration::from_millis(100)).map(|_| Message::UpdateToasts)
} else {
    iced::Subscription::none()
};
```

**Migration Considerations:**
- Overlay rendering may need updates for new widget architecture
- Timer-based auto-dismiss subscriptions require validation
- Toast positioning and stacking behavior needs testing

### 5. TOTP Component Real-Time Updates

The TOTP field has sophisticated timing and update logic:

**Current Features:**
- Real-time countdown timers with second precision
- Subscription-based updates for token regeneration
- Copy-to-clipboard integration with timeout management
- Custom validation and display logic

**Migration Requirements:**
- Timer subscriptions need validation with new API
- Real-time updates must maintain accuracy
- Clipboard integration timing needs testing

### 6. Platform-Specific Integration Points

The Linux app has several platform-specific dependencies:

**Current Dependencies:**
- `nix` for Linux-specific system calls
- `freedesktop-desktop-entry` for desktop integration
- `arboard` for clipboard management
- `keyring` for secure credential storage
- Wayland/X11 feature flag support

**Migration Validation:**
- Platform layer changes in Iced 0.13 may affect integrations
- Clipboard behavior needs thorough testing
- Desktop environment integration requires validation

### 7. File Dialog Integration

File operations use `rfd` crate with async integration:

**Current Pattern:**
```rust
// File selection with async Command integration
Command::perform(
    async { rfd::AsyncFileDialog::new().pick_file().await },
    |result| Message::FileSelected(result)
)
```

**Migration Requirements:**
- File dialog integration needs updates for Task API
- Async file operation patterns require validation
- Path handling and validation logic needs testing

## ZipLock-Specific Migration Steps

### 1. Update Dependencies

**Cargo.toml changes:**
```toml
[workspace.dependencies]
# Update Iced version
iced = { version = "0.13" }

# Consider new features
iced = { version = "0.13", features = ["advanced", "tokio", "debug"] }
```

### 2. Password Input Enhancements

With 0.13's improved text input capabilities, we can enhance our password fields:

```rust
// Enhanced password input with better security and UX
fn password_input<'a>(
    value: &'a str, 
    is_visible: bool, 
    is_valid: bool,
    placeholder: &'a str
) -> Element<'a, Message> {
    let input = text_input(placeholder, value)
        .on_input(Message::PasswordChanged)
        .on_submit(Message::PasswordSubmitted)
        .secure(!is_visible)
        .style(if is_valid { 
            text_input_styles::valid() 
        } else { 
            text_input_styles::standard() 
        })
        .padding(text_input_padding());

    let toggle_button = button(if is_visible { 
        eye_off_icon() 
    } else { 
        eye_icon() 
    })
    .on_press(Message::TogglePasswordVisibility)
    .style(button_styles::password_toggle())
    .padding(password_toggle_padding());

    row![input, toggle_button]
        .spacing(10)
        .align_y(iced::Center)
        .into()
}
```

### 3. Enhanced Error Handling with New Widgets

Iced 0.13's `rich_text` widget can improve our error display system:

```rust
use iced::widget::{rich_text, text::Span};

fn render_enhanced_alert(alert: &AlertMessage) -> Element<'_, Message> {
    let icon = match alert.level {
        AlertLevel::Error => error_icon(),
        AlertLevel::Warning => warning_icon(),
        AlertLevel::Success => check_icon(),
        AlertLevel::Info => alert_icon(),
    };

    let content = rich_text![
        Span::new(&alert.title).font_weight(iced::font::Weight::Bold),
        Span::new("\n"),
        Span::new(&alert.message),
    ];

    let dismiss_button = if alert.dismissible {
        Some(
            button(xmark_icon())
                .on_press(Message::DismissAlert)
                .style(button_styles::toast_close_button())
                .padding(toast_dismiss_padding())
        )
    } else {
        None
    };

    container(
        row![
            icon,
            content,
            if let Some(button) = dismiss_button {
                Some(button.into())
            } else {
                None
            }
        ]
        .spacing(standard_spacing())
        .align_y(iced::Center)
    )
    .style(match alert.level {
        AlertLevel::Error => container_styles::error_toast(),
        AlertLevel::Warning => container_styles::warning_toast(),
        AlertLevel::Success => container_styles::success_toast(),
        AlertLevel::Info => container_styles::info_toast(),
    })
    .padding(alert_padding())
    .into()
}
```

### 4. Archive Operations with Task Enhancements

Leverage 0.13's abortable tasks for better archive operations:

```rust
impl ZipLockApp {
    fn handle_archive_operations(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenArchive(path) => {
                // Cancel any existing operations
                if let Some(handle) = &self.current_operation {
                    handle.abort();
                }

                let password = self.password.clone();
                let (task, handle) = Task::perform(
                    async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        archive::open_archive(path, password).await
                    },
                    |result| Message::ArchiveOpened(result)
                ).abortable();

                self.current_operation = Some(handle.abort_on_drop());
                self.loading_state = LoadingState::OpeningArchive;

                task
            }
            
            Message::SaveCredentials => {
                if let Some(ref archive) = self.archive {
                    let credentials = self.credentials.clone();
                    let archive_path = archive.path.clone();
                    
                    Task::perform(
                        archive::save_all_credentials(archive_path, credentials),
                        |result| Message::CredentialsSaved(result)
                    )
                } else {
                    Task::none()
                }
            }

            _ => Task::none()
        }
    }
}
```

## New Features and Opportunities

### 1. Enhanced Credential Entry with Rich Text

```rust
// Use markdown widget for credential notes
fn credential_notes_view(notes: &str) -> Element<'_, Message> {
    if notes.trim().is_empty() {
        text("No notes").style(color!(0x888888)).into()
    } else {
        // Rich text support for formatted notes
        iced::widget::markdown(notes)
            .style(|theme| iced::widget::markdown::Style {
                text: theme.extended_palette().background.base.text,
                code_block: iced::widget::markdown::CodeBlock {
                    background: theme.extended_palette().background.weak.color,
                    border_radius: border_radius().into(),
                    padding: [8.0; 4].into(),
                },
                ..Default::default()
            })
            .into()
    }
}
```

### 2. Improved TOTP Integration

```rust
// Enhanced TOTP display with countdown using new widgets
fn totp_display(totp: &TOTPConfig, remaining: u32) -> Element<'_, Message> {
    let progress = (remaining as f32 / 30.0) * 100.0;
    
    column![
        row![
            text(&totp.current_code).size(32).font(iced::font::Family::Monospace),
            button("Copy").on_press(Message::CopyTOTP(totp.current_code.clone()))
        ]
        .spacing(10)
        .align_y(iced::Center),
        
        progress_bar(0.0..=100.0, progress)
            .style(progress_bar_styles::primary()),
            
        text(format!("Refreshes in {}s", remaining))
            .size(12)
            .style(color!(0x666666))
    ]
    .spacing(8)
    .into()
}
```

### 3. Modal Dialogs with Stack Widget

```rust
// Enhanced modal system using stack widget
fn render_with_modals(&self) -> Element<'_, Message> {
    let main_view = match &self.current_view {
        View::CredentialList => self.credential_list_view(),
        View::CredentialDetail(id) => self.credential_detail_view(*id),
        View::Settings => self.settings_view(),
    };

    stack![
        main_view,
        
        // Modal overlays
        if self.show_delete_confirmation {
            Some(self.delete_confirmation_modal())
        } else { None },
        
        if self.show_password_generator {
            Some(self.password_generator_modal())
        } else { None },
        
        if let Some(ref error) = self.modal_error {
            Some(self.error_modal(error))
        } else { None }
    ]
    .into()
}

fn delete_confirmation_modal(&self) -> Element<'_, Message> {
    container(
        container(
            column![
                text("Delete Credential").size(20),
                text("Are you sure you want to delete this credential? This action cannot be undone."),
                row![
                    button("Cancel")
                        .on_press(Message::CancelDelete)
                        .style(button_styles::secondary()),
                    button("Delete")
                        .on_press(Message::ConfirmDelete)
                        .style(button_styles::destructive())
                ]
                .spacing(10)
            ]
            .spacing(15)
            .max_width(400)
        )
        .style(container_styles::modal())
        .padding(20)
    )
    .width(iced::Fill)
    .height(iced::Fill)
    .center_x()
    .center_y()
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
        ..Default::default()
    })
    .into()
}
```

## Migration Checklist

### Pre-Migration

- [ ] **Backup current codebase** and create migration branch
- [ ] **Review all custom widgets** and identify 0.13 equivalents
- [ ] **Document current theming system** for reference
- [ ] **Test current functionality** to establish baseline
- [ ] **Create comprehensive widget test suite** for all custom components
- [ ] **Document current subscription behavior** (auto-lock, toasts, activity tracking)
- [ ] **Test file dialog integration** thoroughly across different environments
- [ ] **Validate toast/alert positioning** across different screen sizes and DPIs
- [ ] **Document TOTP timing behavior** for validation after migration

### Core Migration

- [ ] **Update Cargo.toml** dependencies to Iced 0.13
- [ ] **Replace Application trait** with new function-based approach
- [ ] **Convert all Command usage** to Task API
- [ ] **Update main.rs** application startup
- [ ] **Migrate custom themes** to new class-based system

### Widget Updates

- [ ] **Update all alignment calls** from `align_items` to `align_x`/`align_y`
- [ ] **Replace custom button styles** with new class system where possible
- [ ] **Update text input styling** approach
- [ ] **Implement new helper functions** (`center`, `center_x`, etc.)
- [ ] **Add stack widgets** for modal overlays
- [ ] **Migrate all 8+ custom button styles** incrementally (primary, secondary, destructive, etc.)
- [ ] **Update all 5+ text input styles** (standard, valid, invalid, neutral, title)
- [ ] **Convert container styles** for alerts, toasts, modals, and cards
- [ ] **Update progress bar styling** implementation

### ZipLock-Specific

- [ ] **Update password visibility** toggle implementation
- [ ] **Enhance error display** with rich text widgets
- [ ] **Improve TOTP display** with progress indicators
- [ ] **Migrate archive operations** to use abortable tasks
- [ ] **Update alert/toast system** with new styling
- [ ] **Migrate complex subscription system** (6+ subscription sources)
- [ ] **Update text editor integration** in credential forms
- [ ] **Validate file dialog async operations** with Task API
- [ ] **Test platform-specific integrations** (clipboard, keyring, desktop entry)
- [ ] **Update timer-based subscriptions** (auto-lock, toast updates, TOTP counters)

### Testing & Polish

- [ ] **Verify all UI layouts** render correctly
- [ ] **Test all async operations** work with Task API
- [ ] **Validate custom themes** display properly
- [ ] **Check accessibility** hasn't regressed
- [ ] **Performance test** with large archives
- [ ] **Comprehensive theme consistency check** across all 20+ custom styles
- [ ] **Platform integration testing** on multiple Linux distributions (Ubuntu, Fedora, Arch)
- [ ] **File dialog functionality verification** in different desktop environments
- [ ] **Toast overlay positioning validation** across various screen configurations
- [ ] **TOTP timing accuracy verification** (ensure counters remain precise)
- [ ] **Auto-lock functionality testing** with various timeout configurations
- [ ] **Clipboard integration validation** with different clipboard managers

### Documentation

- [ ] **Update README** with new build instructions
- [ ] **Document new theme system** in design.md
- [ ] **Update development guide** with 0.13 patterns
- [ ] **Create troubleshooting guide** for common migration issues

## Testing Strategy

### 1. Pre-Migration Testing Baseline

Before starting the migration, establish comprehensive test coverage for current functionality:

**Theme System Testing:**
```rust
// Test all current button styles render correctly
#[cfg(test)]
mod theme_tests {
    use super::*;
    
    #[test]
    fn test_all_button_styles_render() {
        // Test primary, secondary, destructive, disabled, password toggles
        // Validate colors, borders, hover states
    }
    
    #[test] 
    fn test_text_input_validation_styles() {
        // Test standard, valid, invalid, neutral, title styles
        // Ensure validation colors are correct
    }
}
```

**Subscription System Testing:**
```rust
#[test]
fn test_subscription_system_coverage() {
    // Verify all 6 subscription types are properly batched
    // Test timer accuracy for auto-lock and toasts
    // Validate event handling for user activity
}
```

**File Dialog Integration Testing:**
```rust
#[test] 
async fn test_file_dialog_async_integration() {
    // Test file selection with current Command system
    // Validate path handling and validation
    // Test cancellation behavior
}
```

### 2. Migration Phase Testing

**Incremental Theme Migration Testing:**
- Test one style type at a time (buttons first, then text inputs, then containers)
- Validate color consistency across all themes
- Test hover/pressed/disabled states for each component
- Verify accessibility compliance maintained

**Subscription System Validation:**
- Test each subscription type individually after migration
- Validate timer accuracy (especially for TOTP and auto-lock)
- Test subscription batching behavior
- Verify event handling still works correctly

**Text Editor Migration Testing:**
- Test all credential form functionality
- Validate text editor content persistence
- Test action handling integration
- Verify copy/paste operations work correctly

### 3. Platform-Specific Testing

**Linux Distribution Testing:**
- Ubuntu 22.04/24.04 LTS (primary target)
- Fedora 39/40 (secondary target) 
- Arch Linux (rolling release validation)
- Test both Wayland and X11 environments

**Desktop Environment Testing:**
- GNOME (primary DE)
- KDE Plasma (secondary DE)
- XFCE (lightweight DE validation)
- Test file dialog behavior in each environment

### 4. Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use iced_test::{Simulator, Error};

    #[test]
    fn test_password_input_visibility_toggle() -> Result<(), Error> {
        let mut app = ZipLockApp::default();
        let mut simulator = Simulator::new(app.view());
        
        // Test password visibility toggle
        let toggle_button = simulator.find_by_accessibility_label("Toggle password visibility")?;
        toggle_button.click();
        
        // Verify state change
        assert!(app.password_visible);
        
        Ok(())
    }
    
    #[test]
    fn test_task_cancellation() -> Result<(), Error> {
        let mut app = ZipLockApp::default();
        
        // Start a long-running operation
        let task = app.update(Message::OpenArchive(PathBuf::from("test.7z")));
        
        // Cancel it
        if let Some(handle) = &app.current_operation {
            handle.abort();
        }
        
        // Verify cancellation
        assert!(app.current_operation.is_none());
        
        Ok(())
    }
}
```

### 2. Integration Tests
- Test archive opening/closing with new Task system
- Verify theme changes don't break existing layouts
- Ensure all modals render correctly with stack widgets
- Test TOTP functionality with enhanced widgets

### 3. Visual Regression Tests
- Compare screenshots before and after migration
- Verify color consistency with new theming system
- Check layout alignment with new helper functions

## Rollback Plan

### If Critical Issues Arise:

1. **Immediate Rollback**
   ```bash
   git checkout main
   git revert migration-commit-hash
   cargo build --release
   ```

2. **Partial Rollback Options**
   - Keep new Task API, revert theme changes
   - Keep theme updates, revert to Application trait
   - Identify specific problematic changes and revert incrementally

3. **Contingency Plan**
   - Maintain 0.12 branch as fallback
   - Document known issues for future migration attempt
   - Consider gradual migration over multiple releases

### Known Risk Areas:
- **Custom theme compatibility** with extended palette system (20+ styles to migrate)
- **Archive operation reliability** with abortable tasks
- **Performance regression** with new widget system
- **Platform-specific rendering** changes
- **Subscription system timing accuracy** (auto-lock and TOTP precision critical)
- **Text editor integration complexity** in credential forms
- **Toast overlay positioning** across different screen configurations
- **File dialog async operation** integration with new Task API
- **Platform-specific integrations** (clipboard, keyring, desktop entry compatibility)
- **Complex widget composition** patterns in credential management views

## Conclusion

The migration to Iced 0.13 offers significant improvements in developer experience and application capabilities, particularly beneficial for ZipLock's complex UI requirements. The new Task API provides better async operation management, the enhanced theming system offers more flexibility, and new widgets enable richer user experiences.

While the migration involves substantial changes, particularly the removal of the Sandbox trait and Command system, the benefits of improved maintainability, better performance, and enhanced features make this upgrade worthwhile.

Key success factors:
1. **Thorough testing** at each migration step
2. **Incremental changes** with frequent verification  
3. **Maintaining UI/UX consistency** throughout the process
4. **Documenting lessons learned** for future upgrades
5. **Platform-specific validation** across multiple Linux distributions
6. **Theme migration coordination** to maintain visual consistency
7. **Subscription system precision** for timing-critical features

**Revised Timeline Estimate:** 3-4 weeks for complete implementation and testing, broken down as:
- **Week 1:** Core Application trait and Task migration
- **Week 2:** Theme system migration (incremental, by component type)
- **Week 3:** Complex component migration (TOTP, credential forms, file dialogs)
- **Week 4:** Platform testing, polish, and validation across distributions

The Linux app's complexity requires additional time for the extensive theming system and platform-specific integrations, with potential for staged rollout to minimize risk.