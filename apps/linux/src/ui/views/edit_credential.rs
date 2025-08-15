//! Edit Credential View
//!
//! This module provides the view for editing existing credentials.
//! It reuses the credential form component and handles loading existing data.

use iced::{
    widget::{column, container, text, Space},
    Command, Element, Length,
};
use std::collections::HashMap;

use crate::ui::components::{CredentialForm, CredentialFormConfig, CredentialFormMessage};
use crate::ui::theme::container_styles;
use ziplock_shared::models::{CredentialField, CredentialRecord, CredentialTemplate, FieldType};

/// Messages for the edit credential view
#[derive(Debug, Clone)]
pub enum EditCredentialMessage {
    /// Cancel editing and return to main view
    Cancel,
    /// Load the credential data
    LoadCredential,
    /// Credential data was loaded
    CredentialLoaded(Result<CredentialRecord, String>),
    /// Load available credential types
    RefreshTypes,
    /// Credential types were loaded
    TypesLoaded(Result<Vec<CredentialTemplate>, String>),
    /// A form message was received
    FormMessage(CredentialFormMessage),
    /// Update the credential
    UpdateCredential,
    /// Credential was updated
    CredentialUpdated(Result<(), String>),
    /// Delete the credential
    DeleteCredential,
    /// Credential was deleted
    CredentialDeleted(Result<(), String>),

    // Error and success handling (for toast notifications)
    ShowError(String),
    ShowSuccess(String),
    ShowValidationError(String),
}

/// States for the edit credential view
#[derive(Debug, Clone, PartialEq)]
pub enum EditCredentialState {
    /// Loading credential data
    Loading,
    /// Editing the credential
    Editing,
    /// Saving the credential
    Saving,
    /// Update completed successfully
    Complete,
    /// An error occurred
    Error(String),
}

/// The edit credential view
#[derive(Debug)]
pub struct EditCredentialView {
    /// Current state of the view
    state: EditCredentialState,
    /// Available credential templates
    available_types: Vec<CredentialTemplate>,
    /// The credential being edited
    credential: Option<CredentialRecord>,
    /// The credential ID to edit
    credential_id: String,
    /// The credential form component
    form: CredentialForm,
    /// Session ID for backend communication
    session_id: Option<String>,
}

impl EditCredentialView {
    /// Create a new edit credential view for the specified credential ID
    pub fn new(credential_id: String) -> Self {
        let mut form = CredentialForm::new();
        let config = CredentialFormConfig {
            show_delete_button: true,
            save_button_text: "Save".to_string(),
            ..CredentialFormConfig::default()
        };
        form.set_config(config);

        Self {
            state: EditCredentialState::Loading,
            available_types: Self::get_builtin_templates(),
            credential: None,
            credential_id,
            form,
            session_id: None,
        }
    }

    /// Create a new edit credential view with a session ID
    pub fn with_session(credential_id: String, session_id: Option<String>) -> Self {
        let mut view = Self::new(credential_id);
        view.session_id = session_id;
        view
    }

    /// Get built-in credential templates
    fn get_builtin_templates() -> Vec<CredentialTemplate> {
        use ziplock_shared::models::CommonTemplates;
        vec![
            CommonTemplates::login(),
            CommonTemplates::credit_card(),
            CommonTemplates::secure_note(),
        ]
    }

    /// Update the view based on a message
    pub fn update(&mut self, message: EditCredentialMessage) -> Command<EditCredentialMessage> {
        match message {
            EditCredentialMessage::Cancel => {
                // Parent view will handle transition back to main view
                Command::none()
            }

            EditCredentialMessage::LoadCredential => {
                self.state = EditCredentialState::Loading;
                Command::batch([
                    Command::perform(
                        Self::load_credential_async(
                            self.session_id.clone(),
                            self.credential_id.clone(),
                        ),
                        EditCredentialMessage::CredentialLoaded,
                    ),
                    Command::perform(
                        Self::load_credential_types_async(self.session_id.clone()),
                        EditCredentialMessage::TypesLoaded,
                    ),
                ])
            }

            EditCredentialMessage::CredentialLoaded(result) => {
                match result {
                    Ok(credential) => {
                        // Find the matching template for this credential type
                        let template = self
                            .available_types
                            .iter()
                            .find(|t| t.name == credential.credential_type)
                            .cloned()
                            .unwrap_or_else(|| {
                                // Fallback to a basic template if not found
                                Self::get_builtin_templates().into_iter().next().unwrap()
                            });

                        // Set up the form with the credential data
                        self.form.set_template(template);
                        self.form.set_title(credential.title.clone());

                        // Convert credential fields to form field values using field names
                        let mut field_values = HashMap::new();
                        for (field_name, field) in &credential.fields {
                            field_values.insert(field_name.clone(), field.value.clone());
                        }
                        self.form.set_field_values(field_values);

                        // Configure form to show delete button
                        let config = CredentialFormConfig {
                            show_delete_button: true,
                            save_button_text: "Save".to_string(),
                            ..CredentialFormConfig::default()
                        };
                        self.form.set_config(config);

                        self.credential = Some(credential);
                        self.state = EditCredentialState::Editing;
                    }
                    Err(e) => {
                        self.state =
                            EditCredentialState::Error("Failed to load credential".to_string());
                        return Command::perform(
                            async move { e },
                            EditCredentialMessage::ShowError,
                        );
                    }
                }
                Command::none()
            }

            EditCredentialMessage::RefreshTypes => Command::perform(
                Self::load_credential_types_async(self.session_id.clone()),
                EditCredentialMessage::TypesLoaded,
            ),

            EditCredentialMessage::TypesLoaded(result) => {
                match result {
                    Ok(types) => {
                        self.available_types = types;
                    }
                    Err(e) => {
                        // Fallback to built-in templates if loading fails
                        self.available_types = Self::get_builtin_templates();
                        tracing::warn!(
                            "Failed to load credential types, using built-in templates: {}",
                            e
                        );
                    }
                }
                Command::none()
            }

            EditCredentialMessage::FormMessage(form_msg) => {
                match form_msg {
                    CredentialFormMessage::Save => {
                        tracing::debug!("Save button clicked in edit credential view");
                        return Command::perform(async {}, |_| {
                            EditCredentialMessage::UpdateCredential
                        });
                    }
                    CredentialFormMessage::Cancel => {
                        tracing::debug!("Cancel button clicked in edit credential view");
                        return Command::perform(async {}, |_| EditCredentialMessage::Cancel);
                    }
                    CredentialFormMessage::Delete => {
                        tracing::debug!("Delete button clicked in edit credential view");
                        return Command::perform(async {}, |_| {
                            EditCredentialMessage::DeleteCredential
                        });
                    }
                    _ => {
                        self.form.update(form_msg);
                    }
                }
                Command::none()
            }

            EditCredentialMessage::UpdateCredential => {
                tracing::debug!("Processing UpdateCredential message");
                if !self.form.is_valid() {
                    tracing::warn!("Form validation failed in edit credential");
                    return Command::perform(
                        async { "Please fill in all required fields".to_string() },
                        EditCredentialMessage::ShowValidationError,
                    );
                }

                tracing::debug!("Form validation passed, proceeding with credential update");
                self.state = EditCredentialState::Saving;
                let config = CredentialFormConfig {
                    is_loading: true,
                    ..CredentialFormConfig::default()
                };
                self.form.set_config(config);

                Command::perform(
                    Self::update_credential_async(
                        self.session_id.clone(),
                        self.credential_id.clone(),
                        self.form.title().to_string(),
                        self.form.field_values().clone(),
                        self.credential
                            .as_ref()
                            .map(|c| c.credential_type.clone())
                            .unwrap_or_default(),
                    ),
                    EditCredentialMessage::CredentialUpdated,
                )
            }

            EditCredentialMessage::CredentialUpdated(result) => {
                match result {
                    Ok(()) => {
                        tracing::info!("Credential updated successfully");
                        self.state = EditCredentialState::Complete;
                        Command::perform(
                            async { "Credential updated successfully".to_string() },
                            EditCredentialMessage::ShowSuccess,
                        )
                    }
                    Err(e) => {
                        tracing::error!("Failed to update credential: {}", e);
                        self.state =
                            EditCredentialState::Error("Failed to update credential".to_string());
                        // Reset form to not loading state
                        let config = CredentialFormConfig::default();
                        self.form.set_config(config);

                        Command::perform(async move { e }, EditCredentialMessage::ShowError)
                    }
                }
            }

            EditCredentialMessage::DeleteCredential => {
                tracing::debug!("Processing DeleteCredential message");
                self.state = EditCredentialState::Saving;
                let config = CredentialFormConfig {
                    is_loading: true,
                    show_delete_button: true,
                    ..CredentialFormConfig::default()
                };
                self.form.set_config(config);

                Command::perform(
                    Self::delete_credential_async(
                        self.session_id.clone(),
                        self.credential_id.clone(),
                    ),
                    EditCredentialMessage::CredentialDeleted,
                )
            }

            EditCredentialMessage::CredentialDeleted(result) => {
                match result {
                    Ok(()) => {
                        tracing::info!("Credential deleted successfully");
                        self.state = EditCredentialState::Complete;
                        Command::perform(
                            async { "Credential deleted successfully".to_string() },
                            EditCredentialMessage::ShowSuccess,
                        )
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete credential: {}", e);
                        self.state =
                            EditCredentialState::Error("Failed to delete credential".to_string());
                        // Reset form to not loading state
                        let config = CredentialFormConfig {
                            show_delete_button: true,
                            ..CredentialFormConfig::default()
                        };
                        self.form.set_config(config);

                        Command::perform(async move { e }, EditCredentialMessage::ShowError)
                    }
                }
            }

            EditCredentialMessage::ShowError(_) => {
                // Error handling is now done at the application level via toast system
                Command::none()
            }

            EditCredentialMessage::ShowSuccess(_) => {
                // Success handling is now done at the application level via toast system
                Command::none()
            }

            EditCredentialMessage::ShowValidationError(_) => {
                // Validation error handling is now done at the application level via toast system
                Command::none()
            }
        }
    }

    /// Render the edit credential view
    pub fn view(&self) -> Element<'_, EditCredentialMessage> {
        match &self.state {
            EditCredentialState::Loading => self.view_loading(),
            EditCredentialState::Editing => self.view_editing(),
            EditCredentialState::Saving => self.view_saving(),
            EditCredentialState::Complete => self.view_complete(),
            EditCredentialState::Error(_) => self.view_error(),
        }
    }

    /// Render the header for the view
    fn view_header(&self) -> Element<'_, EditCredentialMessage> {
        let title = if let Some(credential) = &self.credential {
            format!("Edit: {}", credential.title)
        } else {
            "Edit Credential".to_string()
        };

        text(title).size(24).into()
    }

    /// Render the loading state
    fn view_loading(&self) -> Element<'_, EditCredentialMessage> {
        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(40.0)),
                text("Loading credential data...").size(16),
            ]
            .spacing(20)
            .align_items(iced::Alignment::Center),
        )
        .padding(40)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(container_styles::sidebar())
        .into()
    }

    /// Render the editing state
    fn view_editing(&self) -> Element<'_, EditCredentialMessage> {
        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(20.0)),
                self.form.view().map(EditCredentialMessage::FormMessage),
            ]
            .spacing(10),
        )
        .padding(40)
        .height(Length::Fill)
        .style(container_styles::sidebar())
        .into()
    }

    /// Render the saving state
    fn view_saving(&self) -> Element<'_, EditCredentialMessage> {
        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(40.0)),
                text("Saving changes...").size(16),
                self.form.view().map(EditCredentialMessage::FormMessage),
            ]
            .spacing(20),
        )
        .padding(40)
        .height(Length::Fill)
        .style(container_styles::sidebar())
        .into()
    }

    /// Render the completion state
    fn view_complete(&self) -> Element<'_, EditCredentialMessage> {
        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(40.0)),
                text("✅ Credential updated successfully!").size(18).style(
                    iced::theme::Text::Color(iced::Color::from_rgb(0.02, 0.84, 0.63))
                ), // Success green
                Space::with_height(Length::Fixed(20.0)),
                text("You will be returned to the main view shortly.").size(14),
            ]
            .spacing(10)
            .align_items(iced::Alignment::Center),
        )
        .padding(40)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(container_styles::sidebar())
        .into()
    }

    /// Render the error state
    fn view_error(&self) -> Element<'_, EditCredentialMessage> {
        let error_message = "Failed to edit credential";

        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(20.0)),
                text("Error updating credential:").size(16),
                text(error_message).size(14).style(iced::theme::Text::Color(
                    iced::Color::from_rgb(0.94, 0.28, 0.44)
                )), // Error red
                Space::with_height(Length::Fixed(20.0)),
                self.form.view().map(EditCredentialMessage::FormMessage),
            ]
            .spacing(10),
        )
        .padding(40)
        .height(Length::Fill)
        .style(container_styles::sidebar())
        .into()
    }

    /// Check if the edit operation is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.state, EditCredentialState::Complete)
    }

    /// Check if the edit operation was cancelled
    pub fn is_cancelled(&self) -> bool {
        // This would be set by the parent view based on the Cancel message
        false
    }

    /// Get subscriptions for TOTP field updates
    pub fn subscription(&self) -> iced::Subscription<EditCredentialMessage> {
        match &self.state {
            EditCredentialState::Editing => self
                .form
                .subscription()
                .map(EditCredentialMessage::FormMessage),
            _ => iced::Subscription::none(),
        }
    }

    /// Load credential types asynchronously
    async fn load_credential_types_async(
        _session_id: Option<String>,
    ) -> Result<Vec<CredentialTemplate>, String> {
        // For now, return built-in templates as the backend API may not be implemented
        Ok(Self::get_builtin_templates())
    }

    /// Load a specific credential asynchronously
    async fn load_credential_async(
        session_id: Option<String>,
        credential_id: String,
    ) -> Result<CredentialRecord, String> {
        let mut client = ziplock_shared::ZipLockClient::new().map_err(|e| e.to_string())?;
        client.connect().await.map_err(|e| e.to_string())?;
        client
            .get_credential(session_id, credential_id)
            .await
            .map_err(|e| e.to_string())
    }

    /// Update a credential asynchronously
    /// Delete a credential asynchronously
    async fn delete_credential_async(
        session_id: Option<String>,
        credential_id: String,
    ) -> Result<(), String> {
        tracing::debug!("Starting credential deletion for ID: {}", credential_id);

        let mut client = ziplock_shared::ZipLockClient::new().map_err(|e| e.to_string())?;
        client.connect().await.map_err(|e| e.to_string())?;

        tracing::debug!(
            "Calling FFI client to delete credential with ID: {}",
            credential_id
        );

        client
            .delete_credential(session_id, credential_id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn update_credential_async(
        session_id: Option<String>,
        id: String,
        title: String,
        field_values: HashMap<String, String>,
        credential_type: String,
    ) -> Result<(), String> {
        let mut client = ziplock_shared::ZipLockClient::new().map_err(|e| e.to_string())?;
        client.connect().await.map_err(|e| e.to_string())?;

        // Get the template to properly map field types and sensitivity
        let template = match credential_type.as_str() {
            "login" => Some(ziplock_shared::models::CommonTemplates::login()),
            "credit_card" => Some(ziplock_shared::models::CommonTemplates::credit_card()),
            "secure_note" => Some(ziplock_shared::models::CommonTemplates::secure_note()),
            _ => None,
        };

        // Convert field values back to credential fields
        // Use template information to set correct field types and sensitivity
        let fields: HashMap<String, CredentialField> = field_values
            .into_iter()
            .map(|(field_name, value)| {
                // Find the template field to get the correct type and sensitivity
                let (field_type, sensitive, label) = if let Some(ref template) = template {
                    if let Some(field_template) =
                        template.fields.iter().find(|f| f.name == field_name)
                    {
                        (
                            field_template.field_type.clone(),
                            field_template.sensitive,
                            field_template.label.clone(),
                        )
                    } else {
                        (FieldType::Text, false, field_name.clone())
                    }
                } else {
                    (FieldType::Text, false, field_name.clone())
                };

                let field = CredentialField {
                    field_type,
                    value,
                    sensitive,
                    label: Some(label),
                    metadata: HashMap::new(),
                };
                (field_name, field)
            })
            .collect();

        tracing::debug!("Calling IPC client to update credential with ID: {}", id);
        tracing::debug!("Title: {}", title);
        tracing::debug!("Credential type: {}", credential_type);
        tracing::debug!("Fields: {:?}", fields);

        // Create CredentialRecord from the components
        let mut credential = CredentialRecord::new(title, credential_type);
        credential.id = id;
        credential.fields = fields;
        credential.tags = Vec::new(); // tags
        credential.notes = None; // notes

        client
            .update_credential(session_id, credential)
            .await
            .map_err(|e| e.to_string())
    }
}
