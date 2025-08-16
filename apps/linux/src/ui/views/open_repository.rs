//! Open Repository View
//!
//! This view handles opening an existing ZipLock repository. It provides
//! file selection and passphrase input with validation styling consistent
//! with the wizard interface.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Command, Element, Length};
use std::path::PathBuf;
use tracing::{debug, error, info};

use crate::ui::theme::{self, button_styles, container_styles, utils, MEDIUM_GRAY};

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
    /// Try again after error (preserves repository selection)
    TryAgain,
    /// Opening process completed
    OpenComplete(Result<String, String>), // Now returns session ID on success
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
    /// Whether repository can be opened
    can_open: bool,
    /// Session ID if opening is complete
    session_id: Option<String>,
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
            can_open: false,
            session_id: None,
        }
    }

    /// Create a new open repository view with a pre-selected repository file
    pub fn with_repository(repository_path: PathBuf) -> Self {
        Self {
            state: OpenState::Input,
            selected_file: Some(repository_path),
            passphrase: String::new(),
            show_passphrase: false,
            can_open: false,
            session_id: None,
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

            OpenRepositoryMessage::TryAgain => {
                debug!("User clicked try again, returning to input state");
                self.state = OpenState::Input;
                self.passphrase.clear();
                self.show_passphrase = false;
                self.update_can_open();
                Command::none()
            }

            OpenRepositoryMessage::OpenComplete(result) => {
                match result {
                    Ok(session_id) => {
                        info!("Repository opened successfully");
                        self.session_id = Some(session_id);
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
    pub fn view(&self) -> Element<'_, OpenRepositoryMessage> {
        match &self.state {
            OpenState::Input => self.view_input(),
            OpenState::Opening => self.view_opening(),
            OpenState::Complete => self.view_complete(),
            OpenState::Cancelled => self.view_input(), // Show input form when cancelled
            OpenState::Error(error) => self.view_error(error),
        }
    }

    /// Render the input form
    fn view_input(&self) -> Element<'_, OpenRepositoryMessage> {
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
    fn view_header(&self) -> Element<'_, OpenRepositoryMessage> {
        column![
            iced::widget::svg(theme::ziplock_logo())
                .width(Length::Fixed(64.0))
                .height(Length::Fixed(64.0)),
            Space::with_height(Length::Fixed(20.0)),
            text("Open Repository")
                .size(crate::ui::theme::utils::typography::extra_large_text_size())
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::with_height(Length::Fixed(10.0)),
            text("Select your repository file and enter your passphrase to unlock it.")
                .size(crate::ui::theme::utils::typography::normal_text_size())
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Render the file selection section
    fn view_file_selection(&self) -> Element<'_, OpenRepositoryMessage> {
        let file_display = if let Some(ref path) = self.selected_file {
            text(format!(
                "Selected: {}",
                path.file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                    .to_string_lossy()
            ))
            .size(crate::ui::theme::utils::typography::normal_text_size())
            .style(iced::theme::Text::Color(theme::SUCCESS_GREEN))
        } else {
            text("No file selected")
                .size(crate::ui::theme::utils::typography::normal_text_size())
                .style(iced::theme::Text::Color(MEDIUM_GRAY))
        };

        column![
            text("Repository File")
                .size(crate::ui::theme::utils::typography::medium_text_size())
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
    fn view_passphrase_input(&self) -> Element<'_, OpenRepositoryMessage> {
        let passphrase_input = text_input("Enter your passphrase", &self.passphrase)
            .on_input(OpenRepositoryMessage::PassphraseChanged)
            .on_submit(OpenRepositoryMessage::OpenRepository)
            .secure(!self.show_passphrase)
            .style(self.get_passphrase_style())
            .padding(utils::text_input_padding())
            .size(crate::ui::theme::utils::typography::text_input_size())
            .width(Length::Fill);

        let toggle_button = utils::password_visibility_toggle(
            self.show_passphrase,
            OpenRepositoryMessage::TogglePassphraseVisibility,
        );

        column![
            text("Master Passphrase")
                .size(crate::ui::theme::utils::typography::medium_text_size())
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
    fn view_navigation(&self) -> Element<'_, OpenRepositoryMessage> {
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
    fn view_opening(&self) -> Element<'_, OpenRepositoryMessage> {
        container(
            column![
                iced::widget::svg(theme::ziplock_logo())
                    .width(Length::Fixed(64.0))
                    .height(Length::Fixed(64.0)),
                Space::with_height(Length::Fixed(20.0)),
                text("Opening Repository...")
                    .size(crate::ui::theme::utils::typography::header_text_size())
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(10.0)),
                text("Please wait while we unlock your repository.")
                    .size(crate::ui::theme::utils::typography::normal_text_size())
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
    fn view_complete(&self) -> Element<'_, OpenRepositoryMessage> {
        container(
            column![
                text("✅")
                    .size(48.0)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text("Repository Opened")
                    .size(crate::ui::theme::utils::typography::header_text_size())
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(10.0)),
                text("Your repository has been successfully opened and unlocked.")
                    .size(crate::ui::theme::utils::typography::normal_text_size())
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
    fn view_error(&self, error: &str) -> Element<'_, OpenRepositoryMessage> {
        container(
            column![
                text("❌")
                    .size(48.0)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                text("Failed to Open Repository")
                    .size(crate::ui::theme::utils::typography::header_text_size())
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                Space::with_height(Length::Fixed(20.0)),
                container(
                    text(error)
                        .size(crate::ui::theme::utils::typography::normal_text_size())
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                )
                .style(container_styles::error_alert())
                .padding(utils::alert_padding()),
                Space::with_height(Length::Fixed(30.0)),
                button("Try Again")
                    .on_press(OpenRepositoryMessage::TryAgain)
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
            theme::text_input_styles::standard()
        } else {
            // For opening, we don't validate strength, just that it's not empty
            // This giving visual feedback that something has been entered
            theme::text_input_styles::neutral()
        }
    }

    /// Get the session ID if repository was opened successfully
    pub fn session_id(&self) -> Option<&String> {
        if self.is_complete() {
            self.session_id.as_ref()
        } else {
            None
        }
    }

    /// Reset the view to initial state
    pub fn reset(&mut self) {
        self.state = OpenState::Input;
        self.selected_file = None;
        self.passphrase.clear();
        self.show_passphrase = false;
        self.can_open = false;
        self.session_id = None;
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
    async fn open_repository_async(
        archive_path: PathBuf,
        master_password: String,
    ) -> Result<String, String> {
        // Connect to backend
        let mut client = ziplock_shared::ZipLockClient::new().map_err(|e| e.to_string())?;

        client.connect().await.map_err(|e| e.to_string())?;

        // Create a session first (required for database operations)
        let session_id = client.create_session().await.map_err(|e| e.to_string())?;

        // Attempt to open the archive
        client
            .open_archive(archive_path, master_password)
            .await
            .map_err(|e| e.to_string())?;

        // Return the session ID for later use
        Ok(session_id)
    }
}

// PassphraseTextInputStyle removed - now using centralized theme::text_input_styles
