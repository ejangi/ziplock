//! Open Repository View
//!
//! This view handles opening an existing ZipLock repository. It provides
//! file selection and passphrase input with validation styling consistent
//! with the wizard interface.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Command, Element, Length};
use std::path::PathBuf;
use tracing::{debug, error, info};

use crate::ui::theme::{self, button_styles, container_styles, utils};
use ziplock_shared::validation::PassphraseValidator;

/// Messages for the open repository view
#[derive(Debug, Clone)]
pub enum OpenRepositoryMessage {
    /// Select repository file
    SelectFile,
    /// Directly select a specific file
    SelectSpecificFile(PathBuf),
    /// File was selected
    FileSelected(Option<PathBuf>),
    /// Passphrase input changed
    PassphraseChanged(String),
    /// Toggle passphrase visibility
    TogglePassphraseVisibility,
    /// Attempt to open the repository
    OpenRepository,
    /// Cancel and return to previous view
    Cancel,
    /// Opening process completed
    OpenComplete(Result<(), String>),
}

/// State of the repository opening process
#[derive(Debug, Clone)]
pub enum OpenState {
    /// Selecting file and entering passphrase
    Input,
    /// Opening repository in progress
    Opening,
    /// Opening completed successfully
    Complete,
    /// User cancelled the operation
    Cancelled,
    /// Error occurred
    Error(String),
}

/// Open Repository view component
#[derive(Debug)]
pub struct OpenRepositoryView {
    /// Current state of the opening process
    state: OpenState,
    /// Selected repository file path
    selected_file: Option<PathBuf>,
    /// User-entered passphrase
    passphrase: String,
    /// Whether to show passphrase as plain text
    show_passphrase: bool,
    /// Passphrase validator for styling
    passphrase_validator: PassphraseValidator,
    /// Whether the form can be submitted
    can_open: bool,
}

impl Default for OpenRepositoryView {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenRepositoryView {
    /// Create a new open repository view
    pub fn new() -> Self {
        Self {
            state: OpenState::Input,
            selected_file: None,
            passphrase: String::new(),
            show_passphrase: false,
            passphrase_validator: PassphraseValidator::minimal(), // For opening, we just need any passphrase
            can_open: false,
        }
    }

    /// Create a new open repository view with a pre-selected repository file
    pub fn with_repository(repository_path: PathBuf) -> Self {
        Self {
            state: OpenState::Input,
            selected_file: Some(repository_path),
            passphrase: String::new(),
            show_passphrase: false,
            passphrase_validator: PassphraseValidator::minimal(),
            can_open: false,
        }
    }

    /// Update the view with a message
    pub fn update(&mut self, message: OpenRepositoryMessage) -> Command<OpenRepositoryMessage> {
        match message {
            OpenRepositoryMessage::SelectFile => {
                debug!("Opening file selection dialog");
                Command::perform(
                    Self::select_file_async(),
                    OpenRepositoryMessage::FileSelected,
                )
            }

            OpenRepositoryMessage::SelectSpecificFile(path) => {
                debug!("Directly selecting repository file: {:?}", path);
                self.selected_file = Some(path);
                self.state = OpenState::Input;
                Command::none()
            }

            OpenRepositoryMessage::FileSelected(file_path) => {
                if let Some(path) = file_path {
                    info!("Repository file selected: {:?}", path);
                    self.selected_file = Some(path);
                } else {
                    debug!("File selection cancelled");
                }
                self.update_can_open();
                Command::none()
            }

            OpenRepositoryMessage::PassphraseChanged(passphrase) => {
                self.passphrase = passphrase;
                self.update_can_open();
                Command::none()
            }

            OpenRepositoryMessage::TogglePassphraseVisibility => {
                self.show_passphrase = !self.show_passphrase;
                Command::none()
            }

            OpenRepositoryMessage::OpenRepository => {
                if self.can_open {
                    info!("Attempting to open repository");
                    self.state = OpenState::Opening;

                    let file_path = self.selected_file.clone().unwrap();
                    let passphrase = self.passphrase.clone();

                    Command::perform(
                        Self::open_repository_async(file_path, passphrase),
                        OpenRepositoryMessage::OpenComplete,
                    )
                } else {
                    Command::none()
                }
            }

            OpenRepositoryMessage::Cancel => {
                debug!("Open repository cancelled");
                self.state = OpenState::Cancelled;
                Command::none()
            }

            OpenRepositoryMessage::OpenComplete(result) => {
                match result {
                    Ok(()) => {
                        info!("Repository opened successfully");
                        self.state = OpenState::Complete;
                    }
                    Err(error) => {
                        error!("Failed to open repository: {}", error);
                        self.state = OpenState::Error(error);
                    }
                }
                Command::none()
            }
        }
    }

    /// Render the view
    pub fn view(&self) -> Element<OpenRepositoryMessage> {
        match &self.state {
            OpenState::Input => self.view_input(),
            OpenState::Opening => self.view_opening(),
            OpenState::Complete => self.view_complete(),
            OpenState::Cancelled => self.view_input(), // Show input form when cancelled
            OpenState::Error(error) => self.view_error(error),
        }
    }

    /// Render the input form
    fn view_input(&self) -> Element<OpenRepositoryMessage> {
        let header = self.view_header();

        let file_selection = self.view_file_selection();
        let passphrase_input = self.view_passphrase_input();
        let navigation = self.view_navigation();

        scrollable(column![
            Space::with_height(Length::Fixed(40.0)), // Top padding for centering effect
            container(
                column![
                    header,
                    Space::with_height(Length::Fixed(30.0)),
                    file_selection,
                    Space::with_height(Length::Fixed(20.0)),
                    passphrase_input,
                    Space::with_height(Length::Fixed(40.0)),
                    navigation,
                ]
                .align_items(Alignment::Center)
                .max_width(500),
            )
            .width(Length::Fill)
            .center_x(),
            Space::with_height(Length::Fixed(40.0)), // Bottom padding for centering effect
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render the header
    fn view_header(&self) -> Element<OpenRepositoryMessage> {
        column![
            iced::widget::svg(theme::ziplock_logo())
                .width(Length::Fixed(64.0))
                .height(Length::Fixed(64.0)),
            Space::with_height(Length::Fixed(20.0)),
            text("Open Repository")
                .size(28)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::with_height(Length::Fixed(10.0)),
            text("Select your repository file and enter your passphrase to unlock it.")
                .size(14)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Render the file selection section
    fn view_file_selection(&self) -> Element<OpenRepositoryMessage> {
        let file_display = if let Some(ref path) = self.selected_file {
            text(format!(
                "Selected: {}",
                path.file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                    .to_string_lossy()
            ))
            .size(14)
            .style(iced::theme::Text::Color(theme::SUCCESS_GREEN))
        } else {
            text("No file selected")
                .size(14)
                .style(iced::theme::Text::Color(Color::from_rgb(0.5, 0.5, 0.5)))
        };

        column![
            text("Repository File")
                .size(16)
                .horizontal_alignment(iced::alignment::Horizontal::Left),
            Space::with_height(Length::Fixed(8.0)),
            button("Browse...")
                .on_press(OpenRepositoryMessage::SelectFile)
                .style(button_styles::secondary())
                .padding(utils::button_padding()),
            Space::with_height(Length::Fixed(8.0)),
            file_display,
        ]
        .width(Length::Fill)
        .into()
    }

    /// Render the passphrase input section
    fn view_passphrase_input(&self) -> Element<OpenRepositoryMessage> {
        let passphrase_input = text_input(
            if self.show_passphrase {
                "Enter your passphrase"
            } else {
                "Enter your passphrase"
            },
            &self.passphrase,
        )
        .on_input(OpenRepositoryMessage::PassphraseChanged)
        .on_submit(OpenRepositoryMessage::OpenRepository)
        .secure(!self.show_passphrase)
        .style(self.get_passphrase_style())
        .padding(utils::button_padding())
        .width(Length::Fill);

        let toggle_button = utils::password_visibility_toggle(
            self.show_passphrase,
            OpenRepositoryMessage::TogglePassphraseVisibility,
        );

        column![
            text("Master Passphrase")
                .size(16)
                .horizontal_alignment(iced::alignment::Horizontal::Left),
            Space::with_height(Length::Fixed(8.0)),
            row![
                passphrase_input,
                Space::with_width(Length::Fixed(10.0)),
                toggle_button
            ]
            .align_items(Alignment::Center),
        ]
        .width(Length::Fill)
        .into()
    }

    /// Render navigation buttons
    fn view_navigation(&self) -> Element<OpenRepositoryMessage> {
        let open_button = if self.can_open {
            button("Open Repository")
                .on_press(OpenRepositoryMessage::OpenRepository)
                .style(button_styles::primary())
                .padding(utils::button_padding())
        } else {
            button("Open Repository")
                .style(button_styles::disabled())
                .padding(utils::button_padding())
        };

        let cancel_button = button("Cancel")
            .on_press(OpenRepositoryMessage::Cancel)
            .style(button_styles::secondary())
            .padding(utils::button_padding());

        row![
            cancel_button,
            Space::with_width(Length::Fixed(20.0)),
            open_button,
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Render the opening progress view
    fn view_opening(&self) -> Element<OpenRepositoryMessage> {
        container(
            column![
                iced::widget::svg(theme::ziplock_logo())
                    .width(Length::Fixed(64.0))
                    .height(Length::Fixed(64.0)),
                Space::with_height(Length::Fixed(20.0)),
                text("Opening Repository...")
                    .size(24)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(10.0)),
                text("Please wait while we unlock your repository.")
                    .size(14)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
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

    /// Render the completion view
    fn view_complete(&self) -> Element<OpenRepositoryMessage> {
        container(
            column![
                text("✅")
                    .size(48)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text("Repository Opened")
                    .size(24)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(10.0)),
                text("Your repository has been successfully opened and unlocked.")
                    .size(14)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
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

    /// Render the error view
    fn view_error(&self, error: &str) -> Element<OpenRepositoryMessage> {
        container(
            column![
                text("❌")
                    .size(48)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text("Failed to Open Repository")
                    .size(24)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                container(
                    text(error)
                        .size(14)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                )
                .style(container_styles::error_alert())
                .padding(utils::alert_padding()),
                Space::with_height(Length::Fixed(30.0)),
                button("Try Again")
                    .on_press(OpenRepositoryMessage::Cancel)
                    .style(button_styles::primary())
                    .padding(utils::button_padding()),
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

    /// Update whether the repository can be opened
    fn update_can_open(&mut self) {
        self.can_open = self.selected_file.is_some() && !self.passphrase.is_empty();
    }

    /// Get the style for the passphrase input field
    fn get_passphrase_style(&self) -> iced::theme::TextInput {
        if self.passphrase.is_empty() {
            iced::theme::TextInput::Default
        } else {
            // For opening, we don't validate strength, just that it's not empty
            // This gives visual feedback that something has been entered
            iced::theme::TextInput::Custom(Box::new(PassphraseTextInputStyle::Neutral))
        }
    }

    /// Reset the view to initial state
    fn reset(&mut self) {
        self.state = OpenState::Input;
        self.selected_file = None;
        self.passphrase.clear();
        self.show_passphrase = false;
        self.can_open = false;
    }

    /// Check if the opening process is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.state, OpenState::Complete)
    }

    /// Check if the operation was cancelled
    pub fn is_cancelled(&self) -> bool {
        matches!(self.state, OpenState::Cancelled)
    }

    /// Get the selected repository path if complete
    pub fn repository_path(&self) -> Option<&PathBuf> {
        if self.is_complete() {
            self.selected_file.as_ref()
        } else {
            None
        }
    }

    /// Async function to select a file
    async fn select_file_async() -> Option<PathBuf> {
        // This would typically use a file dialog
        // For now, we'll use a simple approach that could be replaced with a proper dialog
        #[cfg(feature = "file-dialog")]
        {
            use rfd::AsyncFileDialog;

            AsyncFileDialog::new()
                .add_filter("ZipLock Repository", &["7z"])
                .set_title("Select ZipLock Repository")
                .pick_file()
                .await
                .map(|handle| handle.path().to_path_buf())
        }

        #[cfg(not(feature = "file-dialog"))]
        {
            // Fallback implementation - could prompt user to enter path manually
            // or use platform-specific dialog
            None
        }
    }

    /// Async function to open a repository
    async fn open_repository_async(file_path: PathBuf, passphrase: String) -> Result<(), String> {
        use crate::ipc::IpcClient;

        // Connect to backend
        let socket_path = IpcClient::default_socket_path();
        let mut client = IpcClient::new(socket_path);

        client
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to backend: {}", e))?;

        // Attempt to open the archive
        client
            .open_archive(file_path, passphrase)
            .await
            .map_err(|e| format!("Failed to open repository: {}", e))?;

        Ok(())
    }
}

/// Custom text input styles for passphrase input
#[derive(Debug, Clone)]
enum PassphraseTextInputStyle {
    Neutral, // For when we don't want to indicate valid/invalid, just that input exists
}

impl iced::widget::text_input::StyleSheet for PassphraseTextInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Color::WHITE.into(),
            border: iced::Border {
                color: theme::LOGO_PURPLE,
                width: 2.0,
                radius: 4.0.into(),
            },
            icon_color: Color::from_rgb(0.5, 0.5, 0.5),
        }
    }

    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Color::WHITE.into(),
            border: iced::Border {
                color: theme::LOGO_PURPLE,
                width: 3.0,
                radius: 4.0.into(),
            },
            icon_color: Color::from_rgb(0.5, 0.5, 0.5),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.5, 0.5, 0.5)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::BLACK
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.5, 0.5, 0.5)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.8, 0.8, 1.0)
    }

    fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Color::from_rgb(0.95, 0.95, 0.95).into(),
            border: iced::Border {
                color: Color::from_rgb(0.8, 0.8, 0.8),
                width: 1.0,
                radius: 4.0.into(),
            },
            icon_color: Color::from_rgb(0.5, 0.5, 0.5),
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::text_input::Appearance {
        self.active(style)
    }
}
