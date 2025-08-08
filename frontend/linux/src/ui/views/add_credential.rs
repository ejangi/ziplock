//! Add Credential View for ZipLock Linux Frontend
//!
//! This view provides a comprehensive credential creation workflow including:
//! - Credential type selection
//! - Dynamic form generation based on selected type
//! - Field validation and user input handling
//! - Integration with backend credential creation API

use iced::{
    widget::{button, column, container, text, Space},
    Alignment, Command, Element, Length,
};
use std::collections::HashMap;
use ziplock_shared::models::{CommonTemplates, CredentialField, CredentialTemplate, FieldType};

use crate::ipc::IpcClient;
use crate::ui::components::{CredentialForm, CredentialFormConfig, CredentialFormMessage};
use crate::ui::theme::alerts::AlertMessage;
use crate::ui::theme::{button_styles, container_styles};

/// Messages for the add credential view
#[derive(Debug, Clone)]
pub enum AddCredentialMessage {
    // Navigation
    Cancel,

    // Type selection
    TypeSelected(String),
    RefreshTypes,
    TypesLoaded(Result<Vec<CredentialTemplate>, String>),

    // Form handling
    FormMessage(CredentialFormMessage),

    // Actions
    CreateCredential,
    CredentialCreated(Result<String, String>),
}

impl PartialEq for AddCredentialMessage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AddCredentialMessage::Cancel, AddCredentialMessage::Cancel) => true,
            (AddCredentialMessage::TypeSelected(a), AddCredentialMessage::TypeSelected(b)) => {
                a == b
            }
            (AddCredentialMessage::RefreshTypes, AddCredentialMessage::RefreshTypes) => true,
            (AddCredentialMessage::CreateCredential, AddCredentialMessage::CreateCredential) => {
                true
            }
            (AddCredentialMessage::FormMessage(a), AddCredentialMessage::FormMessage(b)) => a == b,
            (
                AddCredentialMessage::TypesLoaded(Ok(_)),
                AddCredentialMessage::TypesLoaded(Ok(_)),
            ) => true,
            (
                AddCredentialMessage::TypesLoaded(Err(_)),
                AddCredentialMessage::TypesLoaded(Err(_)),
            ) => true,
            (
                AddCredentialMessage::CredentialCreated(Ok(_)),
                AddCredentialMessage::CredentialCreated(Ok(_)),
            ) => true,
            (
                AddCredentialMessage::CredentialCreated(Err(_)),
                AddCredentialMessage::CredentialCreated(Err(_)),
            ) => true,
            _ => false,
        }
    }
}

/// States for the add credential view
#[derive(Debug, Clone, PartialEq)]
pub enum AddCredentialState {
    SelectingType,
    FillingForm,
    Creating,
    Complete,
    Error(String),
}

/// The add credential view
#[derive(Debug)]
pub struct AddCredentialView {
    /// Current state of the creation process
    state: AddCredentialState,

    /// Available credential types/templates
    available_types: Vec<CredentialTemplate>,

    /// Currently selected credential type
    selected_type: Option<CredentialTemplate>,

    /// The credential form component
    form: CredentialForm,

    /// Current error message if any
    current_error: Option<AlertMessage>,

    /// Loading state
    is_loading: bool,

    /// Session ID for backend communication
    session_id: Option<String>,
}

impl Default for AddCredentialView {
    fn default() -> Self {
        Self::new()
    }
}

impl AddCredentialView {
    /// Create a new add credential view
    pub fn new() -> Self {
        Self {
            state: AddCredentialState::SelectingType,
            available_types: Self::get_builtin_templates(),
            selected_type: None,
            form: CredentialForm::new(),
            current_error: None,
            is_loading: false,
            session_id: None,
        }
    }

    /// Create a new add credential view with a session ID
    pub fn with_session(session_id: Option<String>) -> Self {
        let mut view = Self::new();
        view.session_id = session_id;
        view
    }

    /// Get built-in credential templates
    fn get_builtin_templates() -> Vec<CredentialTemplate> {
        vec![
            CommonTemplates::login(),
            CommonTemplates::credit_card(),
            CommonTemplates::secure_note(),
        ]
    }

    /// Update the view based on a message
    pub fn update(&mut self, message: AddCredentialMessage) -> Command<AddCredentialMessage> {
        match message {
            AddCredentialMessage::Cancel => {
                // Parent view will handle transition back to main view
                Command::none()
            }

            AddCredentialMessage::TypeSelected(type_name) => {
                if let Some(template) = self
                    .available_types
                    .iter()
                    .find(|t| t.name == type_name)
                    .cloned()
                {
                    self.selected_type = Some(template.clone());

                    // Set up the form with the selected template
                    self.form.set_template(template);

                    // Configure the form for adding credentials
                    let mut config = CredentialFormConfig::default();
                    config.save_button_text = "Save".to_string();
                    config.show_cancel_button = true;
                    config.is_loading = false;
                    self.form.set_config(config);

                    self.state = AddCredentialState::FillingForm;
                    self.current_error = None;
                }
                Command::none()
            }

            AddCredentialMessage::RefreshTypes => Command::perform(
                Self::load_credential_types_async(self.session_id.clone()),
                AddCredentialMessage::TypesLoaded,
            ),

            AddCredentialMessage::TypesLoaded(result) => {
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

            AddCredentialMessage::FormMessage(form_msg) => {
                match form_msg {
                    CredentialFormMessage::Save => {
                        tracing::debug!("Save button clicked in add credential view");
                        return Command::perform(async {}, |_| {
                            AddCredentialMessage::CreateCredential
                        });
                    }
                    CredentialFormMessage::Cancel => {
                        tracing::debug!("Cancel button clicked in add credential view");
                        return Command::perform(async {}, |_| AddCredentialMessage::Cancel);
                    }
                    _ => {
                        self.form.update(form_msg);
                    }
                }
                Command::none()
            }

            AddCredentialMessage::CreateCredential => {
                tracing::debug!("Processing CreateCredential message");
                if !self.form.is_valid() {
                    tracing::warn!("Form validation failed in add credential");
                    self.current_error = Some(AlertMessage::warning(
                        "Please fill in all required fields".to_string(),
                    ));
                    return Command::none();
                }

                tracing::debug!("Form validation passed, proceeding with credential creation");
                self.state = AddCredentialState::Creating;
                self.is_loading = true;

                // Update form config to show loading state
                let mut config = CredentialFormConfig::default();
                config.save_button_text = "Save".to_string();
                config.show_cancel_button = true;
                config.is_loading = true;
                self.form.set_config(config);

                Command::perform(
                    Self::create_credential_async(
                        self.session_id.clone(),
                        self.form.title().to_string(),
                        self.form.field_values().clone(),
                        self.selected_type
                            .as_ref()
                            .map(|t| t.name.clone())
                            .unwrap_or_default(),
                    ),
                    AddCredentialMessage::CredentialCreated,
                )
            }

            AddCredentialMessage::CredentialCreated(result) => {
                self.is_loading = false;
                match result {
                    Ok(_id) => {
                        tracing::info!("Credential created successfully");
                        self.state = AddCredentialState::Complete;
                        self.current_error = None;
                    }
                    Err(e) => {
                        tracing::error!("Failed to create credential: {}", e);
                        self.current_error = Some(AlertMessage::error(e));
                        self.state =
                            AddCredentialState::Error("Failed to create credential".to_string());

                        // Reset form config to not loading state
                        let mut config = CredentialFormConfig::default();
                        config.save_button_text = "Save".to_string();
                        config.show_cancel_button = true;
                        config.is_loading = false;
                        config.error_message =
                            self.current_error.as_ref().map(|a| a.message.clone());
                        self.form.set_config(config);
                    }
                }
                Command::none()
            }
        }
    }

    /// Render the add credential view
    pub fn view(&self) -> Element<AddCredentialMessage> {
        match &self.state {
            AddCredentialState::SelectingType => self.view_type_selection(),
            AddCredentialState::FillingForm => self.view_credential_form(),
            AddCredentialState::Creating => self.view_creating(),
            AddCredentialState::Complete => self.view_complete(),
            AddCredentialState::Error(_) => self.view_error(),
        }
    }

    /// Render the view header
    fn view_header(&self) -> Element<AddCredentialMessage> {
        text("Add New Credential").size(24).into()
    }

    /// Render the type selection state
    fn view_type_selection(&self) -> Element<AddCredentialMessage> {
        let mut type_buttons = vec![];

        for template in &self.available_types {
            let button_element = button(text(&template.name).size(16))
                .on_press(AddCredentialMessage::TypeSelected(template.name.clone()))
                .style(button_styles::primary())
                .width(Length::Fill)
                .padding(15);

            type_buttons.push(button_element.into());
            type_buttons.push(Space::with_height(Length::Fixed(10.0)).into());
        }

        // Add cancel button
        type_buttons.push(Space::with_height(Length::Fixed(20.0)).into());
        type_buttons.push(
            button("Cancel")
                .on_press(AddCredentialMessage::Cancel)
                .style(button_styles::secondary())
                .into(),
        );

        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(30.0)),
                text("Select the type of credential to create:").size(16),
                Space::with_height(Length::Fixed(20.0)),
                column(type_buttons).spacing(5).width(Length::Fixed(300.0)),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
        )
        .padding(40)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(container_styles::sidebar())
        .into()
    }

    /// Render the credential form state
    fn view_credential_form(&self) -> Element<AddCredentialMessage> {
        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(20.0)),
                self.form.view().map(AddCredentialMessage::FormMessage),
            ]
            .spacing(10),
        )
        .padding(40)
        .height(Length::Fill)
        .style(container_styles::sidebar())
        .into()
    }

    /// Render the creating state
    fn view_creating(&self) -> Element<AddCredentialMessage> {
        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(40.0)),
                text("Creating credential...").size(16),
                Space::with_height(Length::Fixed(20.0)),
                self.form.view().map(AddCredentialMessage::FormMessage),
            ]
            .spacing(10),
        )
        .padding(40)
        .height(Length::Fill)
        .style(container_styles::sidebar())
        .into()
    }

    /// Render the completion state
    fn view_complete(&self) -> Element<AddCredentialMessage> {
        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(40.0)),
                text("✅ Credential created successfully!").size(18).style(
                    iced::theme::Text::Color(iced::Color::from_rgb(0.02, 0.84, 0.63))
                ), // Success green
                Space::with_height(Length::Fixed(20.0)),
                text("You will be returned to the main view shortly.").size(14),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
        )
        .padding(40)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(container_styles::sidebar())
        .into()
    }

    /// Render the error state
    fn view_error(&self) -> Element<AddCredentialMessage> {
        let error_message = self
            .current_error
            .as_ref()
            .map(|e| e.message.as_str())
            .unwrap_or("An unknown error occurred");

        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(20.0)),
                text("Error creating credential:").size(16),
                text(error_message).size(14).style(iced::theme::Text::Color(
                    iced::Color::from_rgb(0.94, 0.28, 0.44)
                )), // Error red
                Space::with_height(Length::Fixed(20.0)),
                self.form.view().map(AddCredentialMessage::FormMessage),
            ]
            .spacing(10),
        )
        .padding(40)
        .height(Length::Fill)
        .style(container_styles::sidebar())
        .into()
    }

    /// Check if the creation process is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.state, AddCredentialState::Complete)
    }

    /// Check if the creation process was cancelled
    pub fn is_cancelled(&self) -> bool {
        // This would be set by the parent view based on the Cancel message
        false
    }

    /// Load credential types asynchronously
    async fn load_credential_types_async(
        session_id: Option<String>,
    ) -> Result<Vec<CredentialTemplate>, String> {
        let mut client = IpcClient::new().map_err(|e| e.to_string())?;
        client.get_credential_types(session_id).await
    }

    /// Create a credential asynchronously
    async fn create_credential_async(
        session_id: Option<String>,
        title: String,
        field_values: HashMap<String, String>,
        credential_type: String,
    ) -> Result<String, String> {
        let mut client = IpcClient::new().map_err(|e| e.to_string())?;

        // Get the template to properly map field types and sensitivity
        let template = match credential_type.as_str() {
            "login" => Some(ziplock_shared::models::CommonTemplates::login()),
            "credit_card" => Some(ziplock_shared::models::CommonTemplates::credit_card()),
            "secure_note" => Some(ziplock_shared::models::CommonTemplates::secure_note()),
            _ => None,
        };

        // Convert field_values (HashMap<String, String>) to HashMap<String, CredentialField>
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

        tracing::debug!("Calling IPC client to create credential");
        tracing::debug!("Title: {}", title);
        tracing::debug!("Credential type: {}", credential_type);
        tracing::debug!("Fields: {:?}", fields);

        client
            .create_credential(session_id, title, credential_type, fields, Vec::new(), None)
            .await
    }
}
