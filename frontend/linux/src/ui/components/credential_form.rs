//! Reusable Credential Form Component
//!
//! This module provides a reusable form component for creating and editing credentials.
//! It handles field rendering, validation, and user interaction for credential data.

use iced::{
    alignment::Alignment,
    widget::{button, column, row, scrollable, text, text_input, Space},
    Element, Length,
};
use std::collections::HashMap;

use crate::ui::theme::button_styles;
use ziplock_shared::models::{CredentialTemplate, FieldType};

/// Messages that can be sent from the credential form
#[derive(Debug, Clone, PartialEq)]
pub enum CredentialFormMessage {
    /// The title field was changed
    TitleChanged(String),
    /// A specific field value was changed
    FieldChanged(String, String),
    /// Toggle the sensitivity (show/hide) of a field
    ToggleFieldSensitivity(String),
    /// The save button was pressed
    Save,
    /// The cancel button was pressed
    Cancel,
    /// The delete button was pressed
    Delete,
}

/// Configuration for the credential form component
#[derive(Debug, Clone)]
pub struct CredentialFormConfig {
    /// The text to display on the save button (e.g., "Save", "Create Credential")
    pub save_button_text: String,
    /// Whether to show the cancel button
    pub show_cancel_button: bool,
    /// Whether to show the delete button
    pub show_delete_button: bool,
    /// Whether the form is in a loading state
    pub is_loading: bool,
    /// Optional error message to display
    pub error_message: Option<String>,
}

impl Default for CredentialFormConfig {
    fn default() -> Self {
        Self {
            save_button_text: "Save".to_string(),
            show_cancel_button: true,
            show_delete_button: false,
            is_loading: false,
            error_message: None,
        }
    }
}

/// The credential form component
#[derive(Debug, Clone)]
pub struct CredentialForm {
    /// The credential template to render
    template: Option<CredentialTemplate>,
    /// The current title value
    title: String,
    /// Current field values
    field_values: HashMap<String, String>,
    /// Field sensitivity state (whether to show or hide sensitive fields)
    field_sensitivity: HashMap<String, bool>,
    /// Form configuration
    config: CredentialFormConfig,
}

impl Default for CredentialForm {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialForm {
    /// Create a new empty credential form
    pub fn new() -> Self {
        Self {
            template: None,
            title: String::new(),
            field_values: HashMap::new(),
            field_sensitivity: HashMap::new(),
            config: CredentialFormConfig::default(),
        }
    }

    /// Create a credential form with a specific template
    pub fn with_template(template: CredentialTemplate) -> Self {
        let mut form = Self::new();
        form.set_template(template);
        form
    }

    /// Set the credential template for the form
    pub fn set_template(&mut self, template: CredentialTemplate) {
        // Initialize field sensitivity based on template defaults
        for field_template in &template.fields {
            self.field_sensitivity
                .insert(field_template.name.clone(), field_template.sensitive);
        }
        self.template = Some(template);
    }

    /// Set the form configuration
    pub fn set_config(&mut self, config: CredentialFormConfig) {
        self.config = config;
    }

    /// Set the title value
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Set a field value
    pub fn set_field_value(&mut self, field_name: String, value: String) {
        self.field_values.insert(field_name, value);
    }

    /// Set multiple field values at once
    pub fn set_field_values(&mut self, values: HashMap<String, String>) {
        self.field_values = values;
    }

    /// Get the current title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the current field values
    pub fn field_values(&self) -> &HashMap<String, String> {
        &self.field_values
    }

    /// Update the form based on a message
    pub fn update(&mut self, message: CredentialFormMessage) {
        match message {
            CredentialFormMessage::TitleChanged(title) => {
                tracing::debug!("Title changed to: '{}'", title);
                self.title = title;
            }
            CredentialFormMessage::FieldChanged(field_name, value) => {
                tracing::debug!("Field '{}' changed to: '{}'", field_name, value);
                self.field_values.insert(field_name, value);
            }
            CredentialFormMessage::ToggleFieldSensitivity(field_name) => {
                if let Some(sensitive) = self.field_sensitivity.get(&field_name).copied() {
                    tracing::debug!(
                        "Toggled sensitivity for field '{}' to: {}",
                        field_name,
                        !sensitive
                    );
                    self.field_sensitivity.insert(field_name, !sensitive);
                }
            }
            CredentialFormMessage::Save => {
                tracing::debug!("Save button clicked in credential form");
                // This is handled by the parent component
            }
            CredentialFormMessage::Cancel => {
                tracing::debug!("Cancel button clicked in credential form");
                // This is handled by the parent component
            }
            CredentialFormMessage::Delete => {
                tracing::debug!("Delete button clicked in credential form");
                // This is handled by the parent component
            }
        }
    }

    /// Render the credential form
    pub fn view(&self) -> Element<CredentialFormMessage> {
        let template = match &self.template {
            Some(t) => t,
            None => return text("No template selected").into(),
        };

        let mut form_fields = vec![
            // Title field (always first and required)
            text("Title *").size(14).into(),
            text_input("Enter credential title...", &self.title)
                .on_input(CredentialFormMessage::TitleChanged)
                .padding(10)
                .into(),
            Space::with_height(Length::Fixed(15.0)).into(),
        ];

        // Add template-specific fields
        for field_template in &template.fields {
            let field_value = self
                .field_values
                .get(&field_template.name)
                .cloned()
                .unwrap_or_default();

            let is_sensitive = self
                .field_sensitivity
                .get(&field_template.name)
                .copied()
                .unwrap_or(field_template.sensitive);

            // Field label with required indicator
            let label = if field_template.required {
                format!("{} *", field_template.label)
            } else {
                field_template.label.clone()
            };

            form_fields.push(text(label).size(14).into());

            // Create appropriate input based on field type
            let input_element = self.create_field_input(
                &field_template.name,
                &field_template.field_type,
                &field_value,
                is_sensitive,
            );

            form_fields.push(input_element);
            form_fields.push(Space::with_height(Length::Fixed(10.0)).into());
        }

        // Add error message if present
        if let Some(error) = &self.config.error_message {
            form_fields.push(Space::with_height(Length::Fixed(10.0)).into());
            form_fields.push(
                text(error)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.94, 0.28, 0.44,
                    ))) // Error red
                    .size(14)
                    .into(),
            );
        }

        // Add action buttons
        form_fields.push(Space::with_height(Length::Fixed(20.0)).into());

        let mut button_row = vec![];

        if self.config.show_cancel_button {
            button_row.push(
                button("Cancel")
                    .on_press(CredentialFormMessage::Cancel)
                    .style(button_styles::secondary())
                    .into(),
            );
            button_row.push(Space::with_width(Length::Fill).into());
        }

        // Add delete button if enabled (left of save button)
        if self.config.show_delete_button {
            if !self.config.show_cancel_button {
                button_row.push(Space::with_width(Length::Fill).into());
            }

            button_row.push(
                button("Delete")
                    .on_press(CredentialFormMessage::Delete)
                    .style(button_styles::destructive())
                    .into(),
            );
            button_row.push(Space::with_width(Length::Fixed(10.0)).into());
        }

        let save_button = if self.config.is_loading {
            button(text(&self.config.save_button_text)).style(button_styles::primary())
        } else {
            button(text(&self.config.save_button_text))
                .on_press(CredentialFormMessage::Save)
                .style(button_styles::primary())
        };

        button_row.push(save_button.into());

        form_fields.push(row(button_row).into());

        scrollable(column(form_fields).spacing(5))
            .height(Length::Fill)
            .into()
    }

    /// Create an input element for a specific field
    fn create_field_input(
        &self,
        field_name: &str,
        field_type: &FieldType,
        value: &str,
        is_sensitive: bool,
    ) -> Element<CredentialFormMessage> {
        let placeholder = match field_type {
            FieldType::Password => "Enter password...",
            FieldType::Email => "Enter email address...",
            FieldType::Url => "https://example.com",
            FieldType::Username => "Enter username...",
            FieldType::Phone => "Enter phone number...",
            FieldType::CreditCardNumber => "Enter card number...",
            FieldType::ExpiryDate => "MM/YY",
            FieldType::Cvv => "CVV",
            FieldType::TotpSecret => "Enter TOTP secret...",
            FieldType::TextArea => "Enter text...",
            FieldType::Number => "Enter number...",
            FieldType::Date => "YYYY-MM-DD",
            _ => "Enter value...",
        };

        match field_type {
            FieldType::TextArea => {
                // Multi-line text input
                text_input(placeholder, value)
                    .on_input({
                        let field_name = field_name.to_string();
                        move |input| CredentialFormMessage::FieldChanged(field_name.clone(), input)
                    })
                    .padding(10)
                    .into()
            }
            FieldType::Password | _ if is_sensitive => {
                // Password input with toggle
                row![
                    text_input(placeholder, value)
                        .on_input({
                            let field_name = field_name.to_string();
                            move |input| {
                                CredentialFormMessage::FieldChanged(field_name.clone(), input)
                            }
                        })
                        .secure(is_sensitive)
                        .padding(10),
                    button(if is_sensitive { "👁" } else { "🙈" })
                        .on_press(CredentialFormMessage::ToggleFieldSensitivity(
                            field_name.to_string()
                        ))
                        .style(button_styles::secondary()),
                ]
                .spacing(5)
                .align_items(Alignment::Center)
                .into()
            }
            _ => {
                // Regular text input
                text_input(placeholder, value)
                    .on_input({
                        let field_name = field_name.to_string();
                        move |input| CredentialFormMessage::FieldChanged(field_name.clone(), input)
                    })
                    .padding(10)
                    .into()
            }
        }
    }

    /// Check if the form has valid data for submission
    pub fn is_valid(&self) -> bool {
        tracing::debug!("Validating credential form...");
        tracing::debug!("Title: '{}'", self.title);
        tracing::debug!("Field values: {:?}", self.field_values);

        // Title is required
        if self.title.trim().is_empty() {
            tracing::warn!("Validation failed: Title is empty");
            return false;
        }

        // Check required fields if template is available
        if let Some(template) = &self.template {
            tracing::debug!("Template: {}", template.name);
            for field_template in &template.fields {
                if field_template.required {
                    let value = self
                        .field_values
                        .get(&field_template.name)
                        .map(|v| v.trim())
                        .unwrap_or("");

                    tracing::debug!(
                        "Checking required field '{}': '{}'",
                        field_template.name,
                        value
                    );

                    if value.is_empty() {
                        tracing::warn!(
                            "Validation failed: Required field '{}' is empty",
                            field_template.name
                        );
                        return false;
                    }
                }
            }
        }

        tracing::debug!("Validation passed!");
        true
    }
}
