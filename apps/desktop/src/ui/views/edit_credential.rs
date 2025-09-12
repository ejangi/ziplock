//! Edit Credential View
//!
//! This module provides the view for editing existing credentials.
//! It reuses the credential form component and handles loading existing data.

use crate::services::get_repository_service;
use iced::{
    widget::{column, container, text, Space},
    Element, Length, Task,
};
use std::collections::HashMap;

use crate::ui::components::{CredentialForm, CredentialFormConfig, CredentialFormMessage};

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

    // Clipboard operations
    CopyToClipboard {
        content: String,
        content_type: crate::services::ClipboardContentType,
    },

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
            CommonTemplates::identity(),
            CommonTemplates::password(),
            CommonTemplates::document(),
            CommonTemplates::ssh_key(),
            CommonTemplates::bank_account(),
            CommonTemplates::api_credentials(),
            CommonTemplates::crypto_wallet(),
            CommonTemplates::database(),
            CommonTemplates::software_license(),
        ]
    }

    /// Update the view based on a message
    pub fn update(&mut self, message: EditCredentialMessage) -> Task<EditCredentialMessage> {
        match message {
            EditCredentialMessage::Cancel => {
                // Parent view will handle transition back to main view
                Task::none()
            }

            EditCredentialMessage::LoadCredential => {
                self.state = EditCredentialState::Loading;
                Task::batch([
                    Task::perform(
                        Self::load_credential_async(
                            self.session_id.clone(),
                            self.credential_id.clone(),
                        ),
                        EditCredentialMessage::CredentialLoaded,
                    ),
                    Task::perform(
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

                        tracing::info!("Setting up edit form for credential: {}", credential.title);
                        tracing::info!("Template found: {}", template.name);
                        tracing::info!("Credential has {} fields", credential.fields.len());

                        // Log all credential fields
                        for (field_name, field) in &credential.fields {
                            tracing::info!(
                                "Field '{}': type={:?}, has_value={}",
                                field_name,
                                field.field_type,
                                !field.value.is_empty()
                            );
                        }

                        // Set up the form with the credential data
                        tracing::info!("Setting template on form...");
                        self.form.set_template(template);
                        tracing::info!("Setting title on form...");
                        self.form.set_title(credential.title.clone());

                        // Convert credential fields to form field values using field names
                        let mut field_values = HashMap::new();
                        for (field_name, field) in &credential.fields {
                            field_values.insert(field_name.clone(), field.value.clone());
                        }
                        tracing::info!("Setting {} field values on form...", field_values.len());
                        self.form.set_field_values(field_values);

                        // Configure form to show delete button with title styling
                        let config = CredentialFormConfig {
                            show_delete_button: true,
                            save_button_text: "Save".to_string(),
                            use_title_styling: true,
                            credential_type: Some(credential.credential_type.clone()),
                            ..CredentialFormConfig::default()
                        };
                        self.form.set_config(config);

                        self.credential = Some(credential);
                        self.state = EditCredentialState::Editing;
                        tracing::info!("Edit credential form setup completed successfully");
                    }
                    Err(e) => {
                        self.state =
                            EditCredentialState::Error("Failed to load credential".to_string());
                        return Task::perform(async move { e }, EditCredentialMessage::ShowError);
                    }
                }
                Task::none()
            }

            EditCredentialMessage::RefreshTypes => Task::perform(
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
                Task::none()
            }

            EditCredentialMessage::FormMessage(form_msg) => {
                match form_msg {
                    CredentialFormMessage::Save => {
                        tracing::debug!("Save button clicked in edit credential view");
                        Task::perform(async {}, |_| EditCredentialMessage::UpdateCredential)
                    }
                    CredentialFormMessage::Cancel => {
                        tracing::debug!("Cancel button clicked in edit credential view");
                        Task::perform(async {}, |_| EditCredentialMessage::Cancel)
                    }
                    CredentialFormMessage::Delete => {
                        tracing::debug!("Delete button clicked in edit credential view");
                        Task::perform(async {}, |_| EditCredentialMessage::DeleteCredential)
                    }
                    CredentialFormMessage::CopyFieldToClipboard {
                        field_name: _,
                        content,
                        content_type,
                    } => {
                        tracing::debug!(
                            "EditCredential view forwarding field clipboard message: content_type={:?}, content_length={}",
                            content_type,
                            content.len()
                        );
                        // Forward clipboard operations to main app
                        Task::perform(
                            async move { (content, content_type) },
                            |(content, content_type)| EditCredentialMessage::CopyToClipboard {
                                content,
                                content_type,
                            },
                        )
                    }
                    CredentialFormMessage::CopyToClipboard {
                        content,
                        content_type,
                    } => {
                        tracing::debug!(
                            "EditCredential view forwarding clipboard message: content_type={:?}, content_length={}",
                            content_type,
                            content.len()
                        );
                        // Forward clipboard operations to main app
                        Task::perform(
                            async move { (content, content_type) },
                            |(content, content_type)| EditCredentialMessage::CopyToClipboard {
                                content,
                                content_type,
                            },
                        )
                    }
                    _ => {
                        let form_command = self.form.update(form_msg);
                        // Map form commands to edit credential commands
                        form_command.map(EditCredentialMessage::FormMessage)
                    }
                }
            }

            EditCredentialMessage::UpdateCredential => {
                tracing::debug!("Processing UpdateCredential message");
                if !self.form.is_valid() {
                    tracing::warn!("Form validation failed in edit credential");
                    return Task::perform(
                        async { "Please fill in all required fields".to_string() },
                        EditCredentialMessage::ShowValidationError,
                    );
                }

                tracing::debug!("Form validation passed, proceeding with credential update");
                self.state = EditCredentialState::Saving;
                let config = CredentialFormConfig {
                    is_loading: true,
                    use_title_styling: true,
                    credential_type: self.credential.as_ref().map(|c| c.credential_type.clone()),
                    ..CredentialFormConfig::default()
                };
                self.form.set_config(config);

                Task::perform(
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
                        Task::perform(
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

                        Task::perform(async move { e }, EditCredentialMessage::ShowError)
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

                Task::perform(
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
                        Task::perform(
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

                        Task::perform(async move { e }, EditCredentialMessage::ShowError)
                    }
                }
            }

            EditCredentialMessage::ShowError(_) => {
                // Error handling is now done at the application level via toast system
                Task::none()
            }

            EditCredentialMessage::ShowSuccess(_) => {
                // Success handling is now done at the application level via toast system
                Task::none()
            }

            EditCredentialMessage::ShowValidationError(_) => {
                // Validation error handling is now done at the application level via toast system
                Task::none()
            }
            EditCredentialMessage::CopyToClipboard { .. } => {
                // This should be handled by the parent component (main app)
                Task::none()
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
            EditCredentialState::Error(ref error_msg) => self.view_error(error_msg),
        }
    }

    /// Render the loading state
    fn view_loading(&self) -> Element<'_, EditCredentialMessage> {
        container(
            column![
                Space::with_height(Length::Fixed(40.0)),
                text("Loading credential data...")
                    .size(crate::ui::theme::utils::typography::medium_text_size()),
            ]
            .spacing(20)
            .align_x(iced::Alignment::Center),
        )
        .padding(40)
        .into()
    }

    /// Render the editing state
    fn view_editing(&self) -> Element<'_, EditCredentialMessage> {
        container(
            column![
                Space::with_height(Length::Fixed(20.0)),
                self.form.view().map(EditCredentialMessage::FormMessage),
            ]
            .spacing(10),
        )
        .padding(40)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render the saving state
    fn view_saving(&self) -> Element<'_, EditCredentialMessage> {
        container(
            column![
                Space::with_height(Length::Fixed(40.0)),
                text("Saving changes...")
                    .size(crate::ui::theme::utils::typography::medium_text_size()),
                self.form.view().map(EditCredentialMessage::FormMessage),
            ]
            .spacing(20),
        )
        .padding(40)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render the completion state
    fn view_complete(&self) -> Element<'_, EditCredentialMessage> {
        container(
            column![
                Space::with_height(Length::Fixed(40.0)),
                text("✅ Credential updated successfully!")
                    .size(crate::ui::theme::utils::typography::large_text_size()),
                Space::with_height(Length::Fixed(20.0)),
                text("You will be returned to the main view shortly.")
                    .size(crate::ui::theme::utils::typography::normal_text_size()),
            ]
            .spacing(20)
            .align_x(iced::Alignment::Center),
        )
        .padding(40)
        .into()
    }

    /// Render the error state
    fn view_error<'a>(&'a self, error_message: &'a str) -> Element<'a, EditCredentialMessage> {
        container(
            column![
                Space::with_height(Length::Fixed(20.0)),
                text("Error updating credential:")
                    .size(crate::ui::theme::utils::typography::medium_text_size()),
                text(error_message).size(crate::ui::theme::utils::typography::normal_text_size()),
                Space::with_height(Length::Fixed(20.0)),
                self.form.view().map(EditCredentialMessage::FormMessage),
            ]
            .spacing(10),
        )
        .padding(40)
        .width(Length::Fill)
        .height(Length::Fill)
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
        _session_id: Option<String>,
        credential_id: String,
    ) -> Result<CredentialRecord, String> {
        // Use hybrid client for unified architecture
        let repo_service = get_repository_service();

        repo_service
            .get_credential(credential_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Credential not found".to_string())
    }

    /// Update a credential asynchronously
    /// Delete a credential asynchronously
    async fn delete_credential_async(
        _session_id: Option<String>,
        credential_id: String,
    ) -> Result<(), String> {
        tracing::debug!("Starting credential deletion for ID: {}", credential_id);

        // Use hybrid client for unified architecture
        let repo_service = get_repository_service();

        tracing::debug!(
            "Calling hybrid client to delete credential with ID: {}",
            credential_id
        );

        repo_service
            .delete_credential(credential_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn update_credential_async(
        _session_id: Option<String>,
        id: String,
        title: String,
        field_values: HashMap<String, String>,
        credential_type: String,
    ) -> Result<(), String> {
        // Use hybrid client for unified architecture
        let repo_service = get_repository_service();

        // Get the template to properly map field types and sensitivity
        let template = match credential_type.as_str() {
            "login" => Some(ziplock_shared::models::CommonTemplates::login()),
            "credit_card" => Some(ziplock_shared::models::CommonTemplates::credit_card()),
            "secure_note" => Some(ziplock_shared::models::CommonTemplates::secure_note()),
            "identity" => Some(ziplock_shared::models::CommonTemplates::identity()),
            "password" => Some(ziplock_shared::models::CommonTemplates::password()),
            "document" => Some(ziplock_shared::models::CommonTemplates::document()),
            "ssh_key" => Some(ziplock_shared::models::CommonTemplates::ssh_key()),
            "bank_account" => Some(ziplock_shared::models::CommonTemplates::bank_account()),
            "api_credentials" => Some(ziplock_shared::models::CommonTemplates::api_credentials()),
            "crypto_wallet" => Some(ziplock_shared::models::CommonTemplates::crypto_wallet()),
            "database" => Some(ziplock_shared::models::CommonTemplates::database()),
            "software_license" => Some(ziplock_shared::models::CommonTemplates::software_license()),
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

        tracing::debug!("Updating credential with repository service, ID: {}", id);
        tracing::debug!("Title: {}", title);
        tracing::debug!("Credential type: {}", credential_type);
        tracing::debug!("Fields: {:?}", fields);

        // Create CredentialRecord from the components
        let mut credential = CredentialRecord::new(title, credential_type);
        credential.id = id;
        credential.fields = fields;
        credential.tags = Vec::new(); // tags
        credential.notes = None; // notes

        repo_service
            .update_credential(credential)
            .await
            .map_err(|e| e.to_string())
    }
}
