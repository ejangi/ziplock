//! TOTP field component for displaying time-based one-time passwords
//!
//! This component handles the display of TOTP fields, showing either the generated
//! 6-digit code or the underlying secret key with proper visibility controls.
//!
//! ## Synchronization with SaaS Services
//!
//! The TOTP implementation ensures proper synchronization with external SaaS login
//! prompts by:
//!
//! 1. **Time Boundary Alignment**: TOTP codes are generated based on 30-second
//!    intervals starting from Unix epoch, matching RFC 6238 standard that most
//!    services follow.
//!
//! 2. **System Clock Synchronization**: The countdown timer displays the actual
//!    time remaining until the next TOTP boundary (calculated from system time),
//!    not just the time since the last code generation.
//!
//! 3. **Precise Refresh Timing**: Uses a high-frequency timer (100ms) to detect
//!    the exact moment when crossing TOTP time boundaries, ensuring codes refresh
//!    at the precise moment SaaS services expect them to change.
//!
//! This ensures that when a user sees "5 seconds remaining" in ZipLock, the SaaS
//! service they're logging into will also expect the current code to be valid for
//! exactly 5 more seconds.

use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length, Subscription};
use std::time::{Duration, Instant};

use crate::ui::theme::{button_styles, utils, LIGHT_GRAY_TEXT};

use ziplock_shared::utils::totp;

use crate::ui::theme;

/// Messages for the TOTP field component
#[derive(Debug, Clone, PartialEq)]
pub enum TotpFieldMessage {
    /// Toggle between showing code and secret
    ToggleVisibility,
    /// Update the secret value
    SecretChanged(String),
    /// Refresh the TOTP code (triggered by timer)
    RefreshCode,
    /// Copy the current code to clipboard
    CopyCode,
}

/// Display mode for the TOTP field
#[derive(Debug, Clone, PartialEq)]
pub enum TotpDisplayMode {
    /// Show the generated 6-digit code
    Code,
    /// Show the underlying secret key
    Secret,
}

/// TOTP field component state
#[derive(Debug, Clone)]
pub struct TotpField {
    /// The base32-encoded TOTP secret
    secret: String,
    /// Current display mode
    display_mode: TotpDisplayMode,
    /// Currently generated TOTP code
    current_code: Option<String>,
    /// When the code was last generated
    last_refresh: Option<Instant>,
    /// Whether the field is in edit mode
    is_editing: bool,
    /// Field name for form submission
    field_name: String,
    /// Time step for TOTP generation (usually 30 seconds)
    time_step: u64,
}

impl TotpField {
    /// Create a new TOTP field
    pub fn new(field_name: String, secret: String) -> Self {
        let mut field = Self {
            secret: secret.clone(),
            // Show secret input if empty, code view if populated
            display_mode: if secret.trim().is_empty() {
                TotpDisplayMode::Secret
            } else {
                TotpDisplayMode::Code
            },
            current_code: None,
            last_refresh: None,
            is_editing: false,
            field_name,
            time_step: 30,
        };

        // Generate initial code if secret is valid
        field.refresh_code();
        field
    }

    /// Create a new TOTP field in edit mode
    pub fn new_editing(field_name: String, secret: String) -> Self {
        let mut field = Self::new(field_name, secret);
        field.is_editing = true;
        // In edit mode, always start with secret view if empty
        if field.secret.trim().is_empty() {
            field.display_mode = TotpDisplayMode::Secret;
        }
        field
    }

    /// Get the current secret value
    pub fn secret(&self) -> &str {
        &self.secret
    }

    /// Set the secret value
    pub fn set_secret(&mut self, secret: String) {
        self.secret = secret.clone();
        self.refresh_code();

        // Auto-switch display mode based on secret content
        if secret.trim().is_empty() {
            self.display_mode = TotpDisplayMode::Secret;
        } else if self.current_code.is_some() {
            // Only switch to code mode if we successfully generated a code
            self.display_mode = TotpDisplayMode::Code;
        }
    }

    /// Check if the secret is valid
    pub fn is_valid_secret(&self) -> bool {
        if self.secret.trim().is_empty() {
            return true; // Empty is valid (optional field)
        }
        totp::validate_totp_secret(&self.secret)
    }

    /// Refresh the TOTP code
    ///
    /// This method generates a new TOTP code based on the current system time.
    /// The generated code is synchronized with TOTP time boundaries (30-second intervals
    /// starting from Unix epoch), ensuring compatibility with SaaS login prompts.
    fn refresh_code(&mut self) {
        if self.secret.trim().is_empty() {
            self.current_code = None;
            self.last_refresh = None;
            return;
        }

        match totp::generate_totp(&self.secret, self.time_step) {
            Ok(code) => {
                self.current_code = Some(code);
                self.last_refresh = Some(Instant::now());
            }
            Err(_) => {
                self.current_code = None;
                self.last_refresh = None;
            }
        }
    }

    /// Get seconds remaining until next refresh
    ///
    /// Returns the actual time remaining until the next TOTP time boundary,
    /// synchronized with the system clock. This ensures the countdown matches
    /// what SaaS services expect, rather than counting from when the code
    /// was last generated.
    fn seconds_until_refresh(&self) -> u64 {
        if self.secret.trim().is_empty() {
            return 0;
        }

        // Use the shared TOTP utility to get the actual time remaining
        // until the next TOTP boundary (synchronized with system clock)
        totp::get_seconds_until_refresh(self.time_step)
    }

    /// Check if the code needs refreshing
    ///
    /// Determines if a new TOTP code should be generated based on system time boundaries.
    /// This ensures synchronization with SaaS services by checking if we've crossed
    /// a TOTP time boundary (30-second intervals from Unix epoch) rather than just
    /// checking elapsed time since last generation.
    fn needs_refresh(&self) -> bool {
        if self.secret.trim().is_empty() {
            return false;
        }

        // Check if we need to refresh based on system time boundaries
        // This ensures we stay synchronized with what SaaS services expect
        if self.last_refresh.is_none() {
            return true; // No code generated yet
        }

        // Check if we've crossed a TOTP time boundary since last refresh
        let seconds_remaining = totp::get_seconds_until_refresh(self.time_step);
        let last_refresh_seconds_remaining = if let Some(last_refresh) = self.last_refresh {
            let elapsed = last_refresh.elapsed().as_secs();
            if elapsed >= self.time_step {
                // More than a full time step has elapsed, definitely need refresh
                return true;
            }
            self.time_step - elapsed
        } else {
            return true;
        };

        // If the actual system time boundary remaining is greater than what we
        // calculated based on our last refresh, it means we've crossed a boundary
        seconds_remaining > last_refresh_seconds_remaining
    }

    /// Update the component
    pub fn update(&mut self, message: TotpFieldMessage) {
        match message {
            TotpFieldMessage::ToggleVisibility => {
                self.display_mode = match self.display_mode {
                    TotpDisplayMode::Code => TotpDisplayMode::Secret,
                    TotpDisplayMode::Secret => {
                        // Only switch to code mode if we have a valid secret and code
                        if !self.secret.trim().is_empty() && self.current_code.is_some() {
                            TotpDisplayMode::Code
                        } else {
                            TotpDisplayMode::Secret // Stay in secret mode
                        }
                    }
                };
            }
            TotpFieldMessage::SecretChanged(new_secret) => {
                self.secret = new_secret;
                self.refresh_code();
            }
            TotpFieldMessage::RefreshCode => {
                // Check if we need to refresh based on system time boundaries
                if self.needs_refresh() {
                    self.refresh_code();
                }
            }
            TotpFieldMessage::CopyCode => {
                if let Some(ref code) = self.current_code {
                    // Copy the unformatted code (without spaces) to clipboard
                    if let Err(e) = arboard::Clipboard::new()
                        .and_then(|mut clipboard| clipboard.set_text(code.clone()))
                    {
                        eprintln!("Failed to copy TOTP code to clipboard: {}", e);
                    }
                }
            }
        }
    }

    /// Create the view for this component
    pub fn view(&self) -> Element<'_, TotpFieldMessage> {
        if self.is_editing {
            self.view_editing()
        } else {
            self.view_display()
        }
    }

    /// View for editing mode (in forms)
    fn view_editing(&self) -> Element<'_, TotpFieldMessage> {
        let placeholder = match self.display_mode {
            TotpDisplayMode::Secret => {
                if self.secret.trim().is_empty() {
                    "Enter TOTP secret (base32)..."
                } else {
                    "Edit TOTP secret..."
                }
            }
            TotpDisplayMode::Code => {
                if self.secret.trim().is_empty() {
                    "No secret configured"
                } else {
                    "Generated code (switch to edit secret)"
                }
            }
        };

        let input_value = match self.display_mode {
            TotpDisplayMode::Secret => &self.secret,
            TotpDisplayMode::Code => {
                if let Some(ref code) = self.current_code {
                    code
                } else {
                    "------"
                }
            }
        };

        let is_readonly = self.display_mode == TotpDisplayMode::Code;
        let is_secret_mode = self.display_mode == TotpDisplayMode::Secret;

        let display_value =
            if self.display_mode == TotpDisplayMode::Code && self.current_code.is_some() {
                // Format code with space for readability in display
                let code = self.current_code.as_ref().unwrap();
                format!("{} {}", &code[..3], &code[3..])
            } else {
                input_value.to_string()
            };

        let input: Element<'_, TotpFieldMessage> = if is_readonly {
            // Create a button that looks like a text input for copyable TOTP codes
            button(text(&display_value).size(14))
                .on_press(TotpFieldMessage::CopyCode)
                .style(button_styles::text_field_like())
                .padding(utils::text_input_padding())
                .width(Length::Fill)
                .into()
        } else {
            text_input(placeholder, input_value)
                .on_input(TotpFieldMessage::SecretChanged)
                .padding(utils::text_input_padding())
                .style(crate::ui::theme::text_input_styles::standard())
                .into()
        };

        let eye_icon = if is_secret_mode {
            if self.secret.trim().is_empty() {
                "🔑"
            } else {
                "👁"
            }
        } else {
            "🔑"
        };
        let toggle_button =
            if self.secret.trim().is_empty() && self.display_mode == TotpDisplayMode::Secret {
                // No toggle when empty - just show the input icon
                button(eye_icon)
                    .style(button_styles::secondary())
                    .padding(utils::small_button_padding())
            } else {
                button(eye_icon)
                    .on_press(TotpFieldMessage::ToggleVisibility)
                    .style(button_styles::secondary())
                    .padding(utils::small_button_padding())
            };

        let mut row_elements = vec![input.into(), toggle_button.into()];

        // Add refresh indicator for code mode
        if self.display_mode == TotpDisplayMode::Code && self.current_code.is_some() {
            let remaining = self.seconds_until_refresh();
            let refresh_text = if remaining > 0 {
                format!("{}s", remaining)
            } else {
                "0s".to_string()
            };

            let color = if remaining <= 5 {
                theme::ERROR_RED
            } else if remaining <= 10 {
                theme::WARNING_YELLOW
            } else {
                theme::SUCCESS_GREEN
            };

            let refresh_indicator = container(
                text(refresh_text)
                    .size(12)
                    .style(iced::theme::Text::Color(color)),
            )
            .padding(utils::small_element_padding())
            .center_y();

            row_elements.push(refresh_indicator.into());
        }

        // Add validation indicator
        if !self.is_valid_secret() && !self.secret.trim().is_empty() {
            let error_indicator = container(
                text("❌")
                    .size(12)
                    .style(iced::theme::Text::Color(theme::ERROR_RED)),
            )
            .padding(utils::small_element_padding())
            .center_y();

            row_elements.push(error_indicator.into());
        }

        let main_row = row(row_elements).spacing(5).align_items(Alignment::Center);

        // Add help text for TOTP
        let help_text = if self.display_mode == TotpDisplayMode::Secret {
            if self.secret.trim().is_empty() {
                Some("Enter your TOTP secret key (usually provided as a QR code or text)")
            } else if self.current_code.is_some() {
                Some("Modify your TOTP secret key (click the eye to view generated codes)")
            } else {
                Some("Invalid TOTP secret - please check the format")
            }
        } else if self.current_code.is_some() {
            Some("6-digit code refreshes every 30 seconds - click to copy to clipboard")
        } else {
            Some("Invalid TOTP secret - switch to edit mode to fix")
        };

        if let Some(help) = help_text {
            column![
                main_row,
                Space::with_height(Length::Fixed(5.0)),
                text(help)
                    .size(11)
                    .style(iced::theme::Text::Color(LIGHT_GRAY_TEXT))
            ]
            .into()
        } else {
            main_row.into()
        }
    }

    /// View for display mode (in credential details)
    fn view_display(&self) -> Element<'_, TotpFieldMessage> {
        // If no secret, show input field instead of display
        if self.secret.trim().is_empty() {
            return self.view_editing();
        }

        let (display_element, is_code_display) = match self.display_mode {
            TotpDisplayMode::Code => {
                if let Some(ref code) = self.current_code {
                    let formatted_code = format!("{} {}", &code[..3], &code[3..]);
                    // Use a button that looks like a text input for copyable TOTP codes
                    let code_button = button(text(&formatted_code).size(14))
                        .on_press(TotpFieldMessage::CopyCode)
                        .style(button_styles::text_field_like())
                        .padding(10)
                        .width(Length::Fill);
                    (code_button.into(), true)
                } else {
                    let error_text = text("Invalid secret")
                        .size(14)
                        .style(iced::theme::Text::Color(theme::ERROR_RED));
                    (error_text.into(), false)
                }
            }
            TotpDisplayMode::Secret => {
                let formatted_secret = totp::format_totp_secret(&self.secret);
                let secret_text = text(&formatted_secret)
                    .size(14)
                    .style(iced::theme::Text::Color(theme::DARK_TEXT));
                (secret_text.into(), false)
            }
        };

        let eye_icon = if self.display_mode == TotpDisplayMode::Secret {
            "👁"
        } else {
            "🔑"
        };

        let toggle_button = button(eye_icon)
            .on_press(TotpFieldMessage::ToggleVisibility)
            .style(button_styles::secondary())
            .padding(utils::small_button_padding());

        let mut row_elements = vec![
            display_element,
            Space::with_width(Length::Fixed(10.0)).into(),
            toggle_button.into(),
        ];

        // Note: Copy functionality is now built into the code display button itself

        // Add refresh indicator
        if is_code_display {
            let remaining = self.seconds_until_refresh();
            let color = if remaining <= 5 {
                theme::ERROR_RED
            } else if remaining <= 10 {
                theme::WARNING_YELLOW
            } else {
                theme::SUCCESS_GREEN
            };

            let refresh_indicator = container(
                text(format!("{}s", remaining))
                    .size(12)
                    .style(iced::theme::Text::Color(color)),
            )
            .padding(utils::small_element_padding())
            .center_y();

            row_elements.push(refresh_indicator.into());
        }

        row(row_elements)
            .spacing(5)
            .align_items(Alignment::Center)
            .into()
    }

    /// Get subscription for automatic refresh
    pub fn subscription(&self) -> Subscription<TotpFieldMessage> {
        if !self.secret.trim().is_empty() && self.current_code.is_some() {
            // Use a more frequent timer to ensure we catch the exact moment
            // when the TOTP boundary is crossed for precise synchronization
            iced::time::every(Duration::from_millis(100)).map(|_| TotpFieldMessage::RefreshCode)
        } else {
            Subscription::none()
        }
    }
}
