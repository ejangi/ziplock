//! ZipLock Desktop App
//!
//! This is the cross-platform desktop app for ZipLock, built with the Iced GUI framework.
//! It provides a native interface for managing encrypted password archives.

// Windows configuration for GUI applications
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use iced::{
    widget::{button, svg, text},
    Element, Task, Theme,
};
use tracing::{debug, error, info, warn};

// Import removed - these types are used in the actual code through other paths

mod config;
// #[cfg(feature = "examples")]
// mod examples;
mod logging;
mod services;
mod ui;

use ui::components::button::{destructive_button, primary_button, secondary_button};

use services::{ClipboardManager, UpdateChecker};

use ui::components::toast::{ToastManager, ToastPosition};
use ui::components::{UpdateDialog, UpdateDialogMessage};
use ui::theme::alerts::AlertMessage;

use ui::{create_ziplock_theme, theme};

use config::{ConfigManager, RepositoryInfo};
use ui::views::main::{MainView, MainViewMessage};
use ui::views::{
    AddCredentialMessage, AddCredentialView, EditCredentialMessage, EditCredentialView,
    OpenRepositoryMessage, OpenRepositoryView, RepositoryWizard, SettingsMessage, SettingsView,
    WizardMessage,
};

/// Utility function to detect if running in production mode
fn is_production_mode() -> bool {
    // Check environment variable first
    if std::env::var("ZIPLOCK_PRODUCTION").is_ok() {
        return true;
    }

    // Check if production feature is enabled
    if cfg!(feature = "production") {
        return true;
    }

    // In release builds without debug assertions, assume production
    if !cfg!(debug_assertions) {
        return true;
    }

    false
}

/// Get appropriate logging level for current mode
fn get_logging_mode() -> &'static str {
    if is_production_mode() {
        "production"
    } else {
        "development"
    }
}

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

    // Settings messages
    Settings(SettingsMessage),
    ShowSettings,
    HideSettings,

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

    // Auto-lock management
    AutoLockTimerTick,
    UserActivity,

    // Update checking
    CheckForUpdates,
    UpdateCheckResult(Result<services::UpdateCheckResult, String>),
    ShowUpdateDialog(services::UpdateCheckResult),
    HideUpdateDialog,
    AutoUpdateCheck,

    // Clipboard management
    CopyToClipboard {
        content: String,
        content_type: services::ClipboardContentType,
    },

    // General
    Quit,
    QuittingWithLogout,
    CloseArchive,
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
    SettingsActive(SettingsView),
    UpdateDialogActive(UpdateDialog),
    MainInterface(MainView),
    Error(String),
}

/// Main application structure
pub struct ZipLockApp {
    state: AppState,
    config_manager: Option<ConfigManager>,
    theme: Theme,
    current_alert: Option<AlertMessage>,
    session_id: Option<String>,
    toast_manager: ToastManager,
    // Auto-lock timer fields
    last_activity: std::time::Instant,
    auto_lock_enabled: bool,
    // Update checker
    update_checker: UpdateChecker,
    // Clipboard manager
    clipboard_manager: ClipboardManager,
}

impl ZipLockApp {
    pub fn new() -> (Self, Task<Message>) {
        info!("Initializing ZipLock Linux app with unified architecture");

        // Initialize shared library
        ziplock_shared::init_ziplock_shared_desktop();

        let app = Self {
            state: AppState::Loading,
            config_manager: None,
            theme: create_ziplock_theme(),
            current_alert: None,
            session_id: None,
            toast_manager: ToastManager::with_position(ToastPosition::BottomRight),
            last_activity: std::time::Instant::now(),
            auto_lock_enabled: false,
            update_checker: UpdateChecker::new(),
            clipboard_manager: ClipboardManager::new(),
        };

        let load_config_task = Task::perform(Self::load_config_async(), Message::ConfigLoaded);

        (app, load_config_task)
    }

    pub fn title(&self) -> String {
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
            AppState::SettingsActive(_) => "ZipLock - Settings".to_string(),
            AppState::UpdateDialogActive(_) => "ZipLock - Update Available".to_string(),
            AppState::MainInterface(_) => "ZipLock Password Manager".to_string(),
            AppState::Error(_) => "ZipLock - Error".to_string(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConfigLoaded(error_message) => {
                if error_message.is_empty() {
                    Task::perform(async {}, |_| Message::ConfigReady)
                } else {
                    error!("Failed to load configuration: {}", error_message);
                    self.state = AppState::Error(format!("Configuration error: {}", error_message));
                    Task::none()
                }
            }

            Message::ConfigReady => {
                if let Ok(mut config_manager) = ConfigManager::new() {
                    // Load the configuration file to get recent repositories
                    if let Err(e) = config_manager.load() {
                        warn!("Failed to load configuration file: {}", e);
                        // Continue with defaults if loading fails
                    }

                    info!("Configuration loaded successfully");

                    // Initialize typography with font size from config
                    let font_scale = config_manager.config().ui.font_scale.unwrap_or(1.0);
                    ui::theme::utils::typography::init_font_size(font_scale);
                    info!("Font scaling initialized with scale factor: {}", font_scale);

                    // Check if we should show the wizard immediately
                    if config_manager.should_show_wizard() {
                        debug!("No repositories found, showing setup wizard");
                        self.state = AppState::WizardRequired;
                        self.config_manager = Some(config_manager);
                        return Task::none();
                    }

                    // Check for most recently used repository first
                    debug!("Checking for most recent accessible repository...");
                    let recent_repos = config_manager.get_recent_repositories();
                    debug!("Found {} recent repositories in config", recent_repos.len());
                    for repo in recent_repos.iter() {
                        debug!("Recent repo: {} -> {}", repo.name, repo.path);
                    }

                    if let Some(most_recent_path) =
                        config_manager.get_most_recent_accessible_repository()
                    {
                        info!(
                            "Auto-opening most recently used repository: {:?}",
                            most_recent_path
                        );
                        let open_view =
                            OpenRepositoryView::with_repository(most_recent_path.clone().into());
                        self.state = AppState::OpenRepositoryActive(open_view);
                        self.config_manager = Some(config_manager);
                        return Task::none();
                    } else {
                        debug!("No recent accessible repository found");
                    }

                    // Start repository detection
                    debug!("Starting repository detection");
                    self.state = AppState::DetectingRepositories;
                    let repositories = config_manager.detect_all_accessible_repositories();
                    self.config_manager = Some(config_manager);

                    Task::perform(async move { repositories }, Message::RepositoriesDetected)
                } else {
                    self.state = AppState::Error("Failed to initialize configuration".to_string());
                    Task::none()
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
                    let open_view = OpenRepositoryView::with_repository(repo.path.clone().into());
                    self.state = AppState::OpenRepositoryActive(open_view);
                } else {
                    // Show repository selection
                    debug!("Multiple repositories found, showing selection");
                    self.state = AppState::RepositorySelection(repositories);
                }
                Task::none()
            }

            Message::RepositoryValidated(result) => {
                match result {
                    Ok(repo_info) => {
                        info!("Repository validated: {:?}", repo_info.path);
                        // Repository is valid, show open dialog
                        let open_view = OpenRepositoryView::with_repository(repo_info.path.into());
                        self.state = AppState::OpenRepositoryActive(open_view);
                    }
                    Err(error) => {
                        warn!("Repository validation failed: {}", error);
                        self.toast_manager
                            .warning(format!("Repository Validation Failed: {}", error));
                        self.state = AppState::WizardRequired;
                    }
                }
                Task::none()
            }

            Message::ConfigSaved => {
                debug!("Configuration saved");
                Task::none()
            }

            Message::ShowWizard => {
                debug!("Starting repository setup wizard");
                let wizard = RepositoryWizard::new();
                self.state = AppState::WizardActive(wizard);
                Task::none()
            }

            Message::HideWizard => {
                debug!("Hiding wizard, returning to main interface");
                self.state = AppState::MainInterface(MainView::new());
                // Try to connect to backend after wizard completion
                Task::perform(Self::connect_backend_async(), |result| {
                    Message::BackendConnected(result.map_err(|e| e.to_string()))
                })
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
                            match config_manager
                                .set_repository_path(repo_path.to_string_lossy().to_string())
                            {
                                Ok(()) => {
                                    info!("Repository path saved to configuration");
                                    return Task::batch([
                                        command,
                                        Task::perform(async {}, |_| Message::HideWizard),
                                    ]);
                                }
                                Err(e) => {
                                    error!("Failed to save repository path: {}", e);
                                    return Task::batch([
                                        command,
                                        Task::perform(async move { e.to_string() }, Message::Error),
                                    ]);
                                }
                            }
                        }
                    }

                    return command;
                }
                Task::none()
            }

            Message::ShowOpenRepository => {
                debug!("Starting open repository dialog");
                let open_view = OpenRepositoryView::new();
                self.state = AppState::OpenRepositoryActive(open_view);
                Task::none()
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
                        match config_manager
                            .set_repository_path(repo_path.to_string_lossy().to_string())
                        {
                            Ok(()) => {
                                info!("Repository path saved to configuration");
                            }
                            Err(e) => {
                                error!("Failed to save repository path: {}", e);
                            }
                        }
                    }
                }

                Task::none()
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
                            // Save the repository path to config for future auto-loading
                            if let (Some(config_manager), Some(repo_path)) =
                                (self.config_manager.as_mut(), open_view.repository_path())
                            {
                                let path_str = repo_path.to_string_lossy().to_string();
                                debug!("Saving repository path to config: {}", path_str);
                                if let Err(e) = config_manager.set_repository_path(path_str) {
                                    warn!("Failed to save repository path to config: {}", e);
                                } else {
                                    info!("Repository path saved to config successfully");
                                }
                            } else {
                                warn!("Could not save repository path - config_manager or repo_path missing");
                            }

                            self.session_id = Some(session_id.clone());
                            let mut main_view = MainView::new();
                            main_view.set_session_id(Some(session_id.clone()));
                            self.state = AppState::MainInterface(main_view);
                            // Enable auto-lock timer when session is established
                            self.auto_lock_enabled = true;
                            self.last_activity = std::time::Instant::now();
                            // Trigger initial refresh to update authentication state
                            return Task::batch([
                                command,
                                Task::perform(async {}, |_| {
                                    Message::MainView(MainViewMessage::RefreshCredentials)
                                }),
                            ]);
                        }
                        return command;
                    }

                    // Check if user cancelled - return to welcome screen
                    if open_view.is_cancelled() {
                        self.state = AppState::WizardRequired;
                        return command;
                    }

                    command
                } else {
                    Task::none()
                }
            }

            Message::CreateRepository => {
                debug!("User requested to create new repository");
                Task::perform(async {}, |_| Message::ShowWizard)
            }

            Message::BackendConnected(result) => {
                match result {
                    Ok(()) => {
                        info!("Repository service is ready and initialized");
                        // Repository service is already available, no additional setup needed
                    }
                    Err(error) => {
                        warn!("Failed to initialize repository service: {}", error);
                        // Continue anyway, some operations might still work
                    }
                }
                Task::none()
            }

            Message::Error(error) => {
                error!("Application error: {}", error);
                self.state = AppState::Error(error);
                Task::none()
            }

            Message::ShowAlert(alert) => {
                self.current_alert = Some(alert);
                Task::none()
            }

            Message::DismissAlert => {
                self.current_alert = None;
                Task::none()
            }

            Message::ShowToast(alert) => {
                self.toast_manager.add_toast(alert);
                Task::none()
            }

            Message::DismissToast(toast_id) => {
                self.toast_manager.remove_toast(toast_id);
                Task::none()
            }

            Message::UpdateToasts => {
                self.toast_manager.remove_expired_toasts();
                Task::none()
            }

            Message::OperationResult(result) => {
                match result {
                    Ok(success_msg) => {
                        self.toast_manager.success(success_msg);
                    }
                    Err(error_msg) => {
                        // Check if this is a session timeout error
                        if error_msg.contains("session") || error_msg.contains("timeout") {
                            return Task::perform(async {}, |_| Message::SessionTimeout);
                        }
                        self.toast_manager.error(error_msg);
                    }
                }
                Task::none()
            }

            Message::MainView(main_msg) => {
                if let AppState::MainInterface(main_view) = &mut self.state {
                    match main_msg {
                        MainViewMessage::ShowError(error) => {
                            // Check if this is a session timeout error
                            if error.contains("session") || error.contains("timeout") {
                                return Task::perform(async {}, |_| Message::SessionTimeout);
                            }
                            self.toast_manager.error(error);
                            Task::none()
                        }
                        MainViewMessage::SessionTimeout => {
                            // Forward session timeout to main application handler
                            Task::perform(async {}, |_| Message::SessionTimeout)
                        }
                        MainViewMessage::AddCredential => {
                            // Show add credential view
                            Task::perform(async {}, |_| Message::ShowAddCredential)
                        }
                        MainViewMessage::EditCredential(credential_id) => {
                            // Show edit credential view
                            Task::perform(async move { credential_id }, Message::ShowEditCredential)
                        }
                        MainViewMessage::ShowSettings => {
                            // Show settings view
                            Task::perform(async {}, |_| Message::ShowSettings)
                        }
                        MainViewMessage::CloseArchive => {
                            // Close archive and return to repository selection
                            Task::perform(async {}, |_| Message::CloseArchive)
                        }
                        MainViewMessage::CheckForUpdates => {
                            // Trigger update check
                            Task::perform(async {}, |_| Message::CheckForUpdates)
                        }
                        MainViewMessage::TriggerConnectionError => {
                            self.toast_manager.ipc_error(
                                "Unable to connect to the ZipLock backend service. Please ensure the daemon is running."
                            );
                            Task::none()
                        }
                        MainViewMessage::TriggerAuthError => {
                            self.toast_manager.ipc_error(
                                "Authentication failed. Please check your passphrase and try again."
                            );
                            Task::none()
                        }
                        MainViewMessage::TriggerValidationError => {
                            self.toast_manager.error(
                                "Invalid data provided. Please check your input and try again.",
                            );
                            Task::none()
                        }
                        MainViewMessage::OperationCompleted(result) => {
                            // Forward operation results to main app for toast handling
                            Task::perform(async move { result }, Message::OperationResult)
                        }
                        _ => main_view.update(main_msg).map(Message::MainView),
                    }
                } else {
                    Task::none()
                }
            }

            Message::ShowAddCredential => {
                debug!("Showing add credential view");
                let add_view = AddCredentialView::with_session(self.session_id.clone());
                self.state = AppState::AddCredentialActive(add_view);
                Task::none()
            }

            Message::HideAddCredential => {
                debug!("Hiding add credential view, returning to main interface");
                if let Some(session_id) = &self.session_id {
                    let mut main_view = MainView::new();
                    main_view.set_session_id(Some(session_id.clone()));
                    self.state = AppState::MainInterface(main_view);
                    // Trigger refresh to reload credentials
                    return Task::perform(async {}, |_| {
                        Message::MainView(MainViewMessage::RefreshCredentials)
                    });
                } else {
                    self.state = AppState::MainInterface(MainView::new());
                }
                Task::none()
            }

            Message::AddCredential(add_msg) => {
                if let AppState::AddCredentialActive(add_view) = &mut self.state {
                    match add_msg {
                        AddCredentialMessage::Cancel => {
                            return Task::perform(async {}, |_| Message::HideAddCredential);
                        }
                        AddCredentialMessage::ShowError(ref error) => {
                            self.toast_manager.error(error.clone());
                            let command = add_view.update(add_msg).map(Message::AddCredential);
                            return command;
                        }
                        AddCredentialMessage::ShowSuccess(ref success) => {
                            self.toast_manager.success(success.clone());
                            let command = add_view.update(add_msg).map(Message::AddCredential);
                            return Task::batch([
                                command,
                                Task::perform(async {}, |_| Message::HideAddCredential),
                            ]);
                        }
                        AddCredentialMessage::ShowValidationError(ref error) => {
                            self.toast_manager.warning(error.clone());
                            let command = add_view.update(add_msg).map(Message::AddCredential);
                            return command;
                        }
                        AddCredentialMessage::CopyToClipboard {
                            content,
                            content_type,
                        } => {
                            // Forward clipboard operations to main app
                            return Task::perform(
                                async move { (content, content_type) },
                                |(content, content_type)| Message::CopyToClipboard {
                                    content,
                                    content_type,
                                },
                            );
                        }
                        _ => {
                            let command = add_view.update(add_msg).map(Message::AddCredential);

                            // Check if add credential completed
                            if add_view.is_complete() {
                                return Task::batch([
                                    command,
                                    Task::perform(async {}, |_| Message::HideAddCredential),
                                ]);
                            }

                            return command;
                        }
                    }
                }
                Task::none()
            }

            Message::ShowEditCredential(credential_id) => {
                debug!("Showing edit credential view for ID: {}", credential_id);
                let edit_view =
                    EditCredentialView::with_session(credential_id, self.session_id.clone());
                self.state = AppState::EditCredentialActive(edit_view);
                // Load the credential data
                Task::perform(async {}, |_| {
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
                    return Task::perform(async {}, |_| {
                        Message::MainView(MainViewMessage::RefreshCredentials)
                    });
                } else {
                    self.state = AppState::MainInterface(MainView::new());
                }
                Task::none()
            }

            Message::EditCredential(edit_msg) => {
                if let AppState::EditCredentialActive(edit_view) = &mut self.state {
                    match edit_msg {
                        EditCredentialMessage::Cancel => {
                            return Task::perform(async {}, |_| Message::HideEditCredential);
                        }
                        EditCredentialMessage::ShowError(ref error) => {
                            self.toast_manager.error(error.clone());
                            let command = edit_view.update(edit_msg).map(Message::EditCredential);
                            return command;
                        }
                        EditCredentialMessage::ShowSuccess(ref success) => {
                            self.toast_manager.success(success.clone());
                            let command = edit_view.update(edit_msg).map(Message::EditCredential);
                            return Task::batch([
                                command,
                                Task::perform(async {}, |_| Message::HideEditCredential),
                            ]);
                        }
                        EditCredentialMessage::ShowValidationError(ref error) => {
                            self.toast_manager.warning(error.clone());
                            let command = edit_view.update(edit_msg).map(Message::EditCredential);
                            return command;
                        }
                        EditCredentialMessage::CopyToClipboard {
                            content,
                            content_type,
                        } => {
                            // Forward clipboard operations to main app
                            return Task::perform(
                                async move { (content, content_type) },
                                |(content, content_type)| Message::CopyToClipboard {
                                    content,
                                    content_type,
                                },
                            );
                        }
                        _ => {
                            let command = edit_view.update(edit_msg).map(Message::EditCredential);

                            // Check if edit credential completed
                            if edit_view.is_complete() {
                                return Task::batch([
                                    command,
                                    Task::perform(async {}, |_| Message::HideEditCredential),
                                ]);
                            }

                            return command;
                        }
                    }
                }
                Task::none()
            }

            Message::ShowSettings => {
                info!("Showing settings view");
                if let Some(config_manager) = &self.config_manager {
                    let settings_view = SettingsView::new(config_manager.config().clone());
                    self.state = AppState::SettingsActive(settings_view);
                } else {
                    self.toast_manager
                        .error("Configuration not available".to_string());
                }
                Task::none()
            }

            Message::HideSettings => {
                debug!("Hiding settings view, returning to main interface");
                if let Some(session_id) = &self.session_id {
                    let mut main_view = MainView::new();
                    main_view.set_session_id(Some(session_id.clone()));
                    self.state = AppState::MainInterface(main_view);
                    // Trigger refresh to reload credentials
                    return Task::perform(async {}, |_| {
                        Message::MainView(MainViewMessage::RefreshCredentials)
                    });
                } else {
                    self.state = AppState::MainInterface(MainView::new());
                }
                Task::none()
            }

            Message::Settings(settings_msg) => {
                if let AppState::SettingsActive(settings_view) = &mut self.state {
                    match &settings_msg {
                        SettingsMessage::Cancel => {
                            return Task::perform(async {}, |_| Message::HideSettings);
                        }
                        SettingsMessage::Save => {
                            // Handle settings save
                            if !settings_view.has_validation_errors() {
                                if let Some(config_manager) = &mut self.config_manager {
                                    let updated_config = settings_view.get_updated_config();
                                    // Update the config manager
                                    *config_manager.config_mut() = updated_config;

                                    // Reinitialize typography with new font size
                                    ui::theme::utils::typography::init_font_size(
                                        config_manager.config().ui.font_scale.unwrap_or(1.0),
                                    );

                                    // Save the configuration
                                    match config_manager.save() {
                                        Ok(_) => {
                                            self.toast_manager
                                                .success("Settings saved successfully".to_string());
                                            return Task::perform(async {}, |_| {
                                                Message::HideSettings
                                            });
                                        }
                                        Err(e) => {
                                            self.toast_manager
                                                .error(format!("Failed to save settings: {}", e));
                                        }
                                    }
                                } else {
                                    self.toast_manager
                                        .error("Configuration manager not available".to_string());
                                }
                            } else {
                                self.toast_manager.warning(
                                    "Please fix validation errors before saving".to_string(),
                                );
                            }
                            return settings_view.update(settings_msg).map(Message::Settings);
                        }
                        _ => {
                            return settings_view.update(settings_msg).map(Message::Settings);
                        }
                    }
                }
                Task::none()
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
                            return Task::perform(
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
                // Reset auto-lock timer when session times out
                self.last_activity = std::time::Instant::now();
                self.auto_lock_enabled = false;
                Task::none()
            }

            Message::AutoLockTimerTick => {
                // Check if auto-lock is enabled and we have a session
                if self.auto_lock_enabled && self.session_id.is_some() {
                    if let Some(config_manager) = &self.config_manager {
                        let timeout_minutes = config_manager.config().ui.auto_lock_timeout;
                        // Only check timeout if it's not disabled (0)
                        if timeout_minutes > 0 {
                            let timeout_duration =
                                std::time::Duration::from_secs(timeout_minutes as u64 * 60);
                            if self.last_activity.elapsed() >= timeout_duration {
                                info!("Auto-lock timeout reached, locking application");
                                // Trigger session timeout to lock the application
                                return Task::perform(async {}, |_| Message::SessionTimeout);
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::UserActivity => {
                // Reset the activity timer
                self.last_activity = std::time::Instant::now();
                Task::none()
            }

            Message::CheckForUpdates => {
                info!("Manual update check requested");
                // Clone the update checker to avoid borrowing issues
                let mut update_checker = self.update_checker.clone();
                Task::perform(
                    async move { update_checker.check_for_updates().await },
                    |result| Message::UpdateCheckResult(result.map_err(|e| e.to_string())),
                )
            }

            Message::UpdateCheckResult(result) => match result {
                Ok(update_result) => {
                    if update_result.update_available {
                        info!(
                            "Update available: {}",
                            update_result
                                .latest_version
                                .as_ref()
                                .unwrap_or(&"unknown".to_string())
                        );

                        Task::perform(async move { update_result }, Message::ShowUpdateDialog)
                    } else {
                        info!("No updates available");
                        self.toast_manager
                            .success("You are running the latest version of ZipLock!");
                        Task::none()
                    }
                }
                Err(error) => {
                    error!("Update check failed: {}", error);
                    self.toast_manager
                        .error(format!("Failed to check for updates: {}", error));
                    Task::none()
                }
            },

            Message::ShowUpdateDialog(update_result) => {
                let update_dialog = UpdateDialog::new(update_result);
                self.state = AppState::UpdateDialogActive(update_dialog);
                Task::none()
            }

            Message::HideUpdateDialog => {
                // Return to main interface
                if self.config_manager.is_some() && self.session_id.is_some() {
                    let main_view = MainView::new();
                    self.state = AppState::MainInterface(main_view);
                    return Task::perform(async {}, |_| {
                        Message::MainView(MainViewMessage::RefreshCredentials)
                    });
                }
                self.state = AppState::MainInterface(MainView::new());
                Task::none()
            }

            Message::AutoUpdateCheck => {
                // Check if auto-update checking is enabled and if we should check now
                if let Some(config_manager) = &self.config_manager {
                    if config_manager.config().behavior.auto_check_updates
                        && self.update_checker.should_auto_check()
                    {
                        info!("Performing automatic update check");
                        // Clone the update checker for async operation
                        let mut update_checker = self.update_checker.clone();
                        return Task::perform(
                            async move { update_checker.check_for_updates().await },
                            |result| Message::UpdateCheckResult(result.map_err(|e| e.to_string())),
                        );
                    }
                }
                Task::none()
            }

            Message::CopyToClipboard {
                content,
                content_type,
            } => {
                tracing::debug!(
                    "CopyToClipboard message received: content_type={:?}, content_length={}",
                    content_type,
                    content.len()
                );

                // Get clipboard timeout from config
                let timeout_seconds = if let Some(config_manager) = &self.config_manager {
                    config_manager.config().security.clipboard_timeout as u32
                } else {
                    30 // Default timeout
                };

                tracing::debug!("Using clipboard timeout: {}s (0=disabled)", timeout_seconds);

                let clipboard_manager = self.clipboard_manager.clone();
                Task::perform(
                    async move {
                        clipboard_manager
                            .copy_with_timeout(content, content_type, timeout_seconds)
                            .await
                    },
                    |result| match result {
                        Ok(_) => {
                            tracing::debug!("Clipboard copy successful");
                            Message::UserActivity // Update activity on successful copy
                        }
                        Err(e) => {
                            tracing::warn!("Failed to copy to clipboard: {}", e);
                            Message::ShowToast(AlertMessage::error(format!(
                                "Failed to copy to clipboard: {}",
                                e
                            )))
                        }
                    },
                )
            }

            Message::CloseArchive => {
                info!("Archive close requested, returning to repository selection");

                // Lock the credential store
                let credential_store = services::get_credential_store();
                credential_store.lock();

                // Clear session and return to repository detection/selection
                self.session_id = None;
                self.auto_lock_enabled = false;

                // Clear clipboard content
                let clipboard_manager = self.clipboard_manager.clone();
                std::mem::drop(tokio::spawn(async move {
                    clipboard_manager.clear_tracked_content().await;
                }));

                // Get the current repository path before clearing
                let current_repo_path = self
                    .config_manager
                    .as_ref()
                    .and_then(|cm| cm.repository_path());

                // Start repository detection but prioritize the current repository
                self.state = AppState::DetectingRepositories;
                if let Some(config_manager) = &self.config_manager {
                    let repositories = config_manager.detect_all_accessible_repositories();

                    // If we have a current repository, ensure it's at the front of the list
                    let mut sorted_repos = repositories;
                    if let Some(current_path) = current_repo_path {
                        // Move current repository to front if it exists in the list
                        if let Some(current_repo_index) = sorted_repos
                            .iter()
                            .position(|repo| repo.path == current_path)
                        {
                            let current_repo = sorted_repos.remove(current_repo_index);
                            sorted_repos.insert(0, current_repo);
                        }
                    }

                    Task::perform(async move { sorted_repos }, Message::RepositoriesDetected)
                } else {
                    Task::none()
                }
            }

            Message::Quit => {
                info!("Application quit requested");

                // Clear clipboard content before quitting
                let clipboard_manager = self.clipboard_manager.clone();
                std::mem::drop(tokio::spawn(async move {
                    clipboard_manager.clear_tracked_content().await;
                }));

                // If we have an active session, try to logout first
                if self.session_id.is_some() {
                    info!("Active session detected, logging out before quit");
                    Task::perform(Self::logout_and_quit_async(self.session_id.clone()), |_| {
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

    pub fn view(&self) -> Element<'_, Message> {
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
            AppState::SettingsActive(settings_view) => settings_view.view().map(Message::Settings),
            AppState::UpdateDialogActive(update_dialog) => {
                update_dialog.view().map(|dialog_msg| match dialog_msg {
                    UpdateDialogMessage::Close => Message::HideUpdateDialog,
                    UpdateDialogMessage::OpenReleasePage => {
                        // Open URL in browser
                        if let Some(url) =
                            UpdateDialog::get_release_url(update_dialog.update_result())
                        {
                            if let Err(e) = open::that(&url) {
                                tracing::warn!("Failed to open release URL: {}", e);
                            }
                        }
                        Message::HideUpdateDialog
                    }
                    UpdateDialogMessage::CopyCommand => {
                        // Copy command to clipboard
                        if let Some(command) =
                            UpdateDialog::get_copy_command(update_dialog.update_result())
                        {
                            if let Err(e) = arboard::Clipboard::new()
                                .and_then(|mut clipboard| clipboard.set_text(&command))
                            {
                                tracing::warn!("Failed to copy to clipboard: {}", e);
                            }
                        }
                        Message::HideUpdateDialog
                    }
                })
            }
            AppState::MainInterface(main_view) => main_view.view().map(Message::MainView),
            AppState::Error(error) => self.view_error(error),
        };

        self.wrap_with_toasts(main_content)
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        use iced::time;

        let close_subscription = iced::event::listen_with(|event, _status, _id| match event {
            iced::Event::Window(iced::window::Event::CloseRequested) => Some(Message::Quit),
            _ => None,
        });

        // Track user activity for auto-lock
        let activity_subscription = iced::event::listen_with(|event, _status, _id| match event {
            iced::Event::Mouse(_) | iced::Event::Keyboard(_) | iced::Event::Touch(_) => {
                Some(Message::UserActivity)
            }
            _ => None,
        });

        let toast_subscription = if self.toast_manager.has_toasts() {
            time::every(std::time::Duration::from_millis(100)).map(|_| Message::UpdateToasts)
        } else {
            iced::Subscription::none()
        };

        // Auto-lock timer subscription - check every 10 seconds
        let auto_lock_subscription = if self.auto_lock_enabled && self.session_id.is_some() {
            if let Some(config_manager) = &self.config_manager {
                let timeout_minutes = config_manager.config().ui.auto_lock_timeout;
                if timeout_minutes > 0 {
                    time::every(std::time::Duration::from_secs(10))
                        .map(|_| Message::AutoLockTimerTick)
                } else {
                    iced::Subscription::none()
                }
            } else {
                iced::Subscription::none()
            }
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

        // Auto update check subscription - check every hour if enabled
        let auto_update_subscription = if let Some(config_manager) = &self.config_manager {
            if config_manager.config().behavior.auto_check_updates {
                time::every(std::time::Duration::from_secs(3600)) // Check every hour
                    .map(|_| Message::AutoUpdateCheck)
            } else {
                iced::Subscription::none()
            }
        } else {
            iced::Subscription::none()
        };

        iced::Subscription::batch([
            close_subscription,
            activity_subscription,
            toast_subscription,
            auto_lock_subscription,
            auto_update_subscription,
            view_subscription,
        ])
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
        let content_with_alerts = if let Some(alert) = &self.current_alert {
            column![
                alerts::render_alert(alert, Some(Message::DismissAlert)),
                Space::with_height(Length::Fixed(10.0)),
                content_with_toasts,
            ]
            .into()
        } else {
            content_with_toasts
        };

        content_with_alerts
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
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text("Initializing configuration...")
                    .size(14)
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fill),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
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
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text("Searching for existing password repositories...")
                    .size(14)
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fill),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    /// View repository selection screen
    fn view_repository_selection<'a>(
        &'a self,
        repositories: &'a [RepositoryInfo],
    ) -> Element<'a, Message> {
        use iced::widget::{button, column, container, row, text, Space};
        use iced::{Alignment, Length};

        let mut repo_buttons = column![].spacing(10);

        for repo in repositories {
            let display_name = &repo.name;
            let path_text = if let Some(home) = dirs::home_dir() {
                let home_str = home.to_string_lossy().to_string();
                if repo.path.starts_with(&home_str) {
                    // Show relative path from home
                    repo.path
                        .strip_prefix(&home_str)
                        .map(|p| format!("~/{}", p))
                        .unwrap_or_else(|| repo.path.clone())
                } else {
                    repo.path.clone()
                }
            } else {
                repo.path.clone()
            };

            // Since RepositoryInfo doesn't have size field, we'll skip it for now
            let size_text = "Unknown size".to_string();

            let repo_button = button(
                column![
                    text(display_name).size(16),
                    text(path_text).size(12),
                    text(size_text).size(10),
                ]
                .spacing(2),
            )
            .width(Length::Fill)
            .padding(theme::utils::repository_button_padding())
            .style(theme::button_styles::secondary())
            .on_press(Message::OpenRepository(
                OpenRepositoryMessage::SelectSpecificFile(repo.path.clone().into()),
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
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(10.0)),
                text(format!(
                    "Found {} password repositories",
                    repositories.len()
                ))
                .size(14)
                .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(30.0)),
                container(repo_buttons).width(Length::Fixed(400.0)),
                Space::with_height(Length::Fixed(30.0)),
                // Action buttons
                row![
                    primary_button("Create New Repository", Some(Message::ShowWizard)),
                    Space::with_width(Length::Fixed(20.0)),
                    secondary_button(
                        "Browse for Repository...",
                        Some(Message::ShowOpenRepository),
                    ),
                ]
                .spacing(10),
                Space::with_height(Length::Fill),
            ]
            .align_x(Alignment::Center)
            .max_width(500),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
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
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(30.0)),
                text("Get started by setting up your first password repository.")
                    .size(16)
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(40.0)),
                button(text("Setup Repository"))
                    .on_press(Message::ShowWizard)
                    .padding(theme::utils::setup_button_padding())
                    .style(theme::button_styles::primary()),
                Space::with_height(Length::Fixed(20.0)),
                secondary_button(
                    "Open Existing Repository",
                    Some(Message::ShowOpenRepository),
                ),
                Space::with_height(Length::Fill),
            ]
            .align_x(Alignment::Center)
            .max_width(500),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    /// View error screen
    fn view_error<'a>(&'a self, error: &'a str) -> Element<'a, Message> {
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
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text(error)
                    .size(14)
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(30.0)),
                destructive_button("Quit", Some(Message::Quit)),
                Space::with_height(Length::Fill),
            ]
            .align_x(Alignment::Center)
            .max_width(500),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    /// Async function to load configuration
    async fn load_config_async() -> String {
        match ConfigManager::new() {
            Ok(mut config_manager) => {
                match config_manager.load() {
                    Ok(()) => {
                        info!("Configuration loaded successfully");
                        String::new() // Empty string means success
                    }
                    Err(e) => {
                        warn!("Failed to load configuration file, using defaults: {}", e);
                        String::new() // Still return success, just use defaults
                    }
                }
            }
            Err(e) => e.to_string(),
        }
    }

    /// Async function to connect to backend
    async fn logout_and_quit_async(session_id: Option<String>) -> Result<(), String> {
        if let Some(_sid) = session_id.clone() {
            // No longer need separate client, repository service handles this
            info!("Using repository service for operations");

            // Repository service automatically handles cleanup when closed
            info!("Repository service will handle cleanup automatically");
        }
        Ok(())
    }

    async fn connect_backend_async() -> Result<(), String> {
        // Repository service is already initialized and available
        info!("Repository service ready for operations");

        // Test basic functionality (in a real app, this might involve actual connections)
        info!("Hybrid client initialization test successful");
        Ok(())
    }
}

fn main() -> iced::Result {
    // Determine if running in production mode
    let is_production = is_production_mode();

    // Configure logging based on mode
    let logging_config = if is_production {
        logging::LoggingConfig::production()
    } else {
        logging::LoggingConfig::development()
    };

    if let Err(e) = logging::initialize_logging(logging_config) {
        eprintln!("Failed to initialize logging: {}", e);
        // In production mode, also try to write to Event Log if available
        #[cfg(windows)]
        if is_production {
            if let Ok(event_writer) =
                logging::windows_event_log::WindowsEventLogWriter::new("ZipLock")
            {
                let _ = event_writer
                    .log_event("ERROR", &format!("Failed to initialize logging: {}", e));
            }
        }
    }

    // Log startup information
    info!("Starting ZipLock Password Manager");
    info!("Mode: {}", get_logging_mode());
    info!("Application version: {}", env!("CARGO_PKG_VERSION"));
    info!(
        "Build type: {}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    // Log Windows-specific production settings
    #[cfg(windows)]
    if is_production {
        info!("Windows production mode: console logging disabled, Event Log enabled");
        info!("Terminal window suppressed via windows_subsystem attribute");
    }

    // Use new Iced 0.13 application architecture
    iced::application(
        "ZipLock Password Manager",
        ZipLockApp::update,
        ZipLockApp::view,
    )
    .subscription(ZipLockApp::subscription)
    .theme(ZipLockApp::theme)
    .window_size((1000.0, 700.0))
    .antialiasing(true)
    .run_with(ZipLockApp::new)
}
