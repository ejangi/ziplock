//! Add Credential View for ZipLock Linux App
//!
//! This view provides a comprehensive credential creation workflow including:
//! - Credential type selection
//! - Dynamic form generation based on selected type
//! - Field validation and user input handling
//! - Integration with backend credential creation API

use iced::{
    alignment,
    widget::{button, column, container, row, scrollable, svg, text, Space},
    Alignment, Command, Element, Length,
};
use std::collections::HashMap;
use ziplock_shared::models::{CommonTemplates, CredentialField, CredentialTemplate, FieldType};
use ziplock_shared::utils::StringUtils;

use crate::ui::components::{CredentialForm, CredentialFormConfig, CredentialFormMessage};
use crate::ui::theme::{button_styles, container_styles, utils};

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

    // Error and success handling (for toast notifications)
    ShowError(String),
    ShowSuccess(String),
    ShowValidationError(String),
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
            (AddCredentialMessage::ShowError(_), AddCredentialMessage::ShowError(_)) => true,
            (AddCredentialMessage::ShowSuccess(_), AddCredentialMessage::ShowSuccess(_)) => true,
            (
                AddCredentialMessage::ShowValidationError(_),
                AddCredentialMessage::ShowValidationError(_),
            ) => true,
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
                    let config = CredentialFormConfig {
                        save_button_text: "Save".to_string(),
                        show_cancel_button: true,
                        is_loading: false,
                        ..CredentialFormConfig::default()
                    };
                    self.form.set_config(config);

                    self.state = AddCredentialState::FillingForm;
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
                    return Command::perform(
                        async { "Please fill in all required fields".to_string() },
                        AddCredentialMessage::ShowValidationError,
                    );
                }

                tracing::debug!("Form validation passed, proceeding with credential creation");
                self.state = AddCredentialState::Creating;
                self.is_loading = true;

                // Update form config to show loading state
                let config = CredentialFormConfig {
                    save_button_text: "Save".to_string(),
                    show_cancel_button: true,
                    is_loading: true,
                    ..CredentialFormConfig::default()
                };
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
                        Command::perform(
                            async { "Credential created successfully".to_string() },
                            AddCredentialMessage::ShowSuccess,
                        )
                    }
                    Err(e) => {
                        tracing::error!("Failed to create credential: {}", e);
                        self.state =
                            AddCredentialState::Error("Failed to create credential".to_string());

                        // Reset form config to not loading state
                        let config = CredentialFormConfig {
                            save_button_text: "Save".to_string(),
                            show_cancel_button: true,
                            is_loading: false,
                            ..CredentialFormConfig::default()
                        };
                        self.form.set_config(config);

                        Command::perform(async move { e }, AddCredentialMessage::ShowError)
                    }
                }
            }

            AddCredentialMessage::ShowError(_) => {
                // Error handling is now done at the application level via toast system
                Command::none()
            }

            AddCredentialMessage::ShowSuccess(_) => {
                // Success handling is now done at the application level via toast system
                Command::none()
            }

            AddCredentialMessage::ShowValidationError(_) => {
                // Validation error handling is now done at the application level via toast system
                Command::none()
            }
        }
    }

    /// Render the add credential view
    pub fn view(&self) -> Element<'_, AddCredentialMessage> {
        match &self.state {
            AddCredentialState::SelectingType => self.view_type_selection(),
            AddCredentialState::FillingForm => self.view_credential_form(),
            AddCredentialState::Creating => self.view_creating(),
            AddCredentialState::Complete => self.view_complete(),
            AddCredentialState::Error(_) => self.view_error(),
        }
    }

    /// Render the view header
    fn view_header(&self) -> Element<'_, AddCredentialMessage> {
        text("Add New Credential").size(24).into()
    }

    /// Render the type selection state
    fn view_type_selection(&self) -> Element<'_, AddCredentialMessage> {
        // Create a responsive grid layout for credential type buttons
        let mut grid_rows = vec![];
        let templates_per_row = 4; // Increased to 4 for better use of space

        // Process templates in chunks of 4
        for chunk in self.available_types.chunks(templates_per_row) {
            let mut row_buttons = vec![];

            for template in chunk {
                // Get icon SVG for credential type
                let icon_svg = match template.name.as_str() {
                    "login" => crate::ui::theme::lock_icon(),
                    "credit_card" => crate::ui::theme::credit_card_icon(),
                    "secure_note" => crate::ui::theme::note_icon(),
                    "identity" => crate::ui::theme::user_icon(),
                    "password" => crate::ui::theme::lock_icon(),
                    "document" => crate::ui::theme::document_icon(),
                    "ssh_key" => crate::ui::theme::settings_icon(),
                    "bank_account" => crate::ui::theme::bank_icon(),
                    "api_credentials" => crate::ui::theme::settings_icon(),
                    "crypto_wallet" => crate::ui::theme::wallet_icon(),
                    "database" => crate::ui::theme::database_icon(),
                    "software_license" => crate::ui::theme::license_icon(),
                    _ => crate::ui::theme::alert_icon(),
                };

                let button_element = button(
                    column![
                        container(
                            svg(icon_svg)
                                .width(Length::Fixed(24.0))
                                .height(Length::Fixed(24.0))
                        )
                        .width(Length::Fill)
                        .center_x(),
                        Space::with_height(Length::Fixed(12.0)),
                        container(
                            text(StringUtils::to_display_name(&template.name))
                                .size(16)
                                .horizontal_alignment(alignment::Horizontal::Center)
                        )
                        .width(Length::Fill)
                        .center_x()
                    ]
                    .align_items(Alignment::Center)
                    .spacing(0)
                    .width(Length::Fill),
                )
                .on_press(AddCredentialMessage::TypeSelected(template.name.clone()))
                .style(button_styles::primary())
                .width(Length::Fill)
                .height(Length::Fixed(90.0))
                .padding([15, 10]);

                row_buttons.push(button_element.into());
            }

            // Fill remaining space in incomplete rows
            while row_buttons.len() < templates_per_row {
                row_buttons.push(Space::with_width(Length::Fill).into());
            }

            grid_rows.push(row(row_buttons).spacing(18).into());
        }

        // Create the grid container with scrollable support
        let grid_container = container(
            scrollable(column(grid_rows).spacing(18))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([0, 30]); // More horizontal padding for better spacing

        // Cancel button at the bottom
        let cancel_button = container(
            button("Cancel")
                .on_press(AddCredentialMessage::Cancel)
                .style(button_styles::secondary())
                .padding(utils::button_padding()),
        )
        .width(Length::Fill)
        .center_x();

        container(
            column![
                self.view_header(),
                Space::with_height(Length::Fixed(20.0)),
                text("What type of information are we storing?")
                    .size(18)
                    .horizontal_alignment(alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(25.0)),
                grid_container,
                Space::with_height(Length::Fixed(20.0)),
                cancel_button,
            ]
            .spacing(0)
            .align_items(Alignment::Center),
        )
        .padding([30, 20]) // Reduced vertical padding, maintained horizontal
        .width(Length::Fill)
        .height(Length::Fill)
        .style(container_styles::sidebar())
        .into()
    }

    /// Render the credential form state
    fn view_credential_form(&self) -> Element<'_, AddCredentialMessage> {
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
    fn view_creating(&self) -> Element<'_, AddCredentialMessage> {
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
    fn view_complete(&self) -> Element<'_, AddCredentialMessage> {
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
    fn view_error(&self) -> Element<'_, AddCredentialMessage> {
        let error_message = "Failed to create credential";

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
        _session_id: Option<String>,
    ) -> Result<Vec<CredentialTemplate>, String> {
        // Return built-in templates directly - no need for client call
        Ok(vec![
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
        ])
    }

    /// Create a credential asynchronously
    async fn create_credential_async(
        session_id: Option<String>,
        title: String,
        field_values: HashMap<String, String>,
        credential_type: String,
    ) -> Result<String, String> {
        let mut client = ziplock_shared::ZipLockClient::new().map_err(|e| e.to_string())?;
        client.connect().await.map_err(|e| e.to_string())?;

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
            .map_err(|e| e.to_string())
    }

    /// Get subscriptions for TOTP field updates
    pub fn subscription(&self) -> iced::Subscription<AddCredentialMessage> {
        match &self.state {
            AddCredentialState::FillingForm => self
                .form
                .subscription()
                .map(AddCredentialMessage::FormMessage),
            _ => iced::Subscription::none(),
        }
    }
}
