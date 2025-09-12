//! Main Application View for ZipLock Linux App
//!
//! This view represents the primary interface shown after the initial setup wizard.
//! It demonstrates how to use the shared theme system across different views.

use crate::services::get_repository_service;

use crate::ui::{components::button as btn, theme, utils};
use iced::{
    widget::{button, column, container, row, scrollable, svg, text, text_input, Space},
    Alignment, Element, Length, Task,
};

/// Messages for the main application view
#[derive(Debug, Clone)]
pub enum MainViewMessage {
    // Search functionality
    SearchChanged(String),
    SearchSubmitted,
    ClearSearch,

    // Credential management
    AddCredential,
    EditCredential(String),
    CredentialClicked(String),
    DeleteCredential(String),
    RefreshCredentials,

    // Data operations
    CredentialsLoaded(Result<(Vec<CredentialItem>, Option<String>, bool), String>),
    OperationCompleted(Result<String, String>),

    // UI actions
    LockDatabase,
    ShowSettings,
    ShowAbout,
    CheckForUpdates,

    // Repository management
    CloseRepository,
    RepositoryOperationComplete(Result<String, String>),

    // Session management
    SessionTimeout,
    CloseArchive,

    // Error handling (now uses global toast system)
    ShowError(String),
    TriggerConnectionError,
    TriggerAuthError,
    TriggerValidationError,
}

/// Main application view state
#[derive(Debug, Default)]
pub struct MainView {
    search_query: String,
    credentials: Vec<CredentialItem>,
    filtered_credentials: Vec<CredentialItem>,
    session_id: Option<String>,
    is_authenticated: bool,
    selected_credential: Option<String>,
    is_loading: bool,
}

/// Represents a credential item in the list
#[derive(Debug, Clone)]
pub struct CredentialItem {
    pub id: String,
    pub title: String,
    pub username: String,
    pub url: Option<String>,
    pub last_modified: String,
    pub credential_type: String,
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
    pub fn initial_refresh_command(&self) -> Task<MainViewMessage> {
        if self.session_id.is_some() {
            Task::perform(
                Self::load_credentials_async(self.session_id.clone()),
                MainViewMessage::CredentialsLoaded,
            )
        } else {
            Task::none()
        }
    }

    /// Create a command to refresh credentials (public method for external use)
    pub fn refresh_credentials(&mut self) -> Task<MainViewMessage> {
        self.is_loading = true;
        if self.session_id.is_some() {
            Task::perform(
                Self::load_credentials_async(self.session_id.clone()),
                MainViewMessage::CredentialsLoaded,
            )
        } else {
            Task::none()
        }
    }

    /// Update the main view based on messages
    pub fn update(&mut self, message: MainViewMessage) -> Task<MainViewMessage> {
        match message {
            MainViewMessage::SearchChanged(query) => {
                self.search_query = query;
                self.filter_credentials();
                Task::none()
            }

            MainViewMessage::SearchSubmitted => {
                // Perform search using repository service for more advanced search
                if !self.search_query.trim().is_empty() {
                    self.is_loading = true;
                    Task::perform(
                        Self::search_credentials_async(self.search_query.clone()),
                        MainViewMessage::CredentialsLoaded,
                    )
                } else {
                    self.filter_credentials();
                    Task::none()
                }
            }

            MainViewMessage::ClearSearch => {
                self.search_query.clear();
                self.filter_credentials();
                Task::none()
            }

            MainViewMessage::AddCredential => {
                // TODO: Show add credential dialog
                Task::none()
            }

            MainViewMessage::EditCredential(id) => {
                self.selected_credential = Some(id);
                // TODO: Show edit credential dialog
                Task::none()
            }

            MainViewMessage::CredentialClicked(id) => {
                self.selected_credential = Some(id);
                // TODO: Show edit credential dialog
                Task::none()
            }

            MainViewMessage::DeleteCredential(_id) => {
                // TODO: Show confirmation dialog and delete
                Task::none()
            }

            MainViewMessage::RefreshCredentials => {
                self.is_loading = true;
                Task::perform(
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
                        self.filter_credentials(); // Update filtered credentials after loading
                        if let Some(sid) = session_id {
                            self.session_id = Some(sid);
                        }
                        self.is_authenticated = authenticated;
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
                        // Error handling is now done at the application level
                        self.is_authenticated = false;
                    }
                }
                Task::none()
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
                            Task::none()
                        } else {
                            // Auto-refresh credentials after successful operation
                            Task::perform(
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
                        // Error handling is now done at the application level
                        Task::none()
                    }
                }
            }

            MainViewMessage::ShowError(_error) => {
                // Error is now handled at the application level via toast system
                Task::none()
            }

            // Error demonstration handlers - these are for testing
            MainViewMessage::TriggerConnectionError => {
                // These would be handled at the application level
                Task::none()
            }

            MainViewMessage::TriggerAuthError => {
                // These would be handled at the application level
                Task::none()
            }

            MainViewMessage::TriggerValidationError => {
                // These would be handled at the application level
                Task::none()
            }

            MainViewMessage::LockDatabase => {
                // Lock the credential store
                let credential_store = crate::services::get_credential_store();
                credential_store.lock();

                // Clear local state
                self.session_id = None;
                self.is_authenticated = false;
                self.credentials.clear();

                tracing::info!("Database locked successfully");
                Task::none()
            }

            MainViewMessage::ShowSettings => {
                // This is handled at the application level in main.rs
                Task::none()
            }

            MainViewMessage::ShowAbout => {
                // TODO: Show about dialog
                Task::none()
            }

            MainViewMessage::CheckForUpdates => {
                // This is handled by the parent application
                Task::none()
            }

            MainViewMessage::CloseRepository => {
                // Close the current repository
                tracing::info!("Closing repository...");
                Task::perform(
                    Self::close_repository_async(),
                    MainViewMessage::RepositoryOperationComplete,
                )
            }

            MainViewMessage::RepositoryOperationComplete(result) => {
                match result {
                    Ok(message) => {
                        tracing::info!("Repository operation completed: {}", message);
                        // Success messages are handled by the global toast system
                        Task::none()
                    }
                    Err(error) => {
                        tracing::error!("Repository operation failed: {}", error);
                        // Error messages are handled by the global toast system
                        Task::none()
                    }
                }
            }

            MainViewMessage::SessionTimeout => {
                // This is handled by the helper method and parent application
                // Just return none as the timeout has already been processed
                Task::none()
            }

            MainViewMessage::CloseArchive => {
                // This is handled at the application level in main.rs
                Task::none()
            }
        }
    }

    /// Render the main view
    pub fn view(&self) -> Element<'_, MainViewMessage> {
        let sidebar = self.view_sidebar();
        let main_content = self.view_main_content();

        row![sidebar, main_content]
            .spacing(0)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    /// Render the left sidebar with logo and action buttons
    fn view_sidebar(&self) -> Element<'_, MainViewMessage> {
        let logo = container(
            svg(theme::ziplock_logo())
                .width(Length::Fixed(48.0))
                .height(Length::Fixed(48.0)),
        )
        .padding(utils::logo_container_padding())
        .width(Length::Fill)
        .center_x(Length::Fill);

        let add_button = container(
            button(
                svg(theme::plus_icon())
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0)),
            )
            .on_press(MainViewMessage::AddCredential)
            .padding(12)
            .style(theme::button_styles::primary()),
        )
        .width(Length::Fill)
        .center_x(Length::Fill);

        let update_button = container(btn::icon_button(
            svg(theme::refresh_icon())
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0)),
            Some(MainViewMessage::RefreshCredentials),
        ))
        .width(Length::Fill)
        .center_x(Length::Fill);

        let settings_button = container(btn::icon_button(
            svg(theme::settings_icon())
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0)),
            Some(MainViewMessage::ShowSettings),
        ))
        .width(Length::Fill)
        .center_x(Length::Fill);

        let close_button = container(
            button(
                svg(theme::lock_icon())
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0)),
            )
            .on_press(MainViewMessage::CloseArchive)
            .padding(12)
            .style(theme::button_styles::destructive()),
        )
        .width(Length::Fill)
        .center_x(Length::Fill);

        let sidebar_content = column![
            logo,
            Space::with_height(Length::Fixed(30.0)),
            add_button,
            Space::with_height(Length::Fill),
            update_button,
            Space::with_height(Length::Fixed(10.0)),
            settings_button,
            Space::with_height(Length::Fixed(10.0)),
            close_button,
        ]
        .spacing(0)
        .padding(20)
        .width(Length::Fixed(120.0))
        .height(Length::Fill);

        container(sidebar_content)
            .width(Length::Fixed(120.0))
            .height(Length::Fill)
            .into()
    }

    /// Render the main content area with search and credentials
    fn view_main_content(&self) -> Element<'_, MainViewMessage> {
        let search_bar = self.view_search_bar();

        let mut content_column = column![
            Space::with_height(Length::Fixed(20.0)),
            search_bar,
            Space::with_height(Length::Fixed(utils::standard_spacing().into())),
        ];

        let credential_list = self.view_credential_list();
        content_column = content_column.push(credential_list);

        let main_content = content_column
            .padding(utils::main_content_padding())
            .spacing(10);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Render the search bar
    fn view_search_bar(&self) -> Element<'_, MainViewMessage> {
        row![
            text_input("Search credentials...", &self.search_query)
                .on_input(MainViewMessage::SearchChanged)
                .on_submit(MainViewMessage::SearchSubmitted)
                .width(Length::FillPortion(3))
                .padding(utils::search_bar_padding())
                .size(crate::ui::theme::utils::typography::text_input_size())
                .style(theme::text_input_styles::standard()),
            Space::with_width(Length::Fixed(10.0)),
            if !self.search_query.is_empty() {
                btn::presets::clear_button(Some(MainViewMessage::ClearSearch))
            } else {
                btn::presets::clear_button(None)
            }
        ]
        .align_y(Alignment::Center)
        .into()
    }

    /// Render the list of credentials
    fn view_credential_list(&self) -> Element<'_, MainViewMessage> {
        if self.is_loading {
            return column![
                Space::with_height(Length::Fixed(50.0)),
                text("Loading credentials...")
                    .size(crate::ui::theme::utils::typography::medium_text_size()),
                Space::with_height(Length::Fixed(20.0)),
                text("Please wait while we fetch your credentials from the backend...")
                    .size(crate::ui::theme::utils::typography::small_text_size()),
            ]
            .align_x(Alignment::Center)
            .into();
        }

        if self.filtered_credentials.is_empty() {
            return if self.search_query.is_empty() {
                if self.is_authenticated {
                    // No credentials and authenticated - show friendly empty state
                    container(
                        column![
                            text("No credentials yet!")
                                .size(crate::ui::theme::utils::typography::header_text_size()),
                            Space::with_height(Length::Fixed(10.0)),
                            text("Let's add your first credential to get started")
                                .size(crate::ui::theme::utils::typography::medium_text_size()),
                            Space::with_height(Length::Fixed(30.0)),
                            btn::primary_button(
                                "Add Your First Credential",
                                Some(MainViewMessage::AddCredential),
                            ),
                            Space::with_height(Length::Fixed(20.0)),
                            text("or click 'Refresh' to reload from backend")
                                .size(crate::ui::theme::utils::typography::small_text_size()),
                        ]
                        .align_x(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                } else {
                    // Not authenticated - show locked state
                    column![
                        text("Database is locked")
                            .size(crate::ui::theme::utils::typography::medium_text_size()),
                        text("Please unlock it first to view credentials.")
                            .size(crate::ui::theme::utils::typography::normal_text_size()),
                    ]
                    .align_x(Alignment::Center)
                    .into()
                }
            } else {
                // Search returned no results
                container(
                    column![
                        Space::with_height(Length::Fixed(50.0)),
                        text("No credentials found")
                            .size(crate::ui::theme::utils::typography::medium_text_size()),
                        text("Try adjusting your search terms")
                            .size(crate::ui::theme::utils::typography::normal_text_size()),
                    ]
                    .align_x(Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            };
        }

        let credential_items: Vec<Element<MainViewMessage>> = self
            .filtered_credentials
            .iter()
            .map(|credential| self.view_credential_item(credential))
            .collect();

        scrollable(
            column(credential_items)
                .spacing(10)
                .padding(utils::list_padding()),
        )
        .height(Length::Fill)
        .into()
    }

    /// Render a single credential item
    fn view_credential_item<'a>(
        &'a self,
        credential: &'a CredentialItem,
    ) -> Element<'a, MainViewMessage> {
        let is_selected = self.selected_credential.as_ref() == Some(&credential.id);

        // Visual feedback for selected state (currently unused but kept for future styling)
        let _is_selected = is_selected;

        button(
            row![
                svg(
                    crate::ui::theme::utils::typography::get_credential_type_icon(
                        &credential.credential_type
                    )
                )
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0)),
                {
                    let mut content_elements = vec![text(&credential.title)
                        .size(crate::ui::theme::utils::typography::medium_text_size())
                        .into()];

                    if let Some(url) = &credential.url {
                        content_elements.push(
                            text(url)
                                .size(crate::ui::theme::utils::typography::small_text_size())
                                .into(),
                        );
                    }

                    container(
                        column(content_elements).spacing(if credential.url.is_some() {
                            2
                        } else {
                            0
                        }),
                    )
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Left)
                }
            ]
            .spacing(12)
            .padding(15)
            .align_y(Alignment::Center),
        )
        .on_press(MainViewMessage::EditCredential(credential.id.clone()))
        .width(Length::Fill)
        .style(theme::button_styles::credential_list_item())
        .into()
    }

    /// Async function to load credentials from backend
    async fn load_credentials_async(
        session_id: Option<String>,
    ) -> Result<(Vec<CredentialItem>, Option<String>, bool), String> {
        // Use the new repository service
        let repository_service = get_repository_service();

        // Check if repository is open
        if !repository_service.is_open().await {
            return Ok((Vec::new(), session_id, false));
        }

        // Use the provided session ID or generate a new one for compatibility
        let current_session_id = session_id.unwrap_or_else(|| {
            // Generate a simple session ID without uuid dependency
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            format!("session_{}", timestamp)
        });

        // Get credentials from the repository service
        match repository_service.list_credentials().await {
            Ok(credential_records) => {
                let credentials: Vec<CredentialItem> = credential_records
                    .into_iter()
                    .map(|cred| {
                        // Extract username from fields if available
                        let username = cred
                            .fields
                            .iter()
                            .find(|(_, field)| {
                                field.field_type == ziplock_shared::models::FieldType::Username
                            })
                            .map(|(_, field)| field.value.clone())
                            .unwrap_or_else(|| "No username".to_string());

                        // Extract URL from fields if available
                        let url = cred
                            .fields
                            .iter()
                            .find(|(_, field)| {
                                field.field_type == ziplock_shared::models::FieldType::Url
                            })
                            .map(|(_, field)| field.value.clone());

                        CredentialItem {
                            id: cred.id.clone(),
                            title: cred.title,
                            username,
                            url,
                            last_modified: cred.updated_at.to_string(),
                            credential_type: cred.credential_type,
                        }
                    })
                    .collect();

                tracing::info!(
                    "Successfully loaded {} credentials from repository service",
                    credentials.len()
                );

                Ok((credentials, Some(current_session_id), true))
            }
            Err(e) => {
                tracing::error!("Failed to load credentials from repository service: {}", e);
                Err(format!("Failed to load credentials: {}", e))
            }
        }
    }

    /// Async function to lock the database
    /// Handle potential session timeout errors
    fn handle_potential_session_timeout(
        &mut self,
        error_msg: &str,
    ) -> Option<Task<MainViewMessage>> {
        if error_msg.contains("session")
            && (error_msg.contains("timeout") || error_msg.contains("expired"))
        {
            // Clear local session state immediately
            self.session_id = None;
            self.is_authenticated = false;
            self.credentials.clear();
            // Session timeout handling is now done at the application level
            // Return command to trigger session timeout handling
            Some(Task::perform(async {}, |_| MainViewMessage::SessionTimeout))
        } else {
            None
        }
    }

    // Error handling methods removed since we're using global toast system

    /// Filter credentials based on current search query
    fn filter_credentials(&mut self) {
        if self.search_query.trim().is_empty() {
            self.filtered_credentials = self.credentials.clone();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_credentials = self
                .credentials
                .iter()
                .filter(|cred| {
                    cred.title.to_lowercase().contains(&query_lower)
                        || cred.username.to_lowercase().contains(&query_lower)
                        || cred
                            .url
                            .as_ref()
                            .map_or(false, |url| url.to_lowercase().contains(&query_lower))
                })
                .cloned()
                .collect();
        }
    }

    /// Async function to search credentials using repository service
    async fn search_credentials_async(
        query: String,
    ) -> Result<(Vec<CredentialItem>, Option<String>, bool), String> {
        let repository_service = get_repository_service();

        // Check if repository is open
        if !repository_service.is_open().await {
            return Ok((Vec::new(), None, false));
        }

        // Search credentials using repository service
        match repository_service.search_credentials(query).await {
            Ok(credential_records) => {
                let credentials: Vec<CredentialItem> = credential_records
                    .into_iter()
                    .map(|cred| {
                        // Extract username from fields if available
                        let username = cred
                            .fields
                            .iter()
                            .find(|(_, field)| {
                                field.field_type == ziplock_shared::models::FieldType::Username
                            })
                            .map(|(_, field)| field.value.clone())
                            .unwrap_or_else(|| "No username".to_string());

                        // Extract URL from fields if available
                        let url = cred
                            .fields
                            .iter()
                            .find(|(_, field)| {
                                field.field_type == ziplock_shared::models::FieldType::Url
                            })
                            .map(|(_, field)| field.value.clone());

                        CredentialItem {
                            id: cred.id.clone(),
                            title: cred.title,
                            username,
                            url,
                            last_modified: cred.updated_at.to_string(),
                            credential_type: cred.credential_type,
                        }
                    })
                    .collect();

                tracing::info!(
                    "Found {} credentials matching search query",
                    credentials.len()
                );

                Ok((credentials, None, true))
            }
            Err(e) => {
                tracing::error!("Failed to search credentials: {}", e);
                Err(format!("Failed to search credentials: {}", e))
            }
        }
    }

    /// Async function to close the repository
    async fn close_repository_async() -> Result<String, String> {
        let repository_service = get_repository_service();

        match repository_service.close_repository().await {
            Ok(()) => {
                tracing::info!("Repository closed successfully");
                Ok("Repository closed successfully".to_string())
            }
            Err(e) => {
                tracing::error!("Failed to close repository: {}", e);
                Err(format!("Failed to close repository: {}", e))
            }
        }
    }
}
