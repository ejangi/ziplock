//! ZipLock Linux App
//!
//! This is the Linux desktop app for ZipLock, built with the Iced GUI framework.
//! It provides a native Linux interface for managing encrypted password archives.

use iced::{widget::svg, Application, Command, Element, Settings, Theme};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;

mod ui;

use ui::components::toast::{ToastManager, ToastPosition};
use ui::theme::alerts::AlertMessage;

use ui::{create_ziplock_theme, theme};

use config::{ConfigManager, RepositoryInfo};
use ui::views::main::{MainView, MainViewMessage};
use ui::views::{
    AddCredentialMessage, AddCredentialView, EditCredentialMessage, EditCredentialView,
    OpenRepositoryMessage, OpenRepositoryView, RepositoryWizard, WizardMessage,
};
use ziplock_shared::ZipLockClient;

/// Main application messages
#[derive(Debug, Clone)]
pub enum Message {
    // Configuration
    ConfigLoaded(String), // Just store success/error message
    ConfigReady,
    ConfigSaved,

    // Repository detection
    RepositoriesDetected(Vec<RepositoryInfo>),
    RepositoryValidated(Result<RepositoryInfo, String>),

    // Wizard messages
    Wizard(WizardMessage),
    ShowWizard,
    HideWizard,

    // Open repository messages
    OpenRepository(OpenRepositoryMessage),
    ShowOpenRepository,
    HideOpenRepository,

    // Main application
    CreateRepository,
    BackendConnected(Result<(), String>),

    // Main view messages
    MainView(MainViewMessage),

    // Add credential messages
    AddCredential(AddCredentialMessage),
    ShowAddCredential,
    HideAddCredential,

    // Edit credential messages
    EditCredential(EditCredentialMessage),
    ShowEditCredential(String),
    HideEditCredential,

    // Alert management
    ShowAlert(AlertMessage),
    DismissAlert,

    // Toast management
    ShowToast(AlertMessage),
    DismissToast(usize),
    UpdateToasts,

    // Operation results from views
    OperationResult(Result<String, String>),

    // Session management
    SessionTimeout,

    // General
    Quit,
    QuittingWithLogout,
    Error(String),
}

/// Application state
#[derive(Debug)]
pub enum AppState {
    Loading,
    DetectingRepositories,
    RepositorySelection(Vec<RepositoryInfo>),
    WizardRequired,
    WizardActive(RepositoryWizard),
    OpenRepositoryActive(OpenRepositoryView),
    AddCredentialActive(AddCredentialView),
    EditCredentialActive(EditCredentialView),
    MainInterface(MainView),
    Error(String),
}

/// Main application structure
pub struct ZipLockApp {
    state: AppState,
    config_manager: Option<ConfigManager>,
    #[allow(dead_code)] // Future FFI client functionality
    ffi_client: Option<ZipLockClient>,
    theme: Theme,
    current_alert: Option<AlertMessage>,
    session_id: Option<String>,
    toast_manager: ToastManager,
}

impl Application for ZipLockApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        info!("Initializing ZipLock Linux app");

        let app = Self {
            state: AppState::Loading,
            config_manager: None,
            ffi_client: None,
            theme: create_ziplock_theme(),
            current_alert: None,
            session_id: None,
            toast_manager: ToastManager::with_position(ToastPosition::BottomRight),
        };

        let load_config_command =
            Command::perform(Self::load_config_async(), Message::ConfigLoaded);

        (app, load_config_command)
    }

    fn title(&self) -> String {
        match &self.state {
            AppState::Loading => "ZipLock - Loading...".to_string(),
            AppState::DetectingRepositories => "ZipLock - Detecting Repositories...".to_string(),
            AppState::RepositorySelection(_) => "ZipLock - Select Repository".to_string(),
            AppState::WizardRequired | AppState::WizardActive(_) => {
                "ZipLock - Setup Wizard".to_string()
            }
            AppState::OpenRepositoryActive(_) => "ZipLock - Open Repository".to_string(),
            AppState::AddCredentialActive(_) => "ZipLock - Add Credential".to_string(),
            AppState::EditCredentialActive(_) => "ZipLock - Edit Credential".to_string(),
            AppState::MainInterface(_) => "ZipLock Password Manager".to_string(),
            AppState::Error(_) => "ZipLock - Error".to_string(),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ConfigLoaded(error_message) => {
                if error_message.is_empty() {
                    Command::perform(async {}, |_| Message::ConfigReady)
                } else {
                    error!("Failed to load configuration: {}", error_message);
                    self.state = AppState::Error(format!("Configuration error: {}", error_message));
                    Command::none()
                }
            }

            Message::ConfigReady => {
                if let Ok(config_manager) = ConfigManager::new() {
                    info!("Configuration loaded successfully");

                    // Check if we should show the wizard immediately
                    if config_manager.should_show_wizard() {
                        debug!("No repositories found, showing setup wizard");
                        self.state = AppState::WizardRequired;
                        self.config_manager = Some(config_manager);
                        return Command::none();
                    }

                    // Start repository detection
                    debug!("Starting repository detection");
                    self.state = AppState::DetectingRepositories;
                    let repositories = config_manager.detect_all_accessible_repositories();
                    self.config_manager = Some(config_manager);

                    Command::perform(async move { repositories }, Message::RepositoriesDetected)
                } else {
                    self.state = AppState::Error("Failed to initialize configuration".to_string());
                    Command::none()
                }
            }

            Message::RepositoriesDetected(repositories) => {
                info!("Detected {} repositories", repositories.len());
                // Store repositories for selection view

                if repositories.is_empty() {
                    debug!("No repositories detected, showing wizard");
                    self.state = AppState::WizardRequired;
                } else if repositories.len() == 1 {
                    // Auto-select single repository and show open dialog
                    debug!("Single repository found, showing open dialog");
                    let repo = &repositories[0];
                    let open_view = OpenRepositoryView::with_repository(repo.path.clone());
                    self.state = AppState::OpenRepositoryActive(open_view);
                } else {
                    // Show repository selection
                    debug!("Multiple repositories found, showing selection");
                    self.state = AppState::RepositorySelection(repositories);
                }
                Command::none()
            }

            Message::RepositoryValidated(result) => {
                match result {
                    Ok(repo_info) => {
                        info!("Repository validated: {:?}", repo_info.path);
                        // Repository is valid, show open dialog
                        let open_view = OpenRepositoryView::with_repository(repo_info.path);
                        self.state = AppState::OpenRepositoryActive(open_view);
                    }
                    Err(error) => {
                        warn!("Repository validation failed: {}", error);
                        self.toast_manager
                            .warning(format!("Repository Validation Failed: {}", error));
                        self.state = AppState::WizardRequired;
                    }
                }
                Command::none()
            }

            Message::ConfigSaved => {
                debug!("Configuration saved");
                Command::none()
            }

            Message::ShowWizard => {
                debug!("Starting repository setup wizard");
                let wizard = RepositoryWizard::new();
                self.state = AppState::WizardActive(wizard);
                Command::none()
            }

            Message::HideWizard => {
                debug!("Hiding wizard, returning to main interface");
                self.state = AppState::MainInterface(MainView::new());
                // Try to connect to backend after wizard completion
                Command::perform(Self::connect_backend_async(), Message::BackendConnected)
            }

            Message::Wizard(wizard_msg) => {
                if let AppState::WizardActive(wizard) = &mut self.state {
                    let command = wizard.update(wizard_msg.clone()).map(Message::Wizard);

                    // Check if wizard was cancelled
                    if wizard.is_cancelled() {
                        debug!("Wizard cancelled, returning to initial choice screen");
                        self.state = AppState::WizardRequired;
                        return command;
                    }

                    // Check if wizard completed successfully
                    if wizard.is_complete() {
                        // Save repository path to config
                        if let (Some(repo_path), Some(config_manager)) =
                            (wizard.repository_path(), &mut self.config_manager)
                        {
                            match config_manager.set_repository_path(repo_path) {
                                Ok(()) => {
                                    info!("Repository path saved to configuration");
                                    return Command::batch([
                                        command,
                                        Command::perform(async {}, |_| Message::HideWizard),
                                    ]);
                                }
                                Err(e) => {
                                    error!("Failed to save repository path: {}", e);
                                    return Command::batch([
                                        command,
                                        Command::perform(
                                            async move { e.to_string() },
                                            Message::Error,
                                        ),
                                    ]);
                                }
                            }
                        }
                    }

                    return command;
                }
                Command::none()
            }

            Message::ShowOpenRepository => {
                debug!("Starting open repository dialog");
                let open_view = OpenRepositoryView::new();
                self.state = AppState::OpenRepositoryActive(open_view);
                Command::none()
            }

            Message::HideOpenRepository => {
                debug!("Hiding open repository dialog, returning to main interface");

                // Save repository path before changing state
                let mut save_repo_path = None;
                if let AppState::OpenRepositoryActive(ref open_view) = &self.state {
                    if let Some(repo_path) = open_view.repository_path() {
                        save_repo_path = Some(repo_path.clone());
                    }
                }

                self.state = AppState::MainInterface(MainView::new());

                // Save to config if we have a path
                if let Some(repo_path) = save_repo_path {
                    if let Some(config_manager) = &mut self.config_manager {
                        match config_manager.set_repository_path(repo_path) {
                            Ok(()) => {
                                info!("Repository path saved to configuration");
                            }
                            Err(e) => {
                                error!("Failed to save repository path: {}", e);
                            }
                        }
                    }
                }

                Command::none()
            }

            Message::OpenRepository(open_msg) => {
                if let AppState::OpenRepositoryActive(open_view) = &mut self.state {
                    let command = open_view
                        .update(open_msg.clone())
                        .map(Message::OpenRepository);

                    // Check if opening completed successfully or was cancelled
                    if open_view.is_complete() {
                        // Capture session ID and create MainView
                        if let Some(session_id) = open_view.session_id() {
                            self.session_id = Some(session_id.clone());
                            let mut main_view = MainView::new();
                            main_view.set_session_id(Some(session_id.clone()));
                            self.state = AppState::MainInterface(main_view);
                            // Trigger initial refresh to update authentication state
                            return Command::batch([
                                command,
                                Command::perform(async {}, |_| {
                                    Message::MainView(MainViewMessage::RefreshCredentials)
                                }),
                            ]);
                        } else {
                            self.state = AppState::MainInterface(MainView::new());
                        }
                        return command;
                    }

                    // Check if user cancelled - return to welcome screen
                    if open_view.is_cancelled() {
                        self.state = AppState::WizardRequired;
                        return command;
                    }

                    return command;
                }
                Command::none()
            }

            Message::CreateRepository => {
                debug!("User requested to create new repository");
                Command::perform(async {}, |_| Message::ShowWizard)
            }

            Message::BackendConnected(result) => {
                match result {
                    Ok(()) => {
                        info!("Successfully connected to backend");
                        // TODO: Load repository if configured
                    }
                    Err(error) => {
                        warn!("Failed to connect to backend: {}", error);
                        // Continue anyway, backend might not be needed for some operations
                    }
                }
                Command::none()
            }

            Message::Error(error) => {
                error!("Application error: {}", error);
                self.state = AppState::Error(error);
                Command::none()
            }

            Message::ShowAlert(alert) => {
                self.current_alert = Some(alert);
                Command::none()
            }

            Message::DismissAlert => {
                self.current_alert = None;
                Command::none()
            }

            Message::ShowToast(alert) => {
                self.toast_manager.add_toast(alert);
                Command::none()
            }

            Message::DismissToast(toast_id) => {
                self.toast_manager.remove_toast(toast_id);
                Command::none()
            }

            Message::UpdateToasts => {
                self.toast_manager.remove_expired_toasts();
                Command::none()
            }

            Message::OperationResult(result) => {
                match result {
                    Ok(success_msg) => {
                        self.toast_manager.success(success_msg);
                    }
                    Err(error_msg) => {
                        // Check if this is a session timeout error
                        if ziplock_shared::ZipLockClient::is_session_timeout_error(&error_msg) {
                            return Command::perform(async {}, |_| Message::SessionTimeout);
                        }
                        self.toast_manager.error(error_msg);
                    }
                }
                Command::none()
            }

            Message::MainView(main_msg) => {
                if let AppState::MainInterface(main_view) = &mut self.state {
                    match main_msg {
                        MainViewMessage::ShowError(error) => {
                            // Check if this is a session timeout error
                            if ziplock_shared::ZipLockClient::is_session_timeout_error(&error) {
                                return Command::perform(async {}, |_| Message::SessionTimeout);
                            }
                            self.toast_manager.error(error);
                            Command::none()
                        }
                        MainViewMessage::SessionTimeout => {
                            // Forward session timeout to main application handler
                            Command::perform(async {}, |_| Message::SessionTimeout)
                        }
                        MainViewMessage::AddCredential => {
                            // Show add credential view
                            Command::perform(async {}, |_| Message::ShowAddCredential)
                        }
                        MainViewMessage::EditCredential(credential_id) => {
                            // Show edit credential view
                            Command::perform(
                                async move { credential_id },
                                Message::ShowEditCredential,
                            )
                        }
                        MainViewMessage::TriggerConnectionError => {
                            self.toast_manager.ipc_error(
                                "Unable to connect to the ZipLock backend service. Please ensure the daemon is running."
                            );
                            Command::none()
                        }
                        MainViewMessage::TriggerAuthError => {
                            self.toast_manager.ipc_error(
                                "Authentication failed. Please check your passphrase and try again."
                            );
                            Command::none()
                        }
                        MainViewMessage::TriggerValidationError => {
                            self.toast_manager.error(
                                "Invalid data provided. Please check your input and try again.",
                            );
                            Command::none()
                        }
                        MainViewMessage::OperationCompleted(result) => {
                            // Forward operation results to main app for toast handling
                            Command::perform(async move { result }, Message::OperationResult)
                        }
                        _ => main_view.update(main_msg).map(Message::MainView),
                    }
                } else {
                    Command::none()
                }
            }

            Message::ShowAddCredential => {
                debug!("Showing add credential view");
                let add_view = AddCredentialView::with_session(self.session_id.clone());
                self.state = AppState::AddCredentialActive(add_view);
                Command::none()
            }

            Message::HideAddCredential => {
                debug!("Hiding add credential view, returning to main interface");
                if let Some(session_id) = &self.session_id {
                    let mut main_view = MainView::new();
                    main_view.set_session_id(Some(session_id.clone()));
                    self.state = AppState::MainInterface(main_view);
                    // Trigger refresh to reload credentials
                    return Command::perform(async {}, |_| {
                        Message::MainView(MainViewMessage::RefreshCredentials)
                    });
                } else {
                    self.state = AppState::MainInterface(MainView::new());
                }
                Command::none()
            }

            Message::AddCredential(add_msg) => {
                if let AppState::AddCredentialActive(add_view) = &mut self.state {
                    match &add_msg {
                        AddCredentialMessage::Cancel => {
                            return Command::perform(async {}, |_| Message::HideAddCredential);
                        }
                        AddCredentialMessage::ShowError(error) => {
                            self.toast_manager.error(error.clone());
                            let command = add_view.update(add_msg).map(Message::AddCredential);
                            return command;
                        }
                        AddCredentialMessage::ShowSuccess(success) => {
                            self.toast_manager.success(success.clone());
                            let command = add_view.update(add_msg).map(Message::AddCredential);
                            return Command::batch([
                                command,
                                Command::perform(async {}, |_| Message::HideAddCredential),
                            ]);
                        }
                        AddCredentialMessage::ShowValidationError(error) => {
                            self.toast_manager.warning(error.clone());
                            let command = add_view.update(add_msg).map(Message::AddCredential);
                            return command;
                        }
                        _ => {
                            let command = add_view.update(add_msg).map(Message::AddCredential);

                            // Check if add credential completed
                            if add_view.is_complete() {
                                return Command::batch([
                                    command,
                                    Command::perform(async {}, |_| Message::HideAddCredential),
                                ]);
                            }

                            return command;
                        }
                    }
                }
                Command::none()
            }

            Message::ShowEditCredential(credential_id) => {
                debug!("Showing edit credential view for ID: {}", credential_id);
                let edit_view =
                    EditCredentialView::with_session(credential_id, self.session_id.clone());
                self.state = AppState::EditCredentialActive(edit_view);
                // Load the credential data
                Command::perform(async {}, |_| {
                    Message::EditCredential(EditCredentialMessage::LoadCredential)
                })
            }

            Message::HideEditCredential => {
                debug!("Hiding edit credential view, returning to main interface");
                if let Some(session_id) = &self.session_id {
                    let mut main_view = MainView::new();
                    main_view.set_session_id(Some(session_id.clone()));
                    self.state = AppState::MainInterface(main_view);
                    // Trigger refresh to reload credentials
                    return Command::perform(async {}, |_| {
                        Message::MainView(MainViewMessage::RefreshCredentials)
                    });
                } else {
                    self.state = AppState::MainInterface(MainView::new());
                }
                Command::none()
            }

            Message::EditCredential(edit_msg) => {
                if let AppState::EditCredentialActive(edit_view) = &mut self.state {
                    match &edit_msg {
                        EditCredentialMessage::Cancel => {
                            return Command::perform(async {}, |_| Message::HideEditCredential);
                        }
                        EditCredentialMessage::ShowError(error) => {
                            self.toast_manager.error(error.clone());
                            let command = edit_view.update(edit_msg).map(Message::EditCredential);
                            return command;
                        }
                        EditCredentialMessage::ShowSuccess(success) => {
                            self.toast_manager.success(success.clone());
                            let command = edit_view.update(edit_msg).map(Message::EditCredential);
                            return Command::batch([
                                command,
                                Command::perform(async {}, |_| Message::HideEditCredential),
                            ]);
                        }
                        EditCredentialMessage::ShowValidationError(error) => {
                            self.toast_manager.warning(error.clone());
                            let command = edit_view.update(edit_msg).map(Message::EditCredential);
                            return command;
                        }
                        _ => {
                            let command = edit_view.update(edit_msg).map(Message::EditCredential);

                            // Check if edit credential completed
                            if edit_view.is_complete() {
                                return Command::batch([
                                    command,
                                    Command::perform(async {}, |_| Message::HideEditCredential),
                                ]);
                            }

                            return command;
                        }
                    }
                }
                Command::none()
            }

            Message::SessionTimeout => {
                info!("Session timeout detected, redirecting to login");
                // Clear session state
                self.session_id = None;
                // Show repository selection or wizard based on configuration
                if let Some(config_manager) = &self.config_manager {
                    if config_manager.should_show_wizard() {
                        self.state = AppState::WizardRequired;
                    } else {
                        // Try to detect repositories again
                        if let Some(config_manager) = &self.config_manager {
                            let repositories = config_manager.detect_all_accessible_repositories();
                            return Command::perform(
                                async move { repositories },
                                Message::RepositoriesDetected,
                            );
                        } else {
                            self.state = AppState::WizardRequired;
                        }
                    }
                } else {
                    self.state = AppState::WizardRequired;
                }
                // Show timeout notification
                self.toast_manager
                    .warning("Your session has expired. Please unlock your repository again.");
                Command::none()
            }

            Message::Quit => {
                info!("Application quit requested");
                // If we have an active session, try to logout first
                if self.session_id.is_some() {
                    info!("Active session detected, logging out before quit");
                    Command::perform(Self::logout_and_quit_async(self.session_id.clone()), |_| {
                        Message::QuittingWithLogout
                    })
                } else {
                    // No active session, quit immediately
                    std::process::exit(0);
                }
            }

            Message::QuittingWithLogout => {
                info!("Logout complete, exiting application");
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let main_content = match &self.state {
            AppState::Loading => self.view_loading(),
            AppState::DetectingRepositories => self.view_detecting_repositories(),
            AppState::RepositorySelection(repositories) => {
                self.view_repository_selection(repositories)
            }
            AppState::WizardRequired => self.view_wizard_required(),
            AppState::WizardActive(wizard) => wizard.view().map(Message::Wizard),
            AppState::OpenRepositoryActive(open_view) => {
                open_view.view().map(Message::OpenRepository)
            }
            AppState::AddCredentialActive(add_view) => add_view.view().map(Message::AddCredential),
            AppState::EditCredentialActive(edit_view) => {
                edit_view.view().map(Message::EditCredential)
            }
            AppState::MainInterface(main_view) => main_view.view().map(Message::MainView),
            AppState::Error(error) => self.view_error(error),
        };

        self.wrap_with_toasts(main_content)
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        use iced::time;

        let close_subscription = iced::event::listen_with(|event, _status| match event {
            iced::Event::Window(_, iced::window::Event::CloseRequested) => Some(Message::Quit),
            _ => None,
        });

        let toast_subscription = if self.toast_manager.has_toasts() {
            time::every(std::time::Duration::from_millis(100)).map(|_| Message::UpdateToasts)
        } else {
            iced::Subscription::none()
        };

        let view_subscription = match &self.state {
            AppState::AddCredentialActive(view) => view.subscription().map(Message::AddCredential),
            AppState::EditCredentialActive(view) => {
                view.subscription().map(Message::EditCredential)
            }
            _ => iced::Subscription::none(),
        };

        iced::Subscription::batch([close_subscription, toast_subscription, view_subscription])
    }
}

impl ZipLockApp {
    /// Wraps any view content with toast overlay and optional alert display
    fn wrap_with_toasts<'a>(&'a self, content: Element<'a, Message>) -> Element<'a, Message> {
        use iced::widget::{column, Space};
        use iced::Length;
        use ui::components::toast::render_toast_overlay;
        use ui::theme::alerts;

        // First wrap with toasts
        let content_with_toasts =
            render_toast_overlay(&self.toast_manager, content, Message::DismissToast);

        // Then wrap with alert if present (for backwards compatibility)
        if let Some(alert) = &self.current_alert {
            column![
                alerts::render_alert(alert, Some(Message::DismissAlert)),
                Space::with_height(Length::Fixed(10.0)),
                content_with_toasts,
            ]
            .into()
        } else {
            content_with_toasts
        }
    }
    /// View loading screen
    fn view_loading(&self) -> Element<'_, Message> {
        use iced::widget::{container, text, Space};
        use iced::{Alignment, Length};

        container(
            iced::widget::column![
                Space::with_height(Length::Fill),
                text("Loading ZipLock...")
                    .size(24)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text("Initializing configuration...")
                    .size(14)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fill),
            ]
            .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    /// View detecting repositories screen
    fn view_detecting_repositories(&self) -> Element<'_, Message> {
        use iced::widget::{container, text, Space};
        use iced::{Alignment, Length};

        container(
            iced::widget::column![
                Space::with_height(Length::Fill),
                text("Detecting Repositories...")
                    .size(24)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text("Searching for existing password repositories...")
                    .size(14)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fill),
            ]
            .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    /// View repository selection screen
    fn view_repository_selection(&self, repositories: &[RepositoryInfo]) -> Element<'_, Message> {
        use iced::widget::{button, column, container, row, text, Space};
        use iced::{Alignment, Length};

        let mut repo_buttons = column![].spacing(10);

        for repo in repositories {
            let display_name = &repo.display_name;
            let path_text = if let Some(relative) =
                ziplock_shared::config::paths::get_relative_to_home(&repo.path)
            {
                format!("~/{}", relative.display())
            } else {
                repo.path.display().to_string()
            };

            let size_text = if repo.size < 1024 {
                format!("{} bytes", repo.size)
            } else if repo.size < 1024 * 1024 {
                format!("{:.1} KB", repo.size as f64 / 1024.0)
            } else {
                format!("{:.1} MB", repo.size as f64 / (1024.0 * 1024.0))
            };

            let repo_button = button(
                column![
                    text(display_name).size(16),
                    text(&path_text).size(12),
                    text(&size_text).size(10),
                ]
                .spacing(2),
            )
            .width(Length::Fill)
            .padding(theme::utils::repository_button_padding())
            .on_press(Message::OpenRepository(
                OpenRepositoryMessage::SelectSpecificFile(repo.path.clone()),
            ));

            repo_buttons = repo_buttons.push(repo_button);
        }

        container(
            column![
                Space::with_height(Length::Fixed(40.0)),
                svg(theme::ziplock_logo())
                    .width(iced::Length::Fixed(64.0))
                    .height(iced::Length::Fixed(64.0)),
                Space::with_height(Length::Fixed(20.0)),
                text("Select Repository")
                    .size(28)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(10.0)),
                text(format!(
                    "Found {} password repositories",
                    repositories.len()
                ))
                .size(14)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(30.0)),
                container(repo_buttons).width(Length::Fixed(400.0)),
                Space::with_height(Length::Fixed(30.0)),
                row![
                    button("Create New Repository")
                        .on_press(Message::ShowWizard)
                        .padding(theme::utils::standard_button_padding()),
                    Space::with_width(Length::Fixed(20.0)),
                    button("Browse for Repository...")
                        .on_press(Message::ShowOpenRepository)
                        .padding(theme::utils::standard_button_padding()),
                ]
                .spacing(10),
                Space::with_height(Length::Fill),
            ]
            .align_items(Alignment::Center)
            .max_width(500),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    /// View wizard required screen
    fn view_wizard_required(&self) -> Element<'_, Message> {
        use iced::widget::{button, column, container, text, Space};
        use iced::{Alignment, Length};

        container(
            column![
                Space::with_height(Length::Fill),
                svg(theme::ziplock_logo())
                    .width(iced::Length::Fixed(80.0))
                    .height(iced::Length::Fixed(80.0)),
                Space::with_height(Length::Fixed(20.0)),
                text("Welcome to ZipLock!")
                    .size(32)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(30.0)),
                text("Get started by setting up your first password repository.")
                    .size(16)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(40.0)),
                button("Setup Repository")
                    .on_press(Message::ShowWizard)
                    .padding(theme::utils::setup_button_padding()),
                Space::with_height(Length::Fixed(20.0)),
                button("Open Existing Repository")
                    .on_press(Message::ShowOpenRepository)
                    .padding(theme::utils::standard_button_padding()),
                Space::with_height(Length::Fill),
            ]
            .align_items(Alignment::Center)
            .max_width(500),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    /// View error screen
    fn view_error(&self, error: &str) -> Element<'_, Message> {
        use iced::widget::{button, column, container, text, Space};
        use iced::{Alignment, Length};

        container(
            column![
                Space::with_height(Length::Fill),
                svg(theme::ziplock_logo())
                    .width(iced::Length::Fixed(64.0))
                    .height(iced::Length::Fixed(64.0)),
                Space::with_height(Length::Fixed(20.0)),
                text("❌ Error")
                    .size(32)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text(error)
                    .size(14)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(30.0)),
                button("Quit")
                    .on_press(Message::Quit)
                    .padding(theme::utils::standard_button_padding()),
                Space::with_height(Length::Fill),
            ]
            .align_items(Alignment::Center)
            .max_width(500),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    /// Async function to load configuration
    async fn load_config_async() -> String {
        match ConfigManager::new() {
            Ok(_) => String::new(), // Empty string means success
            Err(e) => e.to_string(),
        }
    }

    /// Async function to connect to backend
    async fn logout_and_quit_async(session_id: Option<String>) -> Result<(), String> {
        if let Some(_sid) = session_id.clone() {
            let mut client = ziplock_shared::ZipLockClient::new().map_err(|e| e.to_string())?;

            // Try to connect and logout
            match client.connect().await {
                Ok(()) => {
                    // Try to close/lock the archive
                    match client.close_archive().await {
                        Ok(()) => info!("Successfully logged out before quit"),
                        Err(e) => warn!("Failed to logout cleanly: {}", e),
                    }
                }
                Err(e) => warn!("Could not connect to backend for logout: {}", e),
            }
        }
        Ok(())
    }

    async fn connect_backend_async() -> Result<(), String> {
        let mut client = ZipLockClient::new().map_err(|e| e.to_string())?;

        client.connect().await.map_err(|e| e.to_string())?;
        // Test the connection with a ping
        client.ping().await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_level(true),
        )
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    info!("Starting ZipLock Linux app");

    // Configure application settings
    let settings = Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1000.0, 700.0),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            position: iced::window::Position::Centered,
            ..Default::default()
        },
        fonts: vec![],
        default_font: iced::Font::DEFAULT,
        antialiasing: true,
        ..Default::default()
    };

    ZipLockApp::run(settings)
}
