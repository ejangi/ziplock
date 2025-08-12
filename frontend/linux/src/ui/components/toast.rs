//! Toast Component Module
//!
//! This module provides a centralized toast notification system for the ZipLock Linux frontend.
//! Toasts are overlay notifications that appear temporarily to provide user feedback.

use iced::{
    widget::{button, column, container, row, svg, text, Space},
    Alignment, Element, Length,
};
use std::time::{Duration, Instant};

use crate::ui::theme::{
    alert_icon,
    alerts::{AlertLevel, AlertMessage},
    button_styles, check_icon, container_styles, error_icon, utils, warning_icon, DARK_TEXT,
    ERROR_RED, LOGO_PURPLE, SUCCESS_GREEN, WARNING_YELLOW,
};

/// Duration for toast auto-dismiss (in seconds)
pub const DEFAULT_TOAST_DURATION: Duration = Duration::from_secs(5);

/// Maximum number of toasts to display simultaneously
pub const MAX_VISIBLE_TOASTS: usize = 3;

/// Toast positioning and spacing constants
pub const TOAST_MARGIN: f32 = 20.0;
pub const TOAST_SPACING: f32 = 10.0;
pub const TOAST_MIN_WIDTH: f32 = 300.0;
pub const TOAST_MAX_WIDTH: f32 = 500.0;

/// Position where toasts should appear
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    TopCenter,
    BottomCenter,
}

impl Default for ToastPosition {
    fn default() -> Self {
        ToastPosition::BottomRight
    }
}

/// Individual toast item with timing information
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: usize,
    pub message: AlertMessage,
    pub created_at: Instant,
    pub duration: Duration,
    pub auto_dismiss: bool,
}

impl Toast {
    /// Create a new toast with auto-dismiss
    pub fn new(id: usize, message: AlertMessage) -> Self {
        Self {
            id,
            message,
            created_at: Instant::now(),
            duration: DEFAULT_TOAST_DURATION,
            auto_dismiss: true,
        }
    }

    /// Create a new persistent toast (no auto-dismiss)
    pub fn persistent(id: usize, message: AlertMessage) -> Self {
        Self {
            id,
            message,
            created_at: Instant::now(),
            duration: DEFAULT_TOAST_DURATION,
            auto_dismiss: false,
        }
    }

    /// Create a toast with custom duration
    pub fn with_duration(id: usize, message: AlertMessage, duration: Duration) -> Self {
        Self {
            id,
            message,
            created_at: Instant::now(),
            duration,
            auto_dismiss: true,
        }
    }

    /// Check if this toast should be auto-dismissed
    pub fn should_dismiss(&self) -> bool {
        self.auto_dismiss && self.created_at.elapsed() >= self.duration
    }

    /// Get the remaining time for this toast
    pub fn remaining_time(&self) -> Duration {
        if self.auto_dismiss {
            self.duration.saturating_sub(self.created_at.elapsed())
        } else {
            Duration::MAX
        }
    }

    /// Calculate opacity based on remaining time (for fade-out effect)
    pub fn opacity(&self) -> f32 {
        if !self.auto_dismiss {
            return 1.0;
        }

        let elapsed = self.created_at.elapsed();
        let fade_duration = Duration::from_millis(500); // 500ms fade out

        if elapsed <= self.duration.saturating_sub(fade_duration) {
            1.0
        } else {
            let fade_progress = (self.duration.saturating_sub(elapsed)).as_millis() as f32
                / fade_duration.as_millis() as f32;
            fade_progress.max(0.0).min(1.0)
        }
    }
}

/// Toast manager for handling multiple toasts
#[derive(Debug, Clone)]
pub struct ToastManager {
    toasts: Vec<Toast>,
    next_id: usize,
    position: ToastPosition,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    /// Create a new toast manager
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
            position: ToastPosition::default(),
        }
    }

    /// Create a toast manager with custom position
    pub fn with_position(position: ToastPosition) -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
            position,
        }
    }

    /// Add a new toast
    pub fn add_toast(&mut self, message: AlertMessage) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let toast = Toast::new(id, message);
        self.toasts.push(toast);

        // Limit the number of visible toasts
        if self.toasts.len() > MAX_VISIBLE_TOASTS {
            self.toasts.remove(0);
        }

        id
    }

    /// Add a persistent toast (no auto-dismiss)
    pub fn add_persistent_toast(&mut self, message: AlertMessage) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let toast = Toast::persistent(id, message);
        self.toasts.push(toast);

        // Limit the number of visible toasts
        if self.toasts.len() > MAX_VISIBLE_TOASTS {
            self.toasts.remove(0);
        }

        id
    }

    /// Add a toast with custom duration
    pub fn add_toast_with_duration(&mut self, message: AlertMessage, duration: Duration) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let toast = Toast::with_duration(id, message, duration);
        self.toasts.push(toast);

        // Limit the number of visible toasts
        if self.toasts.len() > MAX_VISIBLE_TOASTS {
            self.toasts.remove(0);
        }

        id
    }

    /// Remove a specific toast by ID
    pub fn remove_toast(&mut self, toast_id: usize) {
        self.toasts.retain(|toast| toast.id != toast_id);
    }

    /// Remove all expired toasts
    pub fn remove_expired_toasts(&mut self) {
        self.toasts.retain(|toast| !toast.should_dismiss());
    }

    /// Clear all toasts
    pub fn clear_all(&mut self) {
        self.toasts.clear();
    }

    /// Get all current toasts
    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }

    /// Check if there are any toasts
    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    /// Get the number of active toasts
    pub fn count(&self) -> usize {
        self.toasts.len()
    }

    /// Set the toast position
    pub fn set_position(&mut self, position: ToastPosition) {
        self.position = position;
    }

    /// Get the current toast position
    pub fn position(&self) -> ToastPosition {
        self.position
    }
}

/// Convenience functions for common toast types
impl ToastManager {
    /// Add an error toast
    pub fn error<S: Into<String>>(&mut self, message: S) -> usize {
        self.add_toast(AlertMessage::error(message))
    }

    /// Add a warning toast
    pub fn warning<S: Into<String>>(&mut self, message: S) -> usize {
        self.add_toast(AlertMessage::warning(message))
    }

    /// Add a success toast
    pub fn success<S: Into<String>>(&mut self, message: S) -> usize {
        self.add_toast(AlertMessage::success(message))
    }

    /// Add an info toast
    pub fn info<S: Into<String>>(&mut self, message: S) -> usize {
        self.add_toast(AlertMessage::info(message))
    }

    /// Add an IPC error toast
    pub fn ipc_error<S: Into<String>>(&mut self, message: S) -> usize {
        self.add_toast(AlertMessage::ipc_error(message))
    }
}

/// Render a single toast
pub fn render_toast<Message: Clone + 'static>(
    toast: &Toast,
    on_dismiss: Option<Message>,
) -> Element<Message> {
    let container_style = match toast.message.level {
        AlertLevel::Error => container_styles::error_alert(),
        AlertLevel::Warning => container_styles::warning_alert(),
        AlertLevel::Success => container_styles::success_alert(),
        AlertLevel::Info => container_styles::info_alert(),
    };

    let icon_svg = match toast.message.level {
        AlertLevel::Error => error_icon(),
        AlertLevel::Warning => warning_icon(),
        AlertLevel::Success => check_icon(),
        AlertLevel::Info => alert_icon(),
    };

    let mut content = row![svg(icon_svg).width(16).height(16)];

    let mut text_column = column![];

    if let Some(title) = &toast.message.title {
        let title_color = match toast.message.level {
            AlertLevel::Error => ERROR_RED,
            AlertLevel::Warning => WARNING_YELLOW,
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
        text(&toast.message.message)
            .size(12)
            .style(iced::theme::Text::Color(DARK_TEXT)),
    );

    content = content
        .push(Space::with_width(Length::Fixed(10.0)))
        .push(text_column.width(Length::Fill).spacing(4));

    // Add dismiss button if dismissible or manual dismiss is provided
    if toast.message.dismissible || on_dismiss.is_some() {
        if let Some(dismiss_msg) = on_dismiss {
            content = content.push(Space::with_width(Length::Fixed(10.0))).push(
                button("✕")
                    .on_press(dismiss_msg)
                    .padding([2, 6])
                    .style(button_styles::secondary()),
            );
        }
    }

    let toast_container = container(content.align_items(Alignment::Center))
        .padding(utils::alert_padding())
        .width(Length::Fixed(
            TOAST_MIN_WIDTH.max(TOAST_MAX_WIDTH.min(400.0)),
        ))
        .style(container_style);

    // Apply opacity for fade effect
    let opacity = toast.opacity();
    if opacity < 1.0 {
        // Note: Iced doesn't have built-in opacity support for containers
        // This would need to be implemented with custom styling or shader effects
        // For now, we'll just return the container as-is
        toast_container.into()
    } else {
        toast_container.into()
    }
}

/// Render all toasts in a toast manager
pub fn render_toasts<Message: Clone + 'static>(
    toast_manager: &ToastManager,
    on_dismiss: impl Fn(usize) -> Message,
) -> Element<Message> {
    if !toast_manager.has_toasts() {
        return Space::new(Length::Shrink, Length::Shrink).into();
    }

    let mut toast_column = column![];

    for toast in toast_manager.toasts() {
        let dismiss_message = on_dismiss(toast.id);
        let toast_element = render_toast(toast, Some(dismiss_message));
        toast_column = toast_column.push(toast_element);
        toast_column = toast_column.push(Space::with_height(Length::Fixed(TOAST_SPACING)));
    }

    // Remove the last spacing
    if toast_manager.count() > 0 {
        // Note: This is a simplified approach - in a real implementation,
        // we might want to handle spacing more elegantly
    }

    let positioned_toasts = match toast_manager.position() {
        ToastPosition::TopRight | ToastPosition::BottomRight => {
            container(toast_column.align_items(Alignment::End))
        }
        ToastPosition::TopLeft | ToastPosition::BottomLeft => {
            container(toast_column.align_items(Alignment::Start))
        }
        ToastPosition::TopCenter | ToastPosition::BottomCenter => {
            container(toast_column.align_items(Alignment::Center))
        }
    };

    positioned_toasts
        .width(Length::Fill)
        .height(Length::Shrink)
        .padding(TOAST_MARGIN)
        .into()
}

/// Toast overlay that can be positioned over the main content
pub fn render_toast_overlay<'a, Message: Clone + 'static>(
    toast_manager: &'a ToastManager,
    main_content: Element<'a, Message>,
    on_dismiss: impl Fn(usize) -> Message,
) -> Element<'a, Message> {
    if !toast_manager.has_toasts() {
        return main_content;
    }

    // For bottom-right positioning, append toasts after main content
    // This creates a floating effect in the bottom right corner
    if toast_manager.position() == ToastPosition::BottomRight {
        let toasts = render_toasts(toast_manager, on_dismiss);

        // Create a toast container positioned at bottom-right
        let toast_container = container(
            container(toasts)
                .width(Length::Shrink)
                .height(Length::Shrink),
        )
        .width(Length::Fill)
        .height(Length::Shrink)
        .align_x(iced::alignment::Horizontal::Right)
        .padding([0.0, TOAST_MARGIN, TOAST_MARGIN, 0.0]);

        // Return main content with toasts floating at bottom
        column![main_content, toast_container].into()
    } else {
        // For other positions, use the previous overlay approach
        let toasts = render_toasts(toast_manager, on_dismiss);

        let positioned_toasts = match toast_manager.position() {
            ToastPosition::TopRight => container(
                container(toasts)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .padding([TOAST_MARGIN, TOAST_MARGIN, 0.0, 0.0]),
            )
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(iced::alignment::Horizontal::Right),
            ToastPosition::TopLeft => container(
                container(toasts)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .padding([TOAST_MARGIN, 0.0, 0.0, TOAST_MARGIN]),
            )
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(iced::alignment::Horizontal::Left),
            ToastPosition::BottomLeft => container(
                container(toasts)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .padding([0.0, 0.0, TOAST_MARGIN, TOAST_MARGIN]),
            )
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(iced::alignment::Horizontal::Left),
            ToastPosition::TopCenter => container(
                container(toasts)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .padding([TOAST_MARGIN, 0.0, 0.0, 0.0]),
            )
            .width(Length::Fill)
            .height(Length::Shrink)
            .center_x(),
            ToastPosition::BottomCenter => container(
                container(toasts)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .padding([0.0, 0.0, TOAST_MARGIN, 0.0]),
            )
            .width(Length::Fill)
            .height(Length::Shrink)
            .center_x(),
            ToastPosition::BottomRight => unreachable!(), // Handled above
        };

        match toast_manager.position() {
            ToastPosition::BottomLeft | ToastPosition::BottomCenter => {
                column![main_content, positioned_toasts].into()
            }
            _ => column![positioned_toasts, main_content].into(),
        }
    }
}
