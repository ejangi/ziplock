//! Repository Creation Wizard for ZipLock Linux App
//!
//! This module contains the wizard implementation that guides users through
//! creating their first password repository (encrypted zip file).

use iced::{
    widget::{
        button, column, container, progress_bar, row, scrollable, svg, text, text_input, Space,
    },
    Alignment, Color, Element, Length, Task,
};
use rfd::AsyncFileDialog;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use crate::ui::{
    components::button as btn,
    theme::{self, utils, WARNING_YELLOW},
};
use ziplock_shared::{PasswordAnalyzer, PasswordStrength};

/// Helper function to get theme color for strength level
fn get_strength_color(level: &PasswordStrength) -> Color {
    match level {
        PasswordStrength::VeryWeak => theme::ERROR_RED,
        PasswordStrength::Weak => theme::ERROR_RED,
        PasswordStrength::Fair => WARNING_YELLOW,
        PasswordStrength::Good => theme::SUCCESS_GREEN,
        PasswordStrength::Strong => theme::SUCCESS_GREEN,
        PasswordStrength::VeryStrong => theme::LOGO_PURPLE,
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
    pub fn update(&mut self, message: WizardMessage) -> Task<WizardMessage> {
        match message {
            WizardMessage::StartWizard => {
                debug!("Starting repository wizard");
                self.advance_step();
                Task::none()
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
                    _ => Task::none(),
                }
            }

            WizardMessage::PreviousStep => {
                self.previous_step();
                Task::none()
            }

            WizardMessage::Cancel => {
                debug!("Wizard cancelled by user");
                // Mark as cancelled
                self.cancelled = true;
                Task::none()
            }

            WizardMessage::SelectDirectory => {
                self.is_loading = true;
                Task::perform(
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
                Task::none()
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
                Task::none()
            }

            WizardMessage::RepositoryNameChanged(name) => {
                self.repository_name = name;
                self.update_can_proceed();
                Task::none()
            }

            WizardMessage::PassphraseChanged(passphrase) => {
                self.passphrase = passphrase;
                self.update_can_proceed();
                Task::none()
            }

            WizardMessage::ConfirmPassphraseChanged(confirm) => {
                debug!("Confirm passphrase changed (length: {})", confirm.len());
                self.confirm_passphrase = confirm;
                self.update_can_proceed();
                Task::none()
            }

            WizardMessage::TogglePassphraseVisibility => {
                self.show_passphrase = !self.show_passphrase;
                Task::none()
            }

            WizardMessage::FocusNextField => {
                // Focus the confirm passphrase field
                text_input::focus(text_input::Id::new("confirm_passphrase"))
            }

            WizardMessage::CreateRepository => {
                if self.can_create_repository() {
                    self.current_step = WizardStep::Creating;
                    self.creation_progress = 0.0;
                    Task::perform(
                        Self::create_repository_async(
                            self.selected_directory.as_ref().unwrap().clone(),
                            self.repository_name.clone(),
                            self.passphrase.clone(),
                        ),
                        WizardMessage::CreationComplete,
                    )
                } else {
                    warn!("Attempted to create repository with invalid settings");
                    Task::none()
                }
            }

            WizardMessage::CreationProgress(progress) => {
                self.creation_progress = progress;
                Task::none()
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
                Task::none()
            }

            WizardMessage::Finish => {
                info!("Wizard completed successfully");
                // This should trigger the parent to close the wizard
                Task::none()
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
            .center_x(Length::Fill),
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
            .align_y(Alignment::Center),
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
        .align_x(Alignment::Center)
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
                    .size(crate::ui::theme::utils::typography::text_input_size())
                    .style(theme::text_input_styles::standard()),
                btn::presets::browse_button(Some(WizardMessage::SelectDirectory)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),

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
        .align_x(Alignment::Start)
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
                    .size(crate::ui::theme::utils::typography::text_input_size())
                    .style(theme::text_input_styles::standard()),
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
        .align_x(Alignment::Start)
        .into()
    }

    /// View passphrase setup step
    fn view_passphrase_setup(&self) -> Element<'_, WizardMessage> {
        let passphrase_analysis = PasswordAnalyzer::analyze(&self.passphrase);
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
                    .size(crate::ui::theme::utils::typography::text_input_size())
                    .style(theme::text_input_styles::standard())
                    .id(text_input::Id::new("master_passphrase"))
                    .on_submit(WizardMessage::FocusNextField),

                Space::with_height(Length::Fixed(5.0)),

                row![
                    text(format!("Strength: {:?}", passphrase_analysis.strength))
                        .size(crate::ui::theme::utils::typography::small_text_size())
                        .color(get_strength_color(&passphrase_analysis.strength)),
                    Space::with_width(Length::Fill),
                    theme::utils::password_visibility_toggle(
                        self.show_passphrase,
                        WizardMessage::TogglePassphraseVisibility
                    ),
                ]
                .align_y(Alignment::Center),
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
                    .size(crate::ui::theme::utils::typography::text_input_size())
                    .style(theme::text_input_styles::standard())
                    .id(text_input::Id::new("confirm_passphrase"))
                    .on_submit(WizardMessage::CreateRepository),

                Space::with_height(Length::Fixed(5.0)),

                if !self.confirm_passphrase.is_empty() {
                    if passphrases_match {
                        text("✓ Passphrases match")
                            .size(crate::ui::theme::utils::typography::small_text_size())
                            .color(theme::SUCCESS_GREEN)
                    } else {
                        text("✗ Passphrases do not match")
                            .size(crate::ui::theme::utils::typography::small_text_size())
                            .color(theme::ERROR_RED)
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
                    text("Passphrase Requirements:").size(crate::ui::theme::utils::typography::small_text_size()),
                    Space::with_height(Length::Fixed(5.0)),

                    // Show feedback if any
                    if !passphrase_analysis.feedback.is_empty() {
                        column(
                            passphrase_analysis.feedback
                                .iter()
                                .cloned()
                                .map(|feedback_msg| {
                                    row![
                                        text("ℹ"),
                                        Space::with_width(Length::Fixed(5.0)),
                                        text(feedback_msg).size(crate::ui::theme::utils::typography::small_text_size())
                                    ].into()
                                })
                                .collect::<Vec<_>>()
                        )
                        .spacing(2)
                    } else {
                        column![]
                    },

                    // Show score and entropy info
                    row![
                        text(format!("Score: {}/100", passphrase_analysis.score))
                            .size(crate::ui::theme::utils::typography::small_text_size()),
                        Space::with_width(Length::Fixed(10.0)),
                        text(format!("Entropy: {:.1} bits", passphrase_analysis.entropy))
                            .size(crate::ui::theme::utils::typography::small_text_size())
                    ].align_y(Alignment::Center),
                ]
                .spacing(8)
            } else {
                column![
                    text("Passphrase Requirements:").size(crate::ui::theme::utils::typography::small_text_size()),
                    Space::with_height(Length::Fixed(5.0)),
                    text("• At least 12 characters long").size(crate::ui::theme::utils::typography::small_text_size()),
                    text("• Contains uppercase letters").size(crate::ui::theme::utils::typography::small_text_size()),
                    text("• Contains lowercase letters").size(crate::ui::theme::utils::typography::small_text_size()),
                    text("• Contains numbers").size(crate::ui::theme::utils::typography::small_text_size()),
                    text("• Contains special characters").size(crate::ui::theme::utils::typography::small_text_size()),
                ]
                .spacing(3)
            },

            Space::with_height(Length::Fixed(20.0)),

            text("⚠️ Important: There is no way to recover your repository if you forget your master passphrase. Write it down and keep it safe!")
                .size(crate::ui::theme::utils::typography::small_text_size()),
        ]
        .align_x(Alignment::Center)
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
                            .size(crate::ui::theme::utils::typography::medium_text_size()),
                        Space::with_height(Length::Fixed(8.0)),
                        text(error).size(crate::ui::theme::utils::typography::small_text_size()),
                    ]
                    .spacing(4)
                )
                .padding(utils::error_container_padding())
                .width(Length::Fill),
                Space::with_height(Length::Fixed(20.0)),
                btn::presets::try_again_button(Some(WizardMessage::CreateRepository)),
            ]
            .align_x(Alignment::Center)
            .into()
        } else {
            column![
                text("Creating Repository...")
                    .size(crate::ui::theme::utils::typography::large_text_size()),
                Space::with_height(Length::Fixed(30.0)),
                progress_bar(0.0..=1.0, self.creation_progress).height(Length::Fixed(20.0)),
                Space::with_height(Length::Fixed(10.0)),
                text(format!("{}%", (self.creation_progress * 100.0) as u32))
                    .size(crate::ui::theme::utils::typography::normal_text_size()),
                Space::with_height(Length::Fixed(20.0)),
                text("Setting up encrypted archive structure...")
                    .size(crate::ui::theme::utils::typography::small_text_size()),
            ]
            .align_x(Alignment::Center)
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

            btn::presets::start_using_button(Some(WizardMessage::Finish)),
        ]
        .align_x(Alignment::Center)
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
                btn::presets::back_button(Some(WizardMessage::PreviousStep))
            } else {
                btn::presets::cancel_button(Some(WizardMessage::Cancel))
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
        .align_y(Alignment::Center)
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
        let analysis = PasswordAnalyzer::analyze(&self.passphrase);
        let passphrase_valid = matches!(
            analysis.strength,
            PasswordStrength::Good | PasswordStrength::Strong | PasswordStrength::VeryStrong
        );
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

        // Use the unified repository service instead of legacy file operations
        info!("Using unified repository service for repository creation");

        let repository_service = crate::services::get_repository_service();

        // Create repository using unified architecture
        repository_service
            .create_repository(repo_path.to_string_lossy().to_string(), passphrase)
            .await
            .map_err(|e| {
                error!("Failed to create repository: {}", e);
                format!("Failed to create repository: {}", e)
            })?;

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
            match label {
                "Get Started" => btn::presets::get_started_button(Some(WizardMessage::NextStep)),
                "Create Repository" => {
                    btn::presets::create_repository_button(Some(WizardMessage::CreateRepository))
                }
                _ => btn::presets::next_button(Some(
                    if self.current_step == WizardStep::PassphraseSetup {
                        WizardMessage::CreateRepository
                    } else {
                        WizardMessage::NextStep
                    },
                )),
            }
        } else {
            match label {
                "Get Started" => btn::presets::get_started_button(None),
                "Create Repository" => btn::presets::create_repository_button(None),
                _ => btn::presets::next_button(None),
            }
        }
    }
}
