//! Edit Credential View
//!
//! This module provides the view for editing existing credentials.
//! It reuses the credential form component and handles loading existing data.

use iced::{
    widget::{column, container, text, Space},
    Command, Element, Length,
};
use std::collections::HashMap;

use crate::ipc::IpcClient;
use crate::ui::components::{CredentialForm, CredentialFormConfig, CredentialFormMessage};
use crate::ui::theme::{alerts::AlertMessage, container_styles};
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
#[derive(Debug, Clone)]
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
    /// Current error message
    current_error: Option<AlertMessage>,
    /// Session ID for backend communication
    session_id: Option<String>,
}

impl EditCredentialView {
    /// Create a new edit credential view for the specified credential ID
    pub fn new(credential_id: String) -> Self {
        Self {
            state: EditCredentialState::Loading,
            available_types: Vec::new(),
            credential: None,
            credential_id,
            form: CredentialForm::new(),
            current_error: None,
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
                self.current_error = None;
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

                        // Convert credential fields to form field values
                        let mut field_values = HashMap::new();
                        for (field_name, field) in &credential.fields {
                            if let Some(label) = &field.label {
                                field_values.insert(label.clone(), field.value.clone());
                            } else {
                                field_values.insert(field_name.clone(), field.value.clone());
                            }
                        }
                        self.form.set_field_values(field_values);

                        self.credential = Some(credential);
                        self.state = EditCredentialState::Editing;
                        self.current_error = None;
                    }
                    Err(e) => {
                        self.current_error = Some(AlertMessage::error(e));
                        self.state =
                            EditCredentialState::Error("Failed to load credential".to_string());
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
                        return Command::perform(async {}, |_| {
                            EditCredentialMessage::UpdateCredential
                        });
                    }
                    CredentialFormMessage::Cancel => {
                        return Command::perform(async {}, |_| EditCredentialMessage::Cancel);
                    }
                    _ => {
                        self.form.update(form_msg);
                    }
                }
                Command::none()
            }

            EditCredentialMessage::UpdateCredential => {
                if !self.form.is_valid() {
                    self.current_error = Some(AlertMessage::warning(
                        "Please fill in all required fields".to_string(),
                    ));
                    return Command::none();
                }

                self.state = EditCredentialState::Saving;
                let mut config = CredentialFormConfig::default();
                config.is_loading = true;
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
                        self.state = EditCredentialState::Complete;
                        self.current_error = None;
                    }
                    Err(e) => {
                        self.current_error = Some(AlertMessage::ipc_error(e));
                        self.state =
                            EditCredentialState::Error("Failed to update credential".to_string());
                        // Reset form to not loading state
                        let mut config = CredentialFormConfig::default();
                        config.error_message =
                            self.current_error.as_ref().map(|a| a.message.clone());
                        self.form.set_config(config);
                    }
                }
                Command::none()
            }
        }
    }

    /// Render the edit credential view
    pub fn view(&self) -> Element<EditCredentialMessage> {
        match &self.state {
            EditCredentialState::Loading => self.view_loading(),
            EditCredentialState::Editing => self.view_editing(),
            EditCredentialState::Saving => self.view_saving(),
            EditCredentialState::Complete => self.view_complete(),
            EditCredentialState::Error(_) => self.view_error(),
        }
    }

    /// Render the header for the view
    fn view_header(&self) -> Element<EditCredentialMessage> {
        let title = if let Some(credential) = &self.credential {
            format!("Edit: {}", credential.title)
        } else {
            "Edit Credential".to_string()
        };

        text(title).size(24).into()
    }

    /// Render the loading state
    fn view_loading(&self) -> Element<EditCredentialMessage> {
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
    fn view_editing(&self) -> Element<EditCredentialMessage> {
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
    fn view_saving(&self) -> Element<EditCredentialMessage> {
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
    fn view_complete(&self) -> Element<EditCredentialMessage> {
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
    fn view_error(&self) -> Element<EditCredentialMessage> {
        let error_message = self
            .current_error
            .as_ref()
            .map(|e| e.message.as_str())
            .unwrap_or("An unknown error occurred");

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
        let mut client = IpcClient::new().map_err(|e| e.to_string())?;
        client.get_credential(session_id, credential_id).await
    }

    /// Update a credential asynchronously
    async fn update_credential_async(
        session_id: Option<String>,
        id: String,
        title: String,
        field_values: HashMap<String, String>,
        credential_type: String,
    ) -> Result<(), String> {
        let mut client = IpcClient::new().map_err(|e| e.to_string())?;

        // Convert field values back to credential fields
        let fields: HashMap<String, CredentialField> = field_values
            .into_iter()
            .map(|(label, value)| {
                // For this mock, we'll use default FieldType::Text and sensitive: false
                // In a real scenario, this would involve retrieving the original field
                // templates or types.
                let field = CredentialField {
                    field_type: FieldType::Text, // Default for now
                    value,
                    sensitive: false, // Default for now
                    label: Some(label.clone()),
                    metadata: HashMap::new(),
                };
                (label, field)
            })
            .collect();

        client
            .update_credential(
                session_id,
                id,
                title,
                credential_type,
                fields,
                Vec::new(), // tags
                None,       // notes
            )
            .await
    }
}
