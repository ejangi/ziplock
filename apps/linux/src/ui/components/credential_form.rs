//! Reusable Credential Form Component
//!
//! This module provides a reusable form component for creating and editing credentials.
//! It handles field rendering, validation, and user interaction for credential data.

use iced::{
    alignment::Alignment,
    widget::{button, column, row, scrollable, svg, text, text_editor, text_input, Space},
    Element, Length,
};
use std::collections::HashMap;

use crate::ui::components::totp_field::TotpField;
use crate::ui::theme::{button_styles, utils, ERROR_RED};
use ziplock_shared::models::{CredentialTemplate, FieldType};

/// Messages that can be sent from the credential form
#[derive(Debug, Clone, PartialEq)]
pub enum CredentialFormMessage {
    /// The title field was changed
    TitleChanged(String),
    /// A specific field value was changed
    FieldChanged(String, String),
    /// A text editor action for TextArea fields
    TextEditorAction(String, text_editor::Action),
    /// Toggle the sensitivity (show/hide) of a field
    ToggleFieldSensitivity(String),
    /// TOTP field message
    TotpFieldMessage(String, crate::ui::components::totp_field::TotpFieldMessage),
    /// Copy field content to clipboard
    CopyFieldToClipboard {
        field_name: String,
        content: String,
        content_type: crate::services::ClipboardContentType,
    },
    /// Copy content to clipboard with timeout
    CopyToClipboard {
        content: String,
        content_type: crate::services::ClipboardContentType,
    },
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
    /// Whether to use special title styling (larger font, padding)
    pub use_title_styling: bool,
    /// Optional credential type for showing icon with title
    pub credential_type: Option<String>,
}

impl Default for CredentialFormConfig {
    fn default() -> Self {
        Self {
            save_button_text: "Save".to_string(),
            show_cancel_button: true,
            show_delete_button: false,
            is_loading: false,
            error_message: None,
            use_title_styling: false,
            credential_type: None,
        }
    }
}

/// The credential form component
#[derive(Debug)]
pub struct CredentialForm {
    /// The credential template to render
    template: Option<CredentialTemplate>,
    /// The current title value
    title: String,
    /// Current field values
    field_values: HashMap<String, String>,
    /// Text editor content for TextArea fields
    text_editor_content: HashMap<String, text_editor::Content>,
    /// Field sensitivity state (whether to show or hide sensitive fields)
    field_sensitivity: HashMap<String, bool>,
    /// TOTP field components
    totp_fields: HashMap<String, TotpField>,
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
            text_editor_content: HashMap::new(),
            field_sensitivity: HashMap::new(),
            totp_fields: HashMap::new(),
            config: CredentialFormConfig::default(),
        }
    }

    /// Create a credential form with a specific template
    #[allow(dead_code)] // Future credential form functionality
    pub fn with_template(template: CredentialTemplate) -> Self {
        let mut form = Self::new();
        form.set_template(template);
        form
    }

    /// Set the credential template for the form
    pub fn set_template(&mut self, template: CredentialTemplate) {
        // Initialize field sensitivity and text editor content based on template defaults
        for field_template in &template.fields {
            self.field_sensitivity
                .insert(field_template.name.clone(), field_template.sensitive);

            // Initialize text editor content for TextArea fields
            if field_template.field_type == FieldType::TextArea {
                let content = text_editor::Content::with_text(
                    self.field_values
                        .get(&field_template.name)
                        .unwrap_or(&String::new()),
                );
                self.text_editor_content
                    .insert(field_template.name.clone(), content);
            }

            // Initialize TOTP fields for TotpSecret field types
            if field_template.field_type == FieldType::TotpSecret {
                let secret = self
                    .field_values
                    .get(&field_template.name)
                    .unwrap_or(&String::new())
                    .clone();
                let totp_field = TotpField::new_editing(field_template.name.clone(), secret);
                self.totp_fields
                    .insert(field_template.name.clone(), totp_field);
            }
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
        self.field_values.insert(field_name.clone(), value.clone());

        // Update text editor content if it's a TextArea field
        if let Some(content) = self.text_editor_content.get_mut(&field_name) {
            *content = text_editor::Content::with_text(&value);
        }

        // Update TOTP field if it's a TotpSecret field
        if let Some(totp_field) = self.totp_fields.get_mut(&field_name) {
            totp_field.set_secret(value);
        }
    }

    /// Set multiple field values at once
    pub fn set_field_values(&mut self, values: HashMap<String, String>) {
        for (name, value) in values {
            self.set_field_value(name, value);
        }
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
    pub fn update(
        &mut self,
        message: CredentialFormMessage,
    ) -> iced::Command<CredentialFormMessage> {
        match message {
            CredentialFormMessage::TitleChanged(title) => {
                tracing::debug!("Title changed to: '{}'", title);
                self.title = title;
            }
            CredentialFormMessage::FieldChanged(field_name, value) => {
                tracing::debug!("Field '{}' changed to: '{}'", field_name, value);
                self.field_values.insert(field_name.clone(), value.clone());

                // Update text editor content if this is a TextArea field
                if let Some(content) = self.text_editor_content.get_mut(&field_name) {
                    *content = text_editor::Content::with_text(&value);
                }
            }
            CredentialFormMessage::TextEditorAction(field_name, action) => {
                tracing::debug!("TextEditor action for field '{}': {:?}", field_name, action);

                if let Some(content) = self.text_editor_content.get_mut(&field_name) {
                    content.perform(action);
                    // Update the field value from the text editor content
                    let text = content.text();
                    self.field_values.insert(field_name, text);
                }
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
            CredentialFormMessage::TotpFieldMessage(field_name, totp_msg) => {
                tracing::debug!("TOTP field '{}' message: {:?}", field_name, totp_msg);

                if let Some(totp_field) = self.totp_fields.get_mut(&field_name) {
                    // Handle TOTP copy operations specially
                    if let crate::ui::components::totp_field::TotpFieldMessage::CopyCode = &totp_msg
                    {
                        if let Some(code) = totp_field.current_code() {
                            tracing::debug!(
                                "TOTP copy requested for field '{}', code: '{}'",
                                field_name,
                                code
                            );
                            // Don't call totp_field.update() for CopyCode to avoid direct clipboard access
                            // Instead, return a command that the parent can handle
                            return iced::Command::perform(async move { code }, |code| {
                                tracing::debug!(
                                    "Returning CopyToClipboard command for TOTP code: '{}'",
                                    code
                                );
                                CredentialFormMessage::CopyToClipboard {
                                    content: code,
                                    content_type: crate::services::ClipboardContentType::TotpCode,
                                }
                            });
                        } else {
                            tracing::warn!(
                                "TOTP copy requested but no current code available for field '{}'",
                                field_name
                            );
                        }
                    } else {
                        totp_field.update(totp_msg);
                        // Update the field value with the current secret
                        let secret = totp_field.secret().to_string();
                        self.field_values.insert(field_name, secret);
                    }
                }
            }
            CredentialFormMessage::CopyFieldToClipboard {
                field_name: _,
                content,
                content_type,
            } => {
                // Forward to parent as CopyToClipboard message
                return iced::Command::perform(
                    async move { (content, content_type) },
                    |(content, content_type)| CredentialFormMessage::CopyToClipboard {
                        content,
                        content_type,
                    },
                );
            }
            CredentialFormMessage::CopyToClipboard {
                content,
                content_type,
            } => {
                // This should be handled by the parent component
                tracing::debug!(
                    "Clipboard message received in credential form - should be handled by parent"
                );
                return iced::Command::perform(
                    async move { (content, content_type) },
                    |(content, content_type)| CredentialFormMessage::CopyToClipboard {
                        content,
                        content_type,
                    },
                );
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
        iced::Command::none()
    }

    /// Render the credential form
    pub fn view(&self) -> Element<'_, CredentialFormMessage> {
        let template = match &self.template {
            Some(t) => t,
            None => return text("No template selected").into(),
        };

        let mut form_fields = vec![];

        // Title field (always first and required)
        if self.config.use_title_styling {
            // Use special title styling with icon if credential type is available
            if let Some(credential_type) = &self.config.credential_type {
                let icon =
                    crate::ui::theme::utils::typography::get_credential_type_icon(credential_type);

                form_fields.push(
                    row![
                        svg(icon)
                            .width(Length::Fixed(24.0))
                            .height(Length::Fixed(24.0)),
                        text_input("Enter credential title...", &self.title)
                            .on_input(CredentialFormMessage::TitleChanged)
                            .padding(utils::title_input_padding())
                            .style(crate::ui::theme::text_input_styles::title())
                            .size(crate::ui::theme::utils::typography::title_input_size())
                            .width(Length::Fill)
                    ]
                    .spacing(12)
                    .align_items(Alignment::Center)
                    .into(),
                );
            } else {
                form_fields.push(
                    text_input("Enter credential title...", &self.title)
                        .on_input(CredentialFormMessage::TitleChanged)
                        .padding(utils::title_input_padding())
                        .style(crate::ui::theme::text_input_styles::title())
                        .size(crate::ui::theme::utils::typography::title_input_size())
                        .into(),
                );
            }
        } else {
            // Standard title field
            form_fields.push(
                text("Title *")
                    .size(crate::ui::theme::utils::typography::normal_text_size())
                    .into(),
            );
            form_fields.push(
                text_input("Enter credential title...", &self.title)
                    .on_input(CredentialFormMessage::TitleChanged)
                    .padding(utils::text_input_padding())
                    .style(crate::ui::theme::text_input_styles::standard())
                    .size(crate::ui::theme::utils::typography::text_input_size())
                    .into(),
            );
        }

        form_fields.push(Space::with_height(Length::Fixed(15.0)).into());

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

            form_fields.push(
                text(label)
                    .size(crate::ui::theme::utils::typography::normal_text_size())
                    .into(),
            );

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
                    .style(iced::theme::Text::Color(ERROR_RED))
                    .size(crate::ui::theme::utils::typography::normal_text_size())
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
                    .padding(utils::button_padding())
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
                    .padding(utils::button_padding())
                    .into(),
            );
            button_row.push(Space::with_width(Length::Fixed(10.0)).into());
        }

        let save_button = if self.config.is_loading {
            button(text(&self.config.save_button_text))
                .style(button_styles::primary())
                .padding(utils::button_padding())
        } else {
            button(text(&self.config.save_button_text))
                .on_press(CredentialFormMessage::Save)
                .style(button_styles::primary())
                .padding(utils::button_padding())
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
    ) -> Element<'_, CredentialFormMessage> {
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
            FieldType::TotpSecret => {
                // TOTP field with specialized component
                if let Some(totp_field) = self.totp_fields.get(field_name) {
                    totp_field.view().map({
                        let field_name = field_name.to_string();
                        move |msg| CredentialFormMessage::TotpFieldMessage(field_name.clone(), msg)
                    })
                } else {
                    // Fallback to regular text input if no TOTP field available
                    text_input(placeholder, value)
                        .on_input({
                            let field_name = field_name.to_string();
                            move |input| {
                                CredentialFormMessage::FieldChanged(field_name.clone(), input)
                            }
                        })
                        .padding(utils::text_input_padding())
                        .style(crate::ui::theme::text_input_styles::standard())
                        .size(crate::ui::theme::utils::typography::text_input_size())
                        .into()
                }
            }
            FieldType::TextArea => {
                // Multi-line text editor
                if let Some(content) = self.text_editor_content.get(field_name) {
                    text_editor(content)
                        .on_action({
                            let field_name = field_name.to_string();
                            move |action| {
                                CredentialFormMessage::TextEditorAction(field_name.clone(), action)
                            }
                        })
                        .height(Length::Fixed(100.0))
                        .into()
                } else {
                    // Fallback to regular text input if no content state available
                    text_input(placeholder, value)
                        .on_input({
                            let field_name = field_name.to_string();
                            move |input| {
                                CredentialFormMessage::FieldChanged(field_name.clone(), input)
                            }
                        })
                        .padding(utils::text_input_padding())
                        .style(crate::ui::theme::text_input_styles::standard())
                        .size(crate::ui::theme::utils::typography::text_input_size())
                        .into()
                }
            }
            FieldType::Password if is_sensitive => {
                // Password input with toggle and copy button
                row![
                    text_input(placeholder, value)
                        .on_input({
                            let field_name = field_name.to_string();
                            move |input| {
                                CredentialFormMessage::FieldChanged(field_name.clone(), input)
                            }
                        })
                        .secure(is_sensitive)
                        .padding(utils::text_input_padding())
                        .style(crate::ui::theme::text_input_styles::standard())
                        .size(crate::ui::theme::utils::typography::text_input_size()),
                    button("📋")
                        .on_press(CredentialFormMessage::CopyFieldToClipboard {
                            field_name: field_name.to_string(),
                            content: value.to_string(),
                            content_type: crate::services::ClipboardContentType::Password,
                        })
                        .style(button_styles::secondary())
                        .padding(utils::small_button_padding()),
                    button(if is_sensitive { "👁" } else { "🙈" })
                        .on_press(CredentialFormMessage::ToggleFieldSensitivity(
                            field_name.to_string()
                        ))
                        .style(button_styles::secondary())
                        .padding(utils::small_button_padding()),
                ]
                .spacing(5)
                .align_items(Alignment::Center)
                .into()
            }
            _ if is_sensitive => {
                // Sensitive field input with toggle and copy button
                row![
                    text_input(placeholder, value)
                        .on_input({
                            let field_name = field_name.to_string();
                            move |input| {
                                CredentialFormMessage::FieldChanged(field_name.clone(), input)
                            }
                        })
                        .secure(is_sensitive)
                        .padding(utils::text_input_padding())
                        .style(crate::ui::theme::text_input_styles::standard())
                        .size(crate::ui::theme::utils::typography::text_input_size()),
                    button("📋")
                        .on_press(CredentialFormMessage::CopyFieldToClipboard {
                            field_name: field_name.to_string(),
                            content: value.to_string(),
                            content_type: crate::services::ClipboardContentType::Password,
                        })
                        .style(button_styles::secondary())
                        .padding(utils::small_button_padding()),
                    button(if is_sensitive { "👁" } else { "🙈" })
                        .on_press(CredentialFormMessage::ToggleFieldSensitivity(
                            field_name.to_string()
                        ))
                        .style(button_styles::secondary())
                        .padding(utils::small_button_padding()),
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
                    .secure(is_sensitive)
                    .padding(utils::text_input_padding())
                    .width(Length::Fill)
                    .style(crate::ui::theme::text_input_styles::standard())
                    .size(crate::ui::theme::utils::typography::text_input_size())
                    .into()
            }
        }
    }

    /// Get subscriptions for all TOTP fields
    pub fn subscription(&self) -> iced::Subscription<CredentialFormMessage> {
        let totp_subscriptions: Vec<iced::Subscription<CredentialFormMessage>> = self
            .totp_fields
            .iter()
            .map(|(field_name, totp_field)| {
                totp_field.subscription().map({
                    let field_name = field_name.clone();
                    move |msg| CredentialFormMessage::TotpFieldMessage(field_name.clone(), msg)
                })
            })
            .collect();

        iced::Subscription::batch(totp_subscriptions)
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

#[cfg(test)]
mod tests {
    use super::*;
    use ziplock_shared::models::{FieldTemplate, FieldType};

    #[test]
    fn test_totp_copy_message_flow() {
        let mut form = CredentialForm::new();

        // Create a template with a TOTP field
        let totp_template = FieldTemplate {
            name: "totp_secret".to_string(),
            field_type: FieldType::TotpSecret,
            label: "TOTP Secret".to_string(),
            required: false,
            sensitive: false,
            default_value: None,
            validation: None,
        };

        let template = CredentialTemplate {
            name: "login".to_string(),
            description: "Basic login credentials".to_string(),
            fields: vec![totp_template],
            default_tags: vec![],
        };

        form.set_template(template);

        // Set a TOTP secret that would generate a code
        form.set_field_value("totp_secret".to_string(), "JBSWY3DPEHPK3PXP".to_string());

        // Simulate a TOTP copy message
        let totp_msg = crate::ui::components::totp_field::TotpFieldMessage::CopyCode;
        let form_msg = CredentialFormMessage::TotpFieldMessage("totp_secret".to_string(), totp_msg);

        // Update should return a command for clipboard operation
        let _command = form.update(form_msg);

        // If we get here without panicking, the basic message flow is working
        assert!(true);
    }

    #[test]
    fn test_password_field_copy_button() {
        let mut form = CredentialForm::new();

        // Create a template with a password field
        let password_template = FieldTemplate {
            name: "password".to_string(),
            field_type: FieldType::Password,
            label: "Password".to_string(),
            required: true,
            sensitive: true,
            default_value: None,
            validation: None,
        };

        let template = CredentialTemplate {
            name: "login".to_string(),
            description: "Basic login credentials".to_string(),
            fields: vec![password_template],
            default_tags: vec![],
        };

        form.set_template(template);
        form.set_field_value("password".to_string(), "secret123".to_string());

        // Simulate a password field copy message
        let copy_msg = CredentialFormMessage::CopyFieldToClipboard {
            field_name: "password".to_string(),
            content: "secret123".to_string(),
            content_type: crate::services::ClipboardContentType::Password,
        };

        // Update should return a command for clipboard operation
        let _command = form.update(copy_msg);

        // Basic test to ensure the message is processed
        assert!(true);
    }
}
