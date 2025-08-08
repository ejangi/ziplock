//! ZipLock Linux Frontend
//!
//! This is the Linux desktop frontend for ZipLock, built with the Iced GUI framework.
//! It provides a native Linux interface for managing encrypted password archives.

use iced::{widget::svg, Application, Command, Element, Settings, Theme};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod ipc;
mod ui;

use ui::theme::alerts::AlertMessage;

use ui::{create_ziplock_theme, theme};

use config::{ConfigManager, RepositoryInfo};
use ipc::IpcClient;
use ui::views::main::{MainView, MainViewMessage};
use ui::views::{OpenRepositoryMessage, OpenRepositoryView, RepositoryWizard, WizardMessage};

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

    // Alert management
    ShowAlert(AlertMessage),
    DismissAlert,

    // General
    Quit,
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
    MainInterface,
    Error(String),
}

/// Main application structure
pub struct ZipLockApp {
    state: AppState,
    config_manager: Option<ConfigManager>,
    ipc_client: Option<IpcClient>,
    theme: Theme,
    current_alert: Option<AlertMessage>,
    main_view: MainView,
    detected_repositories: Vec<RepositoryInfo>,
}

impl Application for ZipLockApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        info!("Initializing ZipLock Linux frontend");

        let app = Self {
            state: AppState::Loading,
            config_manager: None,
            ipc_client: None,
            theme: create_ziplock_theme(),
            current_alert: None,
            main_view: MainView::new(),
            detected_repositories: Vec::new(),
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
            AppState::MainInterface => "ZipLock Password Manager".to_string(),
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

                    return Command::perform(
                        async move { repositories },
                        Message::RepositoriesDetected,
                    );
                } else {
                    self.state = AppState::Error("Failed to initialize configuration".to_string());
                    Command::none()
                }
            }

            Message::RepositoriesDetected(repositories) => {
                info!("Detected {} repositories", repositories.len());
                self.detected_repositories = repositories.clone();

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
                        self.current_alert = Some(AlertMessage::warning(format!(
                            "Repository Validation Failed: {}",
                            error
                        )));
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
                self.state = AppState::MainInterface;
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

                self.state = AppState::MainInterface;

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
                        return Command::batch([
                            command,
                            Command::perform(async {}, |_| Message::HideOpenRepository),
                        ]);
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

            Message::MainView(main_msg) => match main_msg {
                MainViewMessage::ShowError(error) => {
                    self.current_alert = Some(AlertMessage::ipc_error(error));
                    Command::none()
                }
                _ => self.main_view.update(main_msg).map(Message::MainView),
            },

            Message::Quit => {
                info!("Application quit requested");
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<Message> {
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
            AppState::MainInterface => self.view_main_interface(),
            AppState::Error(error) => self.view_error(error),
        };

        self.wrap_with_alert(main_content)
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

impl ZipLockApp {
    /// Wraps any view content with alert display if an alert is present
    fn wrap_with_alert<'a>(&'a self, content: Element<'a, Message>) -> Element<'a, Message> {
        use iced::widget::{column, Space};
        use iced::Length;
        use ui::theme::alerts;

        if let Some(alert) = &self.current_alert {
            column![
                alerts::render_alert(alert, Some(Message::DismissAlert)),
                Space::with_height(Length::Fixed(10.0)),
                content,
            ]
            .into()
        } else {
            content
        }
    }
    /// View loading screen
    fn view_loading(&self) -> Element<Message> {
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
    fn view_detecting_repositories(&self) -> Element<Message> {
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
    fn view_repository_selection(&self, repositories: &[RepositoryInfo]) -> Element<Message> {
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
            .padding([15, 20])
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
                        .padding([10, 20]),
                    Space::with_width(Length::Fixed(20.0)),
                    button("Browse for Repository...")
                        .on_press(Message::ShowOpenRepository)
                        .padding([10, 20]),
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
    fn view_wizard_required(&self) -> Element<Message> {
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
                    .padding([15, 30]),
                Space::with_height(Length::Fixed(20.0)),
                button("Open Existing Repository")
                    .on_press(Message::ShowOpenRepository)
                    .padding([10, 20]),
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

    /// View main interface
    fn view_main_interface(&self) -> Element<Message> {
        self.main_view.view().map(Message::MainView)
    }

    /// View error screen
    fn view_error(&self, error: &str) -> Element<Message> {
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
                button("Quit").on_press(Message::Quit).padding([10, 20]),
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
    async fn connect_backend_async() -> Result<(), String> {
        let socket_path = IpcClient::default_socket_path();
        let mut client = IpcClient::new(socket_path);

        match client.connect().await {
            Ok(()) => {
                // Test the connection with a ping
                match client.ping().await {
                    Ok(()) => Ok(()),
                    Err(e) => Err(format!("Backend ping failed: {}", e)),
                }
            }
            Err(e) => Err(format!("Backend connection failed: {}", e)),
        }
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

    info!("Starting ZipLock Linux frontend");

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
