//! Repository Creation Wizard for ZipLock Linux App
//!
//! This module contains the wizard implementation that guides users through
//! creating their first password repository (encrypted zip file).

use iced::{
    widget::{
        button, column, container, progress_bar, row, scrollable, svg, text, text_input, Space,
    },
    Alignment, Color, Command, Element, Length,
};
use rfd::AsyncFileDialog;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use crate::platform::LinuxFileOperationsHandler;
use crate::ui::theme::{utils, LIGHT_GRAY_TEXT, MEDIUM_GRAY, WARNING_YELLOW};
use crate::ui::{button_styles, progress_bar_styles, theme};
use ziplock_shared::{PassphraseValidator, ValidationUtils};

/// Helper function to get theme color for strength level
fn get_strength_color(level: &ziplock_shared::StrengthLevel) -> Color {
    match level {
        ziplock_shared::StrengthLevel::VeryWeak => theme::ERROR_RED,
        ziplock_shared::StrengthLevel::Weak => theme::ERROR_RED,
        ziplock_shared::StrengthLevel::Fair => WARNING_YELLOW,
        ziplock_shared::StrengthLevel::Good => theme::SUCCESS_GREEN,
        ziplock_shared::StrengthLevel::Strong => theme::SUCCESS_GREEN,
        ziplock_shared::StrengthLevel::VeryStrong => theme::LOGO_PURPLE,
    }
}

/// Messages for the wizard
#[derive(Debug, Clone)]
pub enum WizardMessage {
    // Navigation
    NextStep,
    PreviousStep,
    Cancel,

    // Start wizard
    StartWizard,

    // Directory selection
    SelectDirectory,
    DirectorySelected(Option<PathBuf>),
    DirectoryPathChanged(String),

    // Repository details
    RepositoryNameChanged(String),

    // Passphrase setup
    PassphraseChanged(String),
    ConfirmPassphraseChanged(String),
    TogglePassphraseVisibility,
    FocusNextField,

    // Repository creation
    CreateRepository,
    CreationProgress(f32),
    CreationComplete(Result<(), String>),

    // Completion
    Finish,
}

/// Wizard steps
#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep {
    Welcome,
    DirectorySelection,
    RepositoryDetails,
    PassphraseSetup,
    Creating,
    Complete,
}

/// Wizard state
#[derive(Debug)]
pub struct RepositoryWizard {
    current_step: WizardStep,

    // Directory selection
    selected_directory: Option<PathBuf>,
    directory_path_input: String,

    // Repository details
    repository_name: String,

    // Passphrase
    passphrase: String,
    confirm_passphrase: String,
    show_passphrase: bool,
    passphrase_validator: PassphraseValidator,

    // Creation progress
    creation_progress: f32,
    creation_error: Option<String>,

    // State
    is_loading: bool,
    can_proceed: bool,
    cancelled: bool,
}

impl Default for RepositoryWizard {
    fn default() -> Self {
        Self {
            current_step: WizardStep::Welcome,
            selected_directory: None,
            directory_path_input: String::new(),
            repository_name: "ZipLock".to_string(),
            passphrase: String::new(),
            confirm_passphrase: String::new(),
            show_passphrase: false,
            passphrase_validator: ValidationUtils::for_creation(),
            creation_progress: 0.0,
            creation_error: None,
            is_loading: false,
            can_proceed: false,
            cancelled: false,
        }
    }
}

impl RepositoryWizard {
    /// Create a new repository wizard
    pub fn new() -> Self {
        Self {
            can_proceed: true, // Enable the Get Started button on welcome screen
            ..Self::default()
        }
    }

    /// Update wizard state based on message
    pub fn update(&mut self, message: WizardMessage) -> Command<WizardMessage> {
        match message {
            WizardMessage::StartWizard => {
                debug!("Starting repository wizard");
                self.advance_step();
                Command::none()
            }

            WizardMessage::NextStep => {
                self.advance_step();
                // Auto-focus first input field on each step
                match self.current_step {
                    WizardStep::DirectorySelection => {
                        text_input::focus(text_input::Id::new("directory_path"))
                    }
                    WizardStep::RepositoryDetails => {
                        text_input::focus(text_input::Id::new("repository_name"))
                    }
                    WizardStep::PassphraseSetup => {
                        text_input::focus(text_input::Id::new("master_passphrase"))
                    }
                    _ => Command::none(),
                }
            }

            WizardMessage::PreviousStep => {
                self.previous_step();
                Command::none()
            }

            WizardMessage::Cancel => {
                debug!("Wizard cancelled by user");
                // Mark as cancelled
                self.cancelled = true;
                Command::none()
            }

            WizardMessage::SelectDirectory => {
                self.is_loading = true;
                Command::perform(
                    Self::select_directory_async(),
                    WizardMessage::DirectorySelected,
                )
            }

            WizardMessage::DirectorySelected(directory) => {
                self.is_loading = false;
                if let Some(dir) = directory {
                    self.selected_directory = Some(dir.clone());
                    self.directory_path_input = dir.to_string_lossy().to_string();
                    info!("Directory selected: {:?}", dir);
                }
                self.update_can_proceed();
                Command::none()
            }

            WizardMessage::DirectoryPathChanged(path) => {
                self.directory_path_input = path.clone();
                let path_buf = PathBuf::from(path);
                if path_buf.exists() && path_buf.is_dir() {
                    self.selected_directory = Some(path_buf);
                } else {
                    self.selected_directory = None;
                }
                self.update_can_proceed();
                Command::none()
            }

            WizardMessage::RepositoryNameChanged(name) => {
                self.repository_name = name;
                self.update_can_proceed();
                Command::none()
            }

            WizardMessage::PassphraseChanged(passphrase) => {
                self.passphrase = passphrase;
                self.update_can_proceed();
                Command::none()
            }

            WizardMessage::ConfirmPassphraseChanged(confirm) => {
                self.confirm_passphrase = confirm;
                self.update_can_proceed();
                Command::none()
            }

            WizardMessage::TogglePassphraseVisibility => {
                self.show_passphrase = !self.show_passphrase;
                Command::none()
            }

            WizardMessage::FocusNextField => {
                // Focus the confirm passphrase field
                text_input::focus(text_input::Id::new("confirm_passphrase"))
            }

            WizardMessage::CreateRepository => {
                if self.can_create_repository() {
                    self.current_step = WizardStep::Creating;
                    self.creation_progress = 0.0;
                    Command::perform(
                        Self::create_repository_async(
                            self.selected_directory.as_ref().unwrap().clone(),
                            self.repository_name.clone(),
                            self.passphrase.clone(),
                        ),
                        WizardMessage::CreationComplete,
                    )
                } else {
                    warn!("Attempted to create repository with invalid settings");
                    Command::none()
                }
            }

            WizardMessage::CreationProgress(progress) => {
                self.creation_progress = progress;
                Command::none()
            }

            WizardMessage::CreationComplete(result) => {
                match result {
                    Ok(()) => {
                        info!("Repository created successfully");
                        self.current_step = WizardStep::Complete;
                        self.creation_error = None;
                    }
                    Err(error) => {
                        error!("Failed to create repository: {}", error);
                        self.creation_error = Some(error);
                        // Stay on creation step to show error
                    }
                }
                Command::none()
            }

            WizardMessage::Finish => {
                info!("Wizard completed successfully");
                // This should trigger the parent to close the wizard
                Command::none()
            }
        }
    }

    /// Render the wizard UI
    pub fn view(&self) -> Element<'_, WizardMessage> {
        let content = match &self.current_step {
            WizardStep::Welcome => self.view_welcome(),
            WizardStep::DirectorySelection => self.view_directory_selection(),
            WizardStep::RepositoryDetails => self.view_repository_details(),
            WizardStep::PassphraseSetup => self.view_passphrase_setup(),
            WizardStep::Creating => self.view_creating(),
            WizardStep::Complete => self.view_complete(),
        };

        let scrollable_content = scrollable(column![
            Space::with_height(Length::Fixed(40.0)), // Top padding for centering effect
            container(
                column![
                    self.view_header(),
                    content,
                    Space::with_height(Length::Fixed(20.0)),
                    self.view_navigation(),
                ]
                .spacing(20)
                .padding(30)
                .max_width(600),
            )
            .width(Length::Fill)
            .center_x(),
            Space::with_height(Length::Fixed(40.0)), // Bottom padding for centering effect
        ])
        .width(Length::Fill)
        .height(Length::Fill);

        scrollable_content.into()
    }

    /// View the wizard header with step indicator
    fn view_header(&self) -> Element<'_, WizardMessage> {
        let step_number = match self.current_step {
            WizardStep::Welcome => 0,
            WizardStep::DirectorySelection => 1,
            WizardStep::RepositoryDetails => 2,
            WizardStep::PassphraseSetup => 3,
            WizardStep::Creating => 4,
            WizardStep::Complete => 5,
        };

        let total_steps = 5;
        let progress = if step_number == 0 {
            0.0
        } else {
            step_number as f32 / total_steps as f32
        };

        column![
            row![
                // ZipLock logo
                svg(theme::ziplock_logo())
                    .width(Length::Fixed(32.0))
                    .height(Length::Fixed(32.0)),
                Space::with_width(Length::Fixed(10.0)),
                text("ZipLock Repository Setup")
                    .size(crate::ui::theme::utils::typography::extra_large_text_size()),
            ]
            .align_items(Alignment::Center),
            Space::with_height(Length::Fixed(10.0)),
            progress_bar(0.0..=1.0, progress).height(Length::Fixed(4.0)),
            Space::with_height(Length::Fixed(10.0)),
        ]
        .into()
    }

    /// View welcome step
    fn view_welcome(&self) -> Element<'_, WizardMessage> {
        column![
            // Large ZipLock logo for welcome screen
            svg(theme::ziplock_logo())
                .width(Length::Fixed(80.0))
                .height(Length::Fixed(80.0)),
            Space::with_height(Length::Fixed(20.0)),
            text("Welcome to ZipLock!").size(crate::ui::theme::utils::typography::header_text_size()),
            Space::with_height(Length::Fixed(20.0)),
            text("This wizard will help you create your first password repository.")
                .size(crate::ui::theme::utils::typography::medium_text_size()),
            Space::with_height(Length::Fixed(10.0)),
            text("Your repository is a secure, encrypted file that stores all your passwords and sensitive information.")
                .size(crate::ui::theme::utils::typography::normal_text_size()),
            Space::with_height(Length::Fixed(10.0)),
            text("You can store this file anywhere - on your computer, in cloud storage, or on a USB drive.")
                .size(crate::ui::theme::utils::typography::normal_text_size()),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// View directory selection step
    fn view_directory_selection(&self) -> Element<'_, WizardMessage> {
        column![
            text("Choose Repository Location").size(crate::ui::theme::utils::typography::large_text_size()),
            Space::with_height(Length::Fixed(15.0)),
            text("Where would you like to store your password repository?").size(crate::ui::theme::utils::typography::normal_text_size()),
            Space::with_height(Length::Fixed(20.0)),

            row![
                text_input("Repository directory...", &self.directory_path_input)
                    .on_input(WizardMessage::DirectoryPathChanged)
                    .width(Length::Fill)
                    .id(text_input::Id::new("directory_path"))
                    .on_submit(WizardMessage::NextStep)
                    .padding(theme::utils::text_input_padding())
                    .style(theme::text_input_styles::standard())
                    .size(crate::ui::theme::utils::typography::text_input_size()),
                button("Browse...")
                    .on_press(WizardMessage::SelectDirectory)
                    .padding(utils::button_padding())
                    .style(button_styles::secondary()),
            ]
            .spacing(10)
            .align_items(Alignment::Center),

            Space::with_height(Length::Fixed(10.0)),

            if let Some(dir) = &self.selected_directory {
                text(format!("Selected: {}", dir.display()))
                    .size(crate::ui::theme::utils::typography::small_text_size())

            } else if !self.directory_path_input.is_empty() {
                text("Directory does not exist or is not accessible")
                    .size(crate::ui::theme::utils::typography::small_text_size())

            } else {
                text("No directory selected")
                    .size(crate::ui::theme::utils::typography::small_text_size())

            },

            Space::with_height(Length::Fixed(15.0)),
            text("💡 Tip: You can store your repository in cloud storage (Dropbox, Google Drive, etc.) to access it from multiple devices.")
                .size(crate::ui::theme::utils::typography::small_text_size())
                ,
        ]
        .align_items(Alignment::Start)
        .into()
    }

    /// View repository details step
    fn view_repository_details(&self) -> Element<'_, WizardMessage> {
        column![
            text("Repository Details").size(crate::ui::theme::utils::typography::large_text_size()),
            Space::with_height(Length::Fixed(15.0)),
            column![
                text("Repository Name")
                    .size(crate::ui::theme::utils::typography::normal_text_size()),
                Space::with_height(Length::Fixed(5.0)),
                text_input("Enter a name for your repository", &self.repository_name)
                    .on_input(WizardMessage::RepositoryNameChanged)
                    .width(Length::Fill)
                    .id(text_input::Id::new("repository_name"))
                    .on_submit(WizardMessage::NextStep)
                    .padding(theme::utils::text_input_padding())
                    .style(theme::text_input_styles::standard())
                    .size(crate::ui::theme::utils::typography::text_input_size()),
            ]
            .spacing(5),
            Space::with_height(Length::Fixed(20.0)),
            if let Some(dir) = &self.selected_directory {
                let file_path = dir.join(format!("{}.7z", self.repository_name));
                column![
                    text("Repository will be created as:")
                        .size(crate::ui::theme::utils::typography::small_text_size()),
                    text(file_path.display().to_string())
                        .size(crate::ui::theme::utils::typography::small_text_size()),
                ]
            } else {
                column![]
            },
        ]
        .align_items(Alignment::Start)
        .into()
    }

    /// View passphrase setup step
    fn view_passphrase_setup(&self) -> Element<'_, WizardMessage> {
        let passphrase_strength = self.passphrase_validator.validate(&self.passphrase);
        let passphrases_match =
            !self.confirm_passphrase.is_empty() && self.passphrase == self.confirm_passphrase;

        column![
            text("Set Master Passphrase").size(crate::ui::theme::utils::typography::large_text_size()),
            Space::with_height(Length::Fixed(15.0)),

            text("Your master passphrase protects your entire repository. Choose a strong, memorable passphrase.")
                .size(crate::ui::theme::utils::typography::normal_text_size()),

            Space::with_height(Length::Fixed(20.0)),

            column![
                text("Master Passphrase").size(crate::ui::theme::utils::typography::normal_text_size()),
                Space::with_height(Length::Fixed(5.0)),
                text_input("Enter your master passphrase", &self.passphrase)
                    .on_input(WizardMessage::PassphraseChanged)
                    .secure(!self.show_passphrase)
                    .width(Length::Fill)
                    .padding(theme::utils::text_input_padding())
                    .style(self.get_passphrase_style())
                    .size(crate::ui::theme::utils::typography::text_input_size())
                    .id(text_input::Id::new("master_passphrase"))
                    .on_submit(WizardMessage::FocusNextField),

                Space::with_height(Length::Fixed(5.0)),

                row![
                    text(format!("Strength: {}", passphrase_strength.level.as_str()))
                        .size(crate::ui::theme::utils::typography::small_text_size())
                        .style(iced::theme::Text::Color(get_strength_color(&passphrase_strength.level))),
                    Space::with_width(Length::Fill),
                    utils::password_visibility_toggle(
                        self.show_passphrase,
                        WizardMessage::TogglePassphraseVisibility,
                    ),
                ]
                .align_items(Alignment::Center),
            ]
            .spacing(5),

            Space::with_height(Length::Fixed(15.0)),

            column![
                text("Confirm Passphrase").size(crate::ui::theme::utils::typography::normal_text_size()),
                Space::with_height(Length::Fixed(5.0)),
                text_input("Confirm your master passphrase", &self.confirm_passphrase)
                    .on_input(WizardMessage::ConfirmPassphraseChanged)
                    .secure(!self.show_passphrase)
                    .width(Length::Fill)
                    .padding(theme::utils::text_input_padding())
                    .style(self.get_confirm_passphrase_style())
                    .size(crate::ui::theme::utils::typography::text_input_size())
                    .id(text_input::Id::new("confirm_passphrase"))
                    .on_submit(WizardMessage::CreateRepository),

                Space::with_height(Length::Fixed(5.0)),

                if !self.confirm_passphrase.is_empty() {
                    if passphrases_match {
                        text("✓ Passphrases match")
                            .size(crate::ui::theme::utils::typography::small_text_size())
                    } else {
                        text("✗ Passphrases do not match")
                            .size(crate::ui::theme::utils::typography::small_text_size())
                    }
                } else {
                    text("")
                },
            ]
            .spacing(5),

            Space::with_height(Length::Fixed(15.0)),

            // Passphrase requirements and validation feedback
            if !self.passphrase.is_empty() {
                column![
                    text("Passphrase Requirements:").size(crate::ui::theme::utils::typography::small_text_size()).style(iced::theme::Text::Color(MEDIUM_GRAY)),
                    Space::with_height(Length::Fixed(5.0)),

                    // Show violations if any
                    if !passphrase_strength.violations.is_empty() {
                        column(
                            passphrase_strength.violations
                                .iter()
                                .map(|violation| {
                                    row![
                                        text("✗").style(iced::theme::Text::Color(theme::ERROR_RED)),
                                        Space::with_width(Length::Fixed(5.0)),
                                        text(violation).size(crate::ui::theme::utils::typography::small_text_size()).style(iced::theme::Text::Color(theme::ERROR_RED))
                                    ].into()
                                })
                                .collect::<Vec<Element<WizardMessage>>>()
                        ).spacing(3)
                    } else {
                        column![]
                    },

                    // Show satisfied requirements
                    if !passphrase_strength.satisfied.is_empty() {
                        column(
                            passphrase_strength.satisfied
                                .iter()
                                .map(|satisfied| {
                                    row![
                                        text("✓").style(iced::theme::Text::Color(theme::SUCCESS_GREEN)),
                                        Space::with_width(Length::Fixed(5.0)),
                                        text(satisfied).size(crate::ui::theme::utils::typography::small_text_size()).style(iced::theme::Text::Color(theme::SUCCESS_GREEN))
                                    ].into()
                                })
                                .collect::<Vec<Element<WizardMessage>>>()
                        ).spacing(3)
                    } else {
                        column![]
                    },
                ]
                .spacing(8)
            } else {
                column![
                    text("Passphrase Requirements:").size(crate::ui::theme::utils::typography::small_text_size()).style(iced::theme::Text::Color(MEDIUM_GRAY)),
                    Space::with_height(Length::Fixed(5.0)),
                    text("• At least 12 characters long").size(crate::ui::theme::utils::typography::small_text_size()).style(iced::theme::Text::Color(LIGHT_GRAY_TEXT)),
                    text("• Contains uppercase letters").size(crate::ui::theme::utils::typography::small_text_size()).style(iced::theme::Text::Color(LIGHT_GRAY_TEXT)),
                    text("• Contains lowercase letters").size(crate::ui::theme::utils::typography::small_text_size()).style(iced::theme::Text::Color(LIGHT_GRAY_TEXT)),
                    text("• Contains numbers").size(crate::ui::theme::utils::typography::small_text_size()).style(iced::theme::Text::Color(LIGHT_GRAY_TEXT)),
                    text("• Contains special characters").size(crate::ui::theme::utils::typography::small_text_size()).style(iced::theme::Text::Color(LIGHT_GRAY_TEXT)),
                ]
                .spacing(3)
            },

            Space::with_height(Length::Fixed(20.0)),

            text("⚠️ Important: There is no way to recover your repository if you forget your master passphrase. Write it down and keep it safe!")
                .size(crate::ui::theme::utils::typography::small_text_size()),
        ]
        .align_items(Alignment::Start)
        .into()
    }

    /// View creation progress step
    fn view_creating(&self) -> Element<'_, WizardMessage> {
        if let Some(error) = &self.creation_error {
            column![
                text("Repository Creation Failed")
                    .size(crate::ui::theme::utils::typography::large_text_size()),
                Space::with_height(Length::Fixed(20.0)),
                container(
                    column![
                        text("❌ Creation Failed")
                            .size(crate::ui::theme::utils::typography::medium_text_size())
                            .style(iced::theme::Text::Color(theme::ERROR_RED)),
                        Space::with_height(Length::Fixed(8.0)),
                        text(error)
                            .size(crate::ui::theme::utils::typography::small_text_size())
                            .style(iced::theme::Text::Color(theme::DARK_TEXT)),
                    ]
                    .spacing(4)
                )
                .padding(utils::error_container_padding())
                .width(Length::Fill)
                .style(crate::ui::theme::container_styles::error_alert()),
                Space::with_height(Length::Fixed(20.0)),
                button("Try Again")
                    .on_press(WizardMessage::CreateRepository)
                    .padding(utils::standard_button_padding())
                    .style(button_styles::primary()),
            ]
            .align_items(Alignment::Center)
            .into()
        } else {
            column![
                text("Creating Repository...")
                    .size(crate::ui::theme::utils::typography::large_text_size()),
                Space::with_height(Length::Fixed(30.0)),
                progress_bar(0.0..=1.0, self.creation_progress)
                    .height(Length::Fixed(20.0))
                    .style(progress_bar_styles::primary()),
                Space::with_height(Length::Fixed(10.0)),
                text(format!("{}%", (self.creation_progress * 100.0) as u32))
                    .size(crate::ui::theme::utils::typography::normal_text_size()),
                Space::with_height(Length::Fixed(20.0)),
                text("Setting up encrypted archive structure...")
                    .size(crate::ui::theme::utils::typography::small_text_size()),
            ]
            .align_items(Alignment::Center)
            .into()
        }
    }

    /// View completion step
    fn view_complete(&self) -> Element<'_, WizardMessage> {
        column![
            text("✓ Repository Created Successfully!").size(crate::ui::theme::utils::typography::header_text_size()),
            Space::with_height(Length::Fixed(30.0)),

            if let Some(dir) = &self.selected_directory {
                let repo_path = dir.join(format!("{}.7z", self.repository_name));
                column![
                    text("Your password repository has been created:").size(crate::ui::theme::utils::typography::normal_text_size()),
                    Space::with_height(Length::Fixed(10.0)),
                    text(repo_path.display().to_string())
                        .size(crate::ui::theme::utils::typography::small_text_size()),
                ]
            } else {
                column![]
            },

            Space::with_height(Length::Fixed(30.0)),

            text("You can now start adding your passwords and sensitive information to your secure repository.")
                .size(crate::ui::theme::utils::typography::normal_text_size()),

            Space::with_height(Length::Fixed(20.0)),

            button("Start Using ZipLock")
                .on_press(WizardMessage::Finish)
                .padding(utils::completion_button_padding()),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// View navigation buttons
    fn view_navigation(&self) -> Element<'_, WizardMessage> {
        let can_go_back = !matches!(
            self.current_step,
            WizardStep::Welcome | WizardStep::Creating | WizardStep::Complete
        );

        let show_next_button = !matches!(
            self.current_step,
            WizardStep::Creating | WizardStep::Complete
        );

        let can_go_next = self.can_proceed && show_next_button;

        row![
            if can_go_back {
                button("Back")
                    .on_press(WizardMessage::PreviousStep)
                    .padding(utils::button_padding())
                    .style(button_styles::secondary())
            } else {
                button("Cancel")
                    .on_press(WizardMessage::Cancel)
                    .padding(utils::button_padding())
                    .style(button_styles::destructive())
            },
            Space::with_width(Length::Fill),
            if show_next_button {
                let label = match self.current_step {
                    WizardStep::Welcome => "Get Started",
                    WizardStep::PassphraseSetup => "Create Repository",
                    _ => "Next",
                };

                self.create_next_button(label, can_go_next)
            } else {
                Space::with_width(Length::Shrink).into()
            }
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Advance to the next step
    /// Go to the next step
    fn advance_step(&mut self) {
        self.current_step = match self.current_step {
            WizardStep::Welcome => WizardStep::DirectorySelection,
            WizardStep::DirectorySelection => WizardStep::RepositoryDetails,
            WizardStep::RepositoryDetails => WizardStep::PassphraseSetup,
            WizardStep::PassphraseSetup => WizardStep::Creating, // This should be handled by CreateRepository
            WizardStep::Creating => WizardStep::Complete,
            WizardStep::Complete => WizardStep::Complete, // Stay here
        };
        self.update_can_proceed();
    }

    /// Go to the previous step
    fn previous_step(&mut self) {
        self.current_step = match self.current_step {
            WizardStep::DirectorySelection => WizardStep::Welcome,
            WizardStep::RepositoryDetails => WizardStep::DirectorySelection,
            WizardStep::PassphraseSetup => WizardStep::RepositoryDetails,
            WizardStep::Creating => WizardStep::PassphraseSetup,
            WizardStep::Complete => WizardStep::PassphraseSetup,
            WizardStep::Welcome => WizardStep::Welcome, // Stay here
        };
        self.update_can_proceed();
    }

    /// Update whether we can proceed to the next step
    fn update_can_proceed(&mut self) {
        self.can_proceed = match self.current_step {
            WizardStep::Welcome => true,
            WizardStep::DirectorySelection => self.selected_directory.is_some(),
            WizardStep::RepositoryDetails => !self.repository_name.trim().is_empty(),
            WizardStep::PassphraseSetup => self.can_create_repository(),
            WizardStep::Creating => false,
            WizardStep::Complete => true,
        };
    }

    /// Check if we can create the repository
    fn can_create_repository(&self) -> bool {
        let passphrase_valid = self
            .passphrase_validator
            .meets_requirements(&self.passphrase);
        let passphrases_match =
            !self.confirm_passphrase.is_empty() && self.passphrase == self.confirm_passphrase;

        passphrase_valid
            && passphrases_match
            && self.selected_directory.is_some()
            && !self.repository_name.trim().is_empty()
    }

    /// Async function to select directory
    async fn select_directory_async() -> Option<PathBuf> {
        let dialog = AsyncFileDialog::new()
            .set_title("Select Repository Directory")
            .set_directory(
                dirs::document_dir()
                    .or_else(dirs::home_dir)
                    .unwrap_or_else(|| PathBuf::from(".")),
            );

        dialog
            .pick_folder()
            .await
            .map(|folder| folder.path().to_path_buf())
    }

    /// Async function to create repository
    async fn create_repository_async(
        directory: PathBuf,
        name: String,
        passphrase: String,
    ) -> Result<(), String> {
        info!("Creating repository '{}' in {:?}", name, directory);

        let repo_path = directory.join(format!("{}.7z", name));

        // Validate directory is writable
        if !directory.exists() {
            return Err("Selected directory does not exist".to_string());
        }

        if !directory.is_dir() {
            return Err("Selected path is not a directory".to_string());
        }

        // Check if repository file already exists
        if repo_path.exists() {
            return Err(
                "A repository with this name already exists in the selected directory".to_string(),
            );
        }

        info!("Creating repository: {}", repo_path.display());

        // For now, bypass the hanging hybrid client and use direct external file operations
        // This prevents the async runtime conflicts that cause hanging
        info!("Using external file operations approach to avoid runtime conflicts");

        let mut file_handler = LinuxFileOperationsHandler::new();

        // Create file operations JSON for archive creation
        let file_operations = serde_json::json!({
            "operations": [
                {
                    "type": "create_archive",
                    "path": repo_path.to_string_lossy(),
                    "password": passphrase,
                    "format": "7z"
                }
            ]
        })
        .to_string();

        // Execute the file operations
        file_handler
            .execute_file_operations(&file_operations)
            .await
            .map_err(|e| {
                error!("Failed to execute file operations: {}", e);
                format!("Failed to create repository via file operations: {}", e)
            })?;

        // Log successful creation - avoid hybrid client initialization in async context
        // to prevent FFI deadlocks

        info!("Repository created successfully via external file operations");

        info!("Repository created successfully at {:?}", repo_path);
        Ok(())
    }

    /// Get the repository path that will be created
    pub fn repository_path(&self) -> Option<PathBuf> {
        self.selected_directory
            .as_ref()
            .map(|dir| dir.join(format!("{}.7z", self.repository_name)))
    }

    /// Check if the wizard is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.current_step, WizardStep::Complete)
    }

    /// Check if the wizard is in progress
    pub fn is_in_progress(&self) -> bool {
        !matches!(
            self.current_step,
            WizardStep::Welcome | WizardStep::Complete
        )
    }

    /// Check if the wizard was cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Helper function to create next button with proper typing
    fn create_next_button<'a>(&self, label: &'a str, enabled: bool) -> Element<'a, WizardMessage> {
        if enabled {
            button(label)
                .padding(utils::button_padding())
                .style(button_styles::primary())
                .on_press(if self.current_step == WizardStep::PassphraseSetup {
                    WizardMessage::CreateRepository
                } else {
                    WizardMessage::NextStep
                })
                .into()
        } else {
            button(label)
                .padding(utils::button_padding())
                .style(button_styles::disabled())
                .into()
        }
    }

    /// Get the style for the passphrase field
    fn get_passphrase_style(&self) -> iced::theme::TextInput {
        if self.passphrase.is_empty() {
            theme::text_input_styles::standard()
        } else {
            let strength = self.passphrase_validator.validate(&self.passphrase);
            if strength.meets_requirements && strength.level.is_acceptable() {
                // Green border for strong passphrase
                theme::text_input_styles::valid()
            } else {
                // Red border for weak passphrase
                theme::text_input_styles::invalid()
            }
        }
    }

    /// Get the style for the confirm passphrase field
    fn get_confirm_passphrase_style(&self) -> iced::theme::TextInput {
        if self.confirm_passphrase.is_empty() {
            theme::text_input_styles::standard()
        } else if !self.confirm_passphrase.is_empty() && self.passphrase == self.confirm_passphrase
        {
            // Green border when passphrases match
            theme::text_input_styles::valid()
        } else {
            // Red border when passphrases don't match
            theme::text_input_styles::invalid()
        }
    }
}
