//! Add Credential View for ZipLock Linux Frontend
//!
//! This view provides a comprehensive credential creation workflow including:
//! - Credential type selection
//! - Dynamic form generation based on selected type
//! - Field validation and user input handling
//! - Integration with backend credential creation API

use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Command, Element, Length,
};
use std::collections::HashMap;
use ziplock_shared::models::{
    CommonTemplates, CredentialField, CredentialRecord, CredentialTemplate, FieldType,
};

use crate::ui::theme::alerts::AlertMessage;
use crate::ui::{button_styles, theme};

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
    TitleChanged(String),
    FieldChanged(String, String),   // field_name, new_value
    ToggleFieldSensitivity(String), // field_name

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
            (AddCredentialMessage::TitleChanged(a), AddCredentialMessage::TitleChanged(b)) => {
                a == b
            }
            (
                AddCredentialMessage::FieldChanged(a1, a2),
                AddCredentialMessage::FieldChanged(b1, b2),
            ) => a1 == b1 && a2 == b2,
            (
                AddCredentialMessage::ToggleFieldSensitivity(a),
                AddCredentialMessage::ToggleFieldSensitivity(b),
            ) => a == b,
            (AddCredentialMessage::CreateCredential, AddCredentialMessage::CreateCredential) => {
                true
            }
            _ => false,
        }
    }
}

/// State of the add credential process
#[derive(Debug, Clone, PartialEq)]
pub enum AddCredentialState {
    SelectingType,
    FillingForm,
    Creating,
    Complete,
    Error,
}

/// Add credential view component
#[derive(Debug)]
pub struct AddCredentialView {
    /// Current state of the creation process
    state: AddCredentialState,

    /// Available credential types/templates
    available_types: Vec<CredentialTemplate>,

    /// Currently selected credential type
    selected_type: Option<CredentialTemplate>,

    /// Credential title (required field)
    title: String,

    /// Form field values
    field_values: HashMap<String, String>,

    /// Which fields are marked as sensitive
    field_sensitivity: HashMap<String, bool>,

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
            title: String::new(),
            field_values: HashMap::new(),
            field_sensitivity: HashMap::new(),
            current_error: None,
            is_loading: false,
            session_id: None,
        }
    }

    /// Create a new add credential view with a session ID
    pub fn with_session(session_id: Option<String>) -> Self {
        let mut view = Self::new();
        view.session_id = session_id.clone();
        if session_id.is_none() {
            // No session available, show error
            view.current_error = Some(AlertMessage::error(
                "No session available. Please unlock the database first.".to_string(),
            ));
            view.state = AddCredentialState::Error;
        }
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

    /// Update the view based on messages
    pub fn update(&mut self, message: AddCredentialMessage) -> Command<AddCredentialMessage> {
        match message {
            AddCredentialMessage::Cancel => {
                // Reset state and let parent handle navigation
                *self = Self::new();
                Command::none()
            }

            AddCredentialMessage::TypeSelected(type_name) => {
                if let Some(template) = self.available_types.iter().find(|t| t.name == type_name) {
                    self.selected_type = Some(template.clone());
                    self.state = AddCredentialState::FillingForm;

                    // Initialize field values and sensitivity from template
                    self.field_values.clear();
                    self.field_sensitivity.clear();

                    for field_template in &template.fields {
                        self.field_values.insert(
                            field_template.name.clone(),
                            field_template.default_value.clone().unwrap_or_default(),
                        );
                        self.field_sensitivity
                            .insert(field_template.name.clone(), field_template.sensitive);
                    }
                }
                Command::none()
            }

            AddCredentialMessage::RefreshTypes => {
                self.is_loading = true;
                // TODO: Load types from backend
                Command::perform(
                    Self::load_credential_types_async(),
                    AddCredentialMessage::TypesLoaded,
                )
            }

            AddCredentialMessage::TypesLoaded(result) => {
                self.is_loading = false;
                match result {
                    Ok(types) => {
                        self.available_types = types;
                        self.current_error = None;
                    }
                    Err(error) => {
                        self.current_error = Some(AlertMessage::error(error));
                    }
                }
                Command::none()
            }

            AddCredentialMessage::TitleChanged(new_title) => {
                self.title = new_title;
                Command::none()
            }

            AddCredentialMessage::FieldChanged(field_name, new_value) => {
                self.field_values.insert(field_name, new_value);
                Command::none()
            }

            AddCredentialMessage::ToggleFieldSensitivity(field_name) => {
                if let Some(current) = self.field_sensitivity.get(&field_name) {
                    self.field_sensitivity.insert(field_name, !current);
                }
                Command::none()
            }

            AddCredentialMessage::CreateCredential => {
                if self.title.trim().is_empty() {
                    self.current_error = Some(AlertMessage::error("Title is required".to_string()));
                    return Command::none();
                }

                self.state = AddCredentialState::Creating;
                self.is_loading = true;
                self.current_error = None;

                Command::perform(
                    Self::create_credential_async(
                        self.title.clone(),
                        self.selected_type.clone(),
                        self.field_values.clone(),
                        self.field_sensitivity.clone(),
                        self.session_id.clone(),
                    ),
                    AddCredentialMessage::CredentialCreated,
                )
            }

            AddCredentialMessage::CredentialCreated(result) => {
                self.is_loading = false;
                match result {
                    Ok(success_msg) => {
                        self.state = AddCredentialState::Complete;
                        self.current_error = Some(AlertMessage::success(success_msg));
                    }
                    Err(error) => {
                        self.state = AddCredentialState::Error;
                        self.current_error = Some(AlertMessage::error(error));
                    }
                }
                Command::none()
            }
        }
    }

    /// Render the add credential view
    pub fn view(&self) -> Element<AddCredentialMessage> {
        let header = self.view_header();
        let content = match self.state {
            AddCredentialState::SelectingType => self.view_type_selection(),
            AddCredentialState::FillingForm => self.view_credential_form(),
            AddCredentialState::Creating => self.view_creating(),
            AddCredentialState::Complete => self.view_complete(),
            AddCredentialState::Error => self.view_error(),
        };

        let main_content = column![header, content].spacing(20);

        let body = column![main_content];

        container(body)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Render the header
    fn view_header(&self) -> Element<AddCredentialMessage> {
        row![
            button("← Back")
                .on_press(AddCredentialMessage::Cancel)
                .style(button_styles::secondary()),
            Space::with_width(Length::Fill),
            text("Add New Credential")
                .size(24)
                .style(iced::theme::Text::Color(theme::LOGO_PURPLE)),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Render credential type selection
    fn view_type_selection(&self) -> Element<AddCredentialMessage> {
        let types_list = self
            .available_types
            .iter()
            .map(|template| {
                button(
                    column![
                        text(&template.name.replace("_", " ").to_uppercase()).size(16),
                        text(&template.description)
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                                0.6, 0.6, 0.6
                            ))),
                    ]
                    .spacing(4),
                )
                .width(Length::Fill)
                .padding(15)
                .on_press(AddCredentialMessage::TypeSelected(template.name.clone()))
                .style(button_styles::secondary())
                .into()
            })
            .collect::<Vec<Element<AddCredentialMessage>>>();

        column![
            text("What type of credential would you like to add?").size(18),
            Space::with_height(Length::Fixed(20.0)),
            column(types_list).spacing(10),
            Space::with_height(Length::Fixed(20.0)),
            button("Refresh Types")
                .on_press(AddCredentialMessage::RefreshTypes)
                .style(button_styles::secondary()),
        ]
        .into()
    }

    /// Render credential form
    fn view_credential_form(&self) -> Element<AddCredentialMessage> {
        let template = match &self.selected_type {
            Some(t) => t,
            None => return text("No template selected").into(),
        };

        let mut form_fields = vec![
            // Title field (always first and required)
            text("Title *").size(14).into(),
            text_input("Enter credential title...", &self.title)
                .on_input(AddCredentialMessage::TitleChanged)
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

        // Add action buttons
        form_fields.push(Space::with_height(Length::Fixed(20.0)).into());
        form_fields.push(
            row![
                button("Cancel")
                    .on_press(AddCredentialMessage::Cancel)
                    .style(button_styles::secondary()),
                Space::with_width(Length::Fill),
                button("Create Credential")
                    .on_press(AddCredentialMessage::CreateCredential)
                    .style(button_styles::primary()),
            ]
            .into(),
        );

        scrollable(column(form_fields).spacing(5))
            .height(Length::Fill)
            .into()
    }

    /// Create input field based on field type
    fn create_field_input(
        &self,
        field_name: &str,
        field_type: &FieldType,
        value: &str,
        is_sensitive: bool,
    ) -> Element<AddCredentialMessage> {
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
                        move |input| AddCredentialMessage::FieldChanged(field_name.clone(), input)
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
                                AddCredentialMessage::FieldChanged(field_name.clone(), input)
                            }
                        })
                        .secure(is_sensitive)
                        .padding(10),
                    button(if is_sensitive { "👁" } else { "🙈" })
                        .on_press(AddCredentialMessage::ToggleFieldSensitivity(
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
                        move |input| AddCredentialMessage::FieldChanged(field_name.clone(), input)
                    })
                    .padding(10)
                    .into()
            }
        }
    }

    /// Render creating state
    fn view_creating(&self) -> Element<AddCredentialMessage> {
        column![
            Space::with_height(Length::Fixed(50.0)),
            text("Creating credential...")
                .size(18)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::with_height(Length::Fixed(20.0)),
            text("Please wait while we save your credential.")
                .size(14)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.6, 0.6, 0.6
                )))
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Render completion state
    fn view_complete(&self) -> Element<AddCredentialMessage> {
        column![
            Space::with_height(Length::Fixed(50.0)),
            text("✅ Credential Created!")
                .size(24)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.2, 0.7, 0.2
                )))
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::with_height(Length::Fixed(20.0)),
            text("Your credential has been successfully saved.")
                .size(14)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::with_height(Length::Fixed(30.0)),
            button("Done")
                .on_press(AddCredentialMessage::Cancel)
                .style(button_styles::primary()),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Render error state
    fn view_error(&self) -> Element<AddCredentialMessage> {
        column![
            Space::with_height(Length::Fixed(50.0)),
            text("❌ Creation Failed")
                .size(20)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.8, 0.2, 0.2
                )))
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::with_height(Length::Fixed(30.0)),
            row![
                button("Try Again")
                    .on_press(AddCredentialMessage::CreateCredential)
                    .style(button_styles::primary()),
                Space::with_width(Length::Fixed(10.0)),
                button("Cancel")
                    .on_press(AddCredentialMessage::Cancel)
                    .style(button_styles::secondary()),
            ]
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Check if the form is in a completable state
    pub fn is_complete(&self) -> bool {
        matches!(self.state, AddCredentialState::Complete)
    }

    /// Check if the operation was cancelled
    pub fn is_cancelled(&self) -> bool {
        false // Will be handled by parent via Cancel message
    }

    /// Async function to load credential types from backend
    async fn load_credential_types_async() -> Result<Vec<CredentialTemplate>, String> {
        // For now, return built-in templates
        // TODO: Implement backend API call to get custom types
        Ok(Self::get_builtin_templates())
    }

    /// Async function to create credential
    async fn create_credential_async(
        title: String,
        template: Option<CredentialTemplate>,
        field_values: HashMap<String, String>,
        field_sensitivity: HashMap<String, bool>,
        session_id: Option<String>,
    ) -> Result<String, String> {
        use crate::ipc::IpcClient;
        use std::time::SystemTime;
        use uuid::Uuid;

        let template = template.ok_or("No template selected")?;

        // Create credential record
        let mut credential = CredentialRecord {
            id: Uuid::new_v4().to_string(),
            title,
            credential_type: template.name,
            fields: HashMap::new(),
            tags: template.default_tags,
            notes: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        // Add fields from form
        for field_template in &template.fields {
            if let Some(value) = field_values.get(&field_template.name) {
                if !value.is_empty() || field_template.required {
                    let is_sensitive = field_sensitivity
                        .get(&field_template.name)
                        .copied()
                        .unwrap_or(field_template.sensitive);

                    let field = CredentialField {
                        field_type: field_template.field_type.clone(),
                        value: value.clone(),
                        sensitive: is_sensitive,
                        label: Some(field_template.label.clone()),
                        metadata: HashMap::new(),
                    };

                    credential.fields.insert(field_template.name.clone(), field);
                }
            }
        }

        // Connect to backend and create credential
        let socket_path = IpcClient::default_socket_path();
        let mut client = IpcClient::new(socket_path);

        client
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to backend: {}", e))?;

        // Set the session ID if we have one
        if let Some(sid) = session_id {
            client.set_session_id(sid);
        } else {
            return Err("No session ID available for authentication".to_string());
        }

        // Convert to CredentialData format expected by IpcClient
        let credential_data = crate::ipc::CredentialData {
            id: credential.id,
            title: credential.title.clone(),
            credential_type: credential.credential_type,
            fields: credential.fields.into_iter().collect(),
            tags: credential.tags,
            notes: credential.notes,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
        };

        client
            .create_credential(credential_data)
            .await
            .map_err(|e| format!("Failed to create credential: {}", e))?;

        Ok(format!(
            "Credential '{}' created successfully",
            credential.title
        ))
    }
}
