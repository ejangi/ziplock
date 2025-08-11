//! Main Application View for ZipLock Linux Frontend
//!
//! This view represents the primary interface shown after the initial setup wizard.
//! It demonstrates how to use the shared theme system across different views.

use iced::{
    widget::{button, column, container, row, scrollable, svg, text, text_input, Space},
    Alignment, Command, Element, Length,
};
use tracing::debug;

use crate::ipc::IpcClient;
use crate::ui::theme::alerts::AlertMessage;
use crate::ui::theme::container_styles;
use crate::ui::{button_styles, theme, utils};

/// Messages for the main application view
#[derive(Debug, Clone)]
pub enum MainViewMessage {
    /// Search query changed
    SearchChanged(String),
    ClearSearch,

    // Credential management
    AddCredential,
    EditCredential(String),
    CredentialClicked(String),
    DeleteCredential(String),
    RefreshCredentials,

    // Async results
    CredentialsLoaded(Result<(Vec<CredentialItem>, Option<String>, bool), String>),
    OperationCompleted(Result<String, String>),

    // UI navigation
    ShowSettings,
    ShowAbout,

    // Session management
    SessionTimeout,

    // Error handling
    ShowError(String),
    DismissError,
    TriggerConnectionError,
    TriggerAuthError,
    TriggerValidationError,
}

/// Main application view state
#[derive(Debug)]
pub struct MainView {
    search_query: String,
    credentials: Vec<CredentialItem>,
    session_id: Option<String>,
    is_authenticated: bool,
    selected_credential: Option<String>,
    is_loading: bool,
    current_error: Option<AlertMessage>,
}

/// Represents a credential item in the list
#[derive(Debug, Clone)]
pub struct CredentialItem {
    pub id: String,
    pub title: String,
    pub username: String,
    pub url: Option<String>,
    pub last_modified: String,
}

impl Default for MainView {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            credentials: vec![
                CredentialItem {
                    id: "1".to_string(),
                    title: "GitHub".to_string(),
                    username: "user@example.com".to_string(),
                    url: Some("https://github.com".to_string()),
                    last_modified: "2 days ago".to_string(),
                },
                CredentialItem {
                    id: "2".to_string(),
                    title: "Gmail".to_string(),
                    username: "user@gmail.com".to_string(),
                    url: Some("https://gmail.com".to_string()),
                    last_modified: "1 week ago".to_string(),
                },
            ],
            session_id: None,
            is_authenticated: false,
            selected_credential: None,
            is_loading: false,
            current_error: None,
        }
    }
}

impl MainView {
    /// Create a new main view instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the session ID for this view
    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
        // Don't automatically set authenticated=true just because we have a session
        // Authentication status will be updated when we actually load credentials
    }

    /// Create a command to refresh credentials if we have a session
    pub fn initial_refresh_command(&self) -> Command<MainViewMessage> {
        if self.session_id.is_some() {
            Command::perform(
                Self::load_credentials_async(self.session_id.clone()),
                MainViewMessage::CredentialsLoaded,
            )
        } else {
            Command::none()
        }
    }

    /// Create a command to refresh credentials (public method for external use)
    pub fn refresh_credentials(&mut self) -> Command<MainViewMessage> {
        self.is_loading = true;
        self.current_error = None;
        if self.session_id.is_some() {
            Command::perform(
                Self::load_credentials_async(self.session_id.clone()),
                MainViewMessage::CredentialsLoaded,
            )
        } else {
            Command::none()
        }
    }

    /// Update the main view based on messages
    pub fn update(&mut self, message: MainViewMessage) -> Command<MainViewMessage> {
        match message {
            MainViewMessage::SearchChanged(query) => {
                self.search_query = query;
                Command::none()
            }

            MainViewMessage::ClearSearch => {
                self.search_query.clear();
                Command::none()
            }

            MainViewMessage::AddCredential => {
                // TODO: Show add credential dialog
                Command::none()
            }

            MainViewMessage::EditCredential(id) => {
                self.selected_credential = Some(id);
                // TODO: Show edit credential dialog
                Command::none()
            }

            MainViewMessage::CredentialClicked(id) => {
                self.selected_credential = Some(id);
                // TODO: Show edit credential dialog
                Command::none()
            }

            MainViewMessage::DeleteCredential(_id) => {
                // TODO: Show confirmation dialog and delete
                Command::none()
            }

            MainViewMessage::RefreshCredentials => {
                self.is_loading = true;
                self.current_error = None;
                Command::perform(
                    Self::load_credentials_async(self.session_id.clone()),
                    MainViewMessage::CredentialsLoaded,
                )
            }

            MainViewMessage::CredentialsLoaded(result) => {
                self.is_loading = false;
                match result {
                    Ok((credentials, session_id, authenticated)) => {
                        let cred_count = credentials.len();
                        self.credentials = credentials;
                        if let Some(sid) = session_id {
                            self.session_id = Some(sid);
                        }
                        self.is_authenticated = authenticated;
                        self.current_error = None;
                        // Log for debugging
                        if authenticated {
                            tracing::debug!(
                                "Successfully loaded {} credentials, authenticated=true",
                                cred_count
                            );
                        } else {
                            tracing::debug!(
                                "Loaded {} credentials but authenticated=false",
                                cred_count
                            );
                        }
                    }
                    Err(e) => {
                        // Check if this is a session timeout error
                        if let Some(timeout_command) = self.handle_potential_session_timeout(&e) {
                            return timeout_command;
                        }
                        self.current_error = Some(AlertMessage::error(e));
                        self.is_authenticated = false;
                    }
                }
                Command::none()
            }

            MainViewMessage::OperationCompleted(result) => {
                self.is_loading = false;
                match result {
                    Ok(success_msg) => {
                        if success_msg.contains("locked") {
                            // If we locked the database, clear our session and credentials
                            self.session_id = None;
                            self.is_authenticated = false;
                            self.credentials.clear();
                            self.current_error = Some(AlertMessage::success(success_msg));
                            Command::none()
                        } else {
                            self.current_error = Some(AlertMessage::success(success_msg));
                            // Auto-refresh credentials after successful operation
                            Command::perform(
                                Self::load_credentials_async(self.session_id.clone()),
                                MainViewMessage::CredentialsLoaded,
                            )
                        }
                    }
                    Err(error_msg) => {
                        // Check if this is a session timeout error
                        if let Some(timeout_command) =
                            self.handle_potential_session_timeout(&error_msg)
                        {
                            return timeout_command;
                        }
                        self.current_error = Some(AlertMessage::error(error_msg));
                        Command::none()
                    }
                }
            }

            MainViewMessage::ShowError(error) => {
                self.current_error = Some(AlertMessage::error(error));
                Command::none()
            }

            MainViewMessage::DismissError => {
                self.current_error = None;
                Command::none()
            }

            // Error demonstration handlers
            MainViewMessage::TriggerConnectionError => {
                self.current_error = Some(AlertMessage::ipc_error(
                    "Unable to connect to the ZipLock backend service. Please ensure the daemon is running."
                ));
                Command::none()
            }

            MainViewMessage::TriggerAuthError => {
                self.current_error = Some(AlertMessage::ipc_error(
                    "Authentication failed. Please check your passphrase and try again.",
                ));
                Command::none()
            }

            MainViewMessage::TriggerValidationError => {
                self.current_error = Some(AlertMessage::error(
                    "Invalid data provided. Please check your input and try again.",
                ));
                Command::none()
            }

            MainViewMessage::ShowSettings => {
                // TODO: Navigate to settings view
                Command::none()
            }

            MainViewMessage::ShowAbout => {
                // TODO: Show about dialog
                Command::none()
            }

            MainViewMessage::SessionTimeout => {
                // This is handled by the helper method and parent application
                // Just return none as the timeout has already been processed
                Command::none()
            }
        }
    }

    /// Render the main view
    pub fn view(&self) -> Element<MainViewMessage> {
        let sidebar = self.view_sidebar();
        let main_content = self.view_main_content();

        row![sidebar, main_content]
            .spacing(0)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    /// Render the left sidebar with logo and action buttons
    fn view_sidebar(&self) -> Element<MainViewMessage> {
        let logo = container(
            svg(theme::ziplock_logo())
                .width(Length::Fixed(48.0))
                .height(Length::Fixed(48.0)),
        )
        .padding([20, 0, 30, 0])
        .width(Length::Fill)
        .center_x();

        let add_button = container(
            button(
                svg(theme::plus_icon())
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0)),
            )
            .on_press(MainViewMessage::AddCredential)
            .padding(12)
            .style(button_styles::primary()),
        )
        .width(Length::Fill)
        .center_x();

        let settings_button = container(
            button(
                svg(theme::settings_icon())
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0)),
            )
            .on_press(MainViewMessage::ShowSettings)
            .padding(12)
            .style(button_styles::secondary()),
        )
        .width(Length::Fill)
        .center_x();

        let sidebar_content = column![
            logo,
            Space::with_height(Length::Fixed(30.0)),
            add_button,
            Space::with_height(Length::Fill),
            settings_button,
        ]
        .spacing(0)
        .padding(20)
        .width(Length::Fixed(120.0))
        .height(Length::Fill);

        container(sidebar_content)
            .style(container_styles::sidebar())
            .width(Length::Fixed(120.0))
            .height(Length::Fill)
            .into()
    }

    /// Render the main content area with search and credentials
    fn view_main_content(&self) -> Element<MainViewMessage> {
        let search_bar = self.view_search_bar();

        let mut content_column = column![
            Space::with_height(Length::Fixed(20.0)),
            search_bar,
            Space::with_height(Length::Fixed(utils::standard_spacing().into())),
        ];

        // Add error alert if present
        if let Some(error_alert) = &self.current_error {
            content_column = content_column.push(crate::ui::theme::alerts::render_alert(
                error_alert,
                Some(MainViewMessage::DismissError),
            ));
            content_column = content_column.push(Space::with_height(Length::Fixed(10.0)));
        }

        let credential_list = self.view_credential_list();
        content_column = content_column.push(credential_list);

        let main_content = content_column.padding([0, 30, 30, 30]).spacing(10);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Render the search bar
    fn view_search_bar(&self) -> Element<MainViewMessage> {
        row![
            text_input("Search credentials...", &self.search_query)
                .on_input(MainViewMessage::SearchChanged)
                .width(Length::FillPortion(3))
                .padding([8, 12]),
            Space::with_width(Length::Fixed(10.0)),
            if !self.search_query.is_empty() {
                button("Clear")
                    .on_press(MainViewMessage::ClearSearch)
                    .padding(utils::small_button_padding())
                    .style(button_styles::secondary())
            } else {
                button("Clear")
                    .padding(utils::small_button_padding())
                    .style(button_styles::disabled())
            }
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Render the list of credentials
    fn view_credential_list(&self) -> Element<MainViewMessage> {
        if self.is_loading {
            return column![
                Space::with_height(Length::Fixed(50.0)),
                text("Loading credentials...")
                    .size(16)
                    .style(iced::theme::Text::Color(theme::LOGO_PURPLE)),
                Space::with_height(Length::Fixed(20.0)),
                text("Please wait while we fetch your credentials from the backend...")
                    .size(12)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.7, 0.7, 0.7
                    ))),
            ]
            .align_items(Alignment::Center)
            .into();
        }

        let filtered_credentials: Vec<&CredentialItem> = self
            .credentials
            .iter()
            .filter(|cred| {
                if self.search_query.is_empty() {
                    true
                } else {
                    let query_lower = self.search_query.to_lowercase();
                    cred.title.to_lowercase().contains(&query_lower)
                        || cred.username.to_lowercase().contains(&query_lower)
                        || cred
                            .url
                            .as_ref()
                            .map_or(false, |url| url.to_lowercase().contains(&query_lower))
                }
            })
            .collect();

        if filtered_credentials.is_empty() {
            return if self.search_query.is_empty() {
                if self.is_authenticated {
                    // No credentials and authenticated - show friendly empty state
                    column![
                        Space::with_height(Length::Fixed(80.0)),
                        text("No credentials yet!")
                            .size(24)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                                0.4, 0.4, 0.4
                            ))),
                        Space::with_height(Length::Fixed(10.0)),
                        text("Let's add your first credential to get started")
                            .size(16)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                                0.6, 0.6, 0.6
                            ))),
                        Space::with_height(Length::Fixed(30.0)),
                        button(
                            row![
                                svg(theme::plus_icon())
                                    .width(Length::Fixed(18.0))
                                    .height(Length::Fixed(18.0)),
                                Space::with_width(Length::Fixed(8.0)),
                                text("Add Your First Credential").size(16)
                            ]
                            .align_items(Alignment::Center)
                        )
                        .on_press(MainViewMessage::AddCredential)
                        .padding([12, 24])
                        .style(button_styles::primary()),
                        Space::with_height(Length::Fixed(20.0)),
                        text("or click 'Refresh' to reload from backend")
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                                0.7, 0.7, 0.7
                            ))),
                    ]
                    .align_items(Alignment::Center)
                    .into()
                } else {
                    // Not authenticated - show locked state
                    column![
                        Space::with_height(Length::Fixed(50.0)),
                        text("Database is locked")
                            .size(16)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                                0.5, 0.5, 0.5
                            ))),
                        text("Please unlock it first to view credentials.")
                            .size(14)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                                0.9, 0.6, 0.4
                            ))),
                    ]
                    .align_items(Alignment::Center)
                    .into()
                }
            } else {
                // Search returned no results
                column![
                    Space::with_height(Length::Fixed(50.0)),
                    text("No credentials found")
                        .size(16)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(
                            0.5, 0.5, 0.5
                        ))),
                    text("Try adjusting your search terms").size(14).style(
                        iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))
                    ),
                ]
                .align_items(Alignment::Center)
                .into()
            };
        }

        let credential_items: Vec<Element<MainViewMessage>> = filtered_credentials
            .iter()
            .map(|credential| self.view_credential_item(credential))
            .collect();

        scrollable(column(credential_items).spacing(10).padding([10, 0]))
            .height(Length::Fill)
            .into()
    }

    /// Render a single credential item
    fn view_credential_item(&self, credential: &CredentialItem) -> Element<MainViewMessage> {
        let is_selected = self
            .selected_credential
            .as_ref()
            .map_or(false, |id| id == &credential.id);

        let background_color = if is_selected {
            iced::Color::from_rgba(0.514, 0.220, 0.925, 0.1) // Light purple tint
        } else {
            iced::Color::WHITE
        };

        let border_color = if is_selected {
            theme::LOGO_PURPLE
        } else {
            iced::Color::from_rgb(0.9, 0.9, 0.9)
        };

        button(
            row![
                column![
                    text(&credential.title)
                        .size(16)
                        .style(iced::theme::Text::Color(theme::DARK_TEXT)),
                    text(&credential.username)
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(
                            0.6, 0.6, 0.6
                        ))),
                    if let Some(url) = &credential.url {
                        text(url)
                            .size(10)
                            .style(iced::theme::Text::Color(theme::LOGO_PURPLE))
                    } else {
                        text("")
                    }
                ]
                .width(Length::Fill)
                .spacing(4),
                column![
                    Space::with_height(Length::Fixed(10.0)),
                    row![button("Edit")
                        .on_press(MainViewMessage::EditCredential(credential.id.clone()))
                        .padding(utils::small_button_padding())
                        .style(button_styles::secondary()),]
                ]
                .align_items(Alignment::End)
            ]
            .padding(15)
            .align_items(Alignment::Center),
        )
        .on_press(MainViewMessage::CredentialClicked(credential.id.clone()))
        .width(Length::Fill)
        .style(iced::theme::Button::Custom(Box::new(
            CredentialItemButtonStyle {
                background_color,
                border_color,
            },
        )))
        .into()
    }

    /// Async function to load credentials from backend
    async fn load_credentials_async(
        session_id: Option<String>,
    ) -> Result<(Vec<CredentialItem>, Option<String>, bool), String> {
        let mut client = IpcClient::new().map_err(|e| e.to_string())?;

        // Connect to backend
        client
            .connect()
            .await
            .map_err(|e| format!("Could not connect to ZipLock backend: {}", e))?;

        // Create a session if we don't have one
        let current_session_id = if let Some(sid) = session_id {
            client.set_session_id(sid.clone());
            sid
        } else {
            // Create a new session
            client
                .create_session()
                .await
                .map_err(|e| format!("Failed to create session: {}", e))?;

            // Get the session ID from the client
            match client.get_session_id() {
                Some(sid) => sid,
                None => return Err("Failed to obtain session ID after creation".to_string()),
            }
        };

        // Try listing credentials with the session
        match client.list_credentials().await {
            Ok(records) => {
                let credentials = records
                    .into_iter()
                    .map(|record| CredentialItem {
                        id: record.id,
                        title: record.title,
                        username: format!("Type: {}", record.credential_type),
                        url: None,
                        last_modified: record
                            .updated_at
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .to_string()
                            + " (timestamp)",
                    })
                    .collect();
                Ok((credentials, Some(current_session_id), true))
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("Database not unlocked")
                    || error_msg.contains("NotAuthenticated")
                {
                    // Database is not unlocked - this is expected behavior
                    Ok((Vec::new(), Some(current_session_id), false))
                } else if crate::ipc::IpcClient::is_session_timeout_error(&error_msg) {
                    // Session timeout - return error to trigger timeout handling
                    Err(format!("Session expired: {}", e))
                } else {
                    Err(format!("Failed to load credentials: {}", e))
                }
            }
        }
    }

    /// Async function to lock the database

    /// Handle potential session timeout errors
    fn handle_potential_session_timeout(
        &mut self,
        error_msg: &str,
    ) -> Option<Command<MainViewMessage>> {
        if crate::ipc::IpcClient::is_session_timeout_error(error_msg) {
            // Clear local session state immediately
            self.session_id = None;
            self.is_authenticated = false;
            self.credentials.clear();
            self.current_error = Some(AlertMessage::warning(
                "Your session has expired. You will be redirected to unlock your repository."
                    .to_string(),
            ));
            // Return command to trigger session timeout handling
            Some(Command::perform(async {}, |_| {
                MainViewMessage::SessionTimeout
            }))
        } else {
            None
        }
    }

    /// Dismiss the current error alert
    pub fn dismiss_error(&mut self) {
        self.current_error = None;
    }

    /// Check if there's currently an error to display
    pub fn has_error(&self) -> bool {
        self.current_error.is_some()
    }
}

/// Custom container style for credential items
struct CredentialItemButtonStyle {
    background_color: iced::Color,
    border_color: iced::Color,
}

impl button::StyleSheet for CredentialItemButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.background_color)),
            border: iced::Border {
                color: self.border_color,
                width: 1.0,
                radius: iced::border::Radius::from(8.0),
            },
            text_color: theme::DARK_TEXT,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.514, 0.220, 0.925, 0.05,
            ))),
            border: iced::Border {
                color: theme::LOGO_PURPLE,
                width: 1.0,
                radius: iced::border::Radius::from(8.0),
            },
            text_color: theme::DARK_TEXT,
            ..Default::default()
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.hovered(style)
    }

    fn disabled(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.95, 0.95, 0.95,
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.9, 0.9, 0.9),
                width: 1.0,
                radius: iced::border::Radius::from(8.0),
            },
            text_color: iced::Color::from_rgb(0.6, 0.6, 0.6),
            ..Default::default()
        }
    }
}
