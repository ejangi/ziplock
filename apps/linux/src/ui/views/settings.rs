use iced::{
    widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};

use std::path::PathBuf;
use tracing::info;

use crate::ui::theme::{self, button_styles, text_input_styles, utils};
use ziplock_shared::config::{AppConfig, FrontendConfig, RepositoryConfig, UiConfig};

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    // Tab navigation
    SelectTab(SettingsTab),

    // UI Settings
    FontSizeChanged(String),
    FontSizeIncrement,
    FontSizeDecrement,
    ShowWizardOnStartupToggled(bool),

    // App Settings
    AutoLockTimeoutChanged(String),
    AutoLockTimeoutIncrement,
    AutoLockTimeoutDecrement,
    ClipboardTimeoutChanged(String),
    ClipboardTimeoutIncrement,
    ClipboardTimeoutDecrement,
    EnableBackupToggled(bool),
    BackupCountChanged(String),
    BackupCountIncrement,
    BackupCountDecrement,
    ShowPasswordStrengthToggled(bool),
    MinimizeToTrayToggled(bool),
    StartMinimizedToggled(bool),
    AutoCheckUpdatesToggled(bool),

    // Repository Settings
    DefaultDirectoryChanged(String),
    BrowseDefaultDirectory,
    AutoDetectToggled(bool),

    // Security Settings
    MinPasswordLengthChanged(String),
    MinPasswordLengthIncrement,
    MinPasswordLengthDecrement,
    RequireLowercaseToggled(bool),
    RequireUppercaseToggled(bool),
    RequireNumericToggled(bool),
    RequireSpecialToggled(bool),
    MaxPasswordLengthChanged(String),
    MaxPasswordLengthIncrement,
    MaxPasswordLengthDecrement,
    MinUniqueCharsChanged(String),
    MinUniqueCharsIncrement,
    MinUniqueCharsDecrement,

    // Actions
    Save,
    Reset,
    Cancel,
    DirectorySelected(Option<PathBuf>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Interface,
    Application,
    Repository,
    Security,
}

impl SettingsTab {
    fn all() -> Vec<Self> {
        vec![
            Self::Interface,
            Self::Application,
            Self::Repository,
            Self::Security,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Interface => "Interface",
            Self::Application => "Application",
            Self::Repository => "Repository",
            Self::Security => "Security",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsView {
    // Current tab
    current_tab: SettingsTab,

    // Store original config for comparison
    original_config: FrontendConfig,

    // UI Settings
    font_size: String,
    show_wizard_on_startup: bool,

    // App Settings
    auto_lock_timeout: String,
    clipboard_timeout: String,
    enable_backup: bool,
    backup_count: String,
    show_password_strength: bool,
    minimize_to_tray: bool,
    start_minimized: bool,
    auto_check_updates: bool,

    // Repository Settings
    default_directory: String,
    auto_detect: bool,

    // Security Settings
    min_password_length: String,
    require_lowercase: bool,
    require_uppercase: bool,
    require_numeric: bool,
    require_special: bool,
    max_password_length: String,
    min_unique_chars: String,

    // Validation and state
    validation_errors: Vec<String>,
    validation_warnings: Vec<String>,

    // Store original values for fields not in FrontendConfig
    original_backup_count: String,
    original_min_password_length: String,
    original_require_lowercase: bool,
    original_require_uppercase: bool,
    original_require_numeric: bool,
    original_require_special: bool,
    original_max_password_length: String,
    original_min_unique_chars: String,

    // State tracking
    has_changes: bool,
    is_saving: bool,
}

impl SettingsView {
    pub fn new(config: FrontendConfig) -> Self {
        let mut result = Self {
            current_tab: SettingsTab::Interface,
            original_config: config.clone(),

            // Initialize form fields from config
            font_size: config.ui.font_size.to_string(),
            show_wizard_on_startup: config.ui.show_wizard_on_startup,

            auto_lock_timeout: config.app.auto_lock_timeout.to_string(),
            clipboard_timeout: config.app.clipboard_timeout.to_string(),
            enable_backup: config.app.enable_backup,
            backup_count: "3".to_string(), // Default backup count (not from config yet)
            show_password_strength: config.app.show_password_strength,
            minimize_to_tray: config.app.minimize_to_tray,
            start_minimized: config.app.start_minimized,
            auto_check_updates: config.app.auto_check_updates,

            default_directory: config
                .repository
                .default_directory
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            auto_detect: config.repository.auto_detect,

            // Default security settings
            min_password_length: "12".to_string(),
            require_lowercase: true,
            require_uppercase: true,
            require_numeric: true,
            require_special: true,
            max_password_length: "0".to_string(),
            min_unique_chars: "8".to_string(),

            // Store original values for change detection
            original_backup_count: "3".to_string(),
            original_min_password_length: "12".to_string(),
            original_require_lowercase: true,
            original_require_uppercase: true,
            original_require_numeric: true,
            original_require_special: true,
            original_max_password_length: "0".to_string(),
            original_min_unique_chars: "8".to_string(),

            validation_errors: Vec::new(),
            validation_warnings: Vec::new(),
            has_changes: false,
            is_saving: false,
        };

        // Run initial validation and change detection
        result.validate();
        result.check_for_changes();

        // Log warnings for user awareness
        for warning in &result.validation_warnings {
            info!("Settings warning: {}", warning);
        }

        info!(
            "Settings initialized: has_changes={}, validation_errors={}, validation_warnings={}, font_size='{}'",
            result.has_changes,
            result.validation_errors.len(),
            result.validation_warnings.len(),
            result.font_size
        );

        result
    }

    pub fn update(&mut self, message: SettingsMessage) -> iced::Command<SettingsMessage> {
        match message {
            SettingsMessage::SelectTab(tab) => {
                self.current_tab = tab;
                iced::Command::none()
            }

            // UI Settings
            SettingsMessage::FontSizeChanged(value) => {
                // Ensure font size doesn't go below 8.0
                if let Ok(size) = value.parse::<f32>() {
                    if size >= 8.0 {
                        self.font_size = value;
                    } else {
                        self.font_size = "8.0".to_string();
                    }
                } else {
                    self.font_size = value;
                }
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::FontSizeIncrement => {
                if let Ok(mut size) = self.font_size.parse::<f32>() {
                    size += 1.0;
                    if size <= 24.0 {
                        self.font_size = size.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::FontSizeDecrement => {
                if let Ok(mut size) = self.font_size.parse::<f32>() {
                    if size > 8.0 {
                        size -= 1.0;
                        if size < 8.0 {
                            size = 8.0;
                        }
                        self.font_size = size.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::ShowWizardOnStartupToggled(value) => {
                self.show_wizard_on_startup = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }

            // App Settings
            SettingsMessage::AutoLockTimeoutChanged(value) => {
                self.auto_lock_timeout = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::AutoLockTimeoutIncrement => {
                if let Ok(mut timeout) = self.auto_lock_timeout.parse::<u32>() {
                    timeout += 5;
                    if timeout <= 1440 {
                        self.auto_lock_timeout = timeout.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::AutoLockTimeoutDecrement => {
                if let Ok(mut timeout) = self.auto_lock_timeout.parse::<u32>() {
                    if timeout >= 5 {
                        timeout -= 5;
                        self.auto_lock_timeout = timeout.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::ClipboardTimeoutChanged(value) => {
                self.clipboard_timeout = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::ClipboardTimeoutIncrement => {
                if let Ok(mut timeout) = self.clipboard_timeout.parse::<u32>() {
                    timeout += 5;
                    if timeout <= 300 {
                        self.clipboard_timeout = timeout.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::ClipboardTimeoutDecrement => {
                if let Ok(mut timeout) = self.clipboard_timeout.parse::<u32>() {
                    if timeout >= 5 {
                        timeout -= 5;
                        self.clipboard_timeout = timeout.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::EnableBackupToggled(value) => {
                self.enable_backup = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::BackupCountChanged(value) => {
                self.backup_count = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::BackupCountIncrement => {
                if let Ok(mut count) = self.backup_count.parse::<u32>() {
                    count += 1;
                    if count <= 50 {
                        self.backup_count = count.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::BackupCountDecrement => {
                if let Ok(mut count) = self.backup_count.parse::<u32>() {
                    if count > 1 {
                        count -= 1;
                        self.backup_count = count.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::ShowPasswordStrengthToggled(value) => {
                self.show_password_strength = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::MinimizeToTrayToggled(value) => {
                self.minimize_to_tray = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::StartMinimizedToggled(value) => {
                self.start_minimized = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::AutoCheckUpdatesToggled(value) => {
                self.auto_check_updates = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }

            // Repository Settings
            SettingsMessage::DefaultDirectoryChanged(value) => {
                self.default_directory = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::BrowseDefaultDirectory => {
                // TODO: Open file dialog
                iced::Command::none()
            }
            SettingsMessage::AutoDetectToggled(value) => {
                self.auto_detect = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }

            // Security Settings
            SettingsMessage::MinPasswordLengthChanged(value) => {
                self.min_password_length = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::MinPasswordLengthIncrement => {
                if let Ok(mut len) = self.min_password_length.parse::<usize>() {
                    len += 1;
                    if len <= 256 {
                        self.min_password_length = len.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::MinPasswordLengthDecrement => {
                if let Ok(mut len) = self.min_password_length.parse::<usize>() {
                    if len > 1 {
                        len -= 1;
                        self.min_password_length = len.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::RequireLowercaseToggled(value) => {
                self.require_lowercase = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::RequireUppercaseToggled(value) => {
                self.require_uppercase = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::RequireNumericToggled(value) => {
                self.require_numeric = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::RequireSpecialToggled(value) => {
                self.require_special = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::MaxPasswordLengthChanged(value) => {
                self.max_password_length = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::MaxPasswordLengthIncrement => {
                if let Ok(mut len) = self.max_password_length.parse::<usize>() {
                    len += 1;
                    if len <= 1024 {
                        self.max_password_length = len.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                } else if self.max_password_length == "0" {
                    self.max_password_length = "1".to_string();
                    self.check_for_changes();
                    self.validate();
                }
                iced::Command::none()
            }
            SettingsMessage::MaxPasswordLengthDecrement => {
                if let Ok(mut len) = self.max_password_length.parse::<usize>() {
                    if len > 1 {
                        len -= 1;
                        self.max_password_length = len.to_string();
                    } else if len == 1 {
                        self.max_password_length = "0".to_string(); // 0 = no limit
                    }
                    self.check_for_changes();
                    self.validate();
                }
                iced::Command::none()
            }
            SettingsMessage::MinUniqueCharsChanged(value) => {
                self.min_unique_chars = value;
                self.check_for_changes();
                self.validate();
                iced::Command::none()
            }
            SettingsMessage::MinUniqueCharsIncrement => {
                if let Ok(mut chars) = self.min_unique_chars.parse::<usize>() {
                    chars += 1;
                    if chars <= 64 {
                        self.min_unique_chars = chars.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }
            SettingsMessage::MinUniqueCharsDecrement => {
                if let Ok(mut chars) = self.min_unique_chars.parse::<usize>() {
                    if chars > 0 {
                        chars -= 1;
                        self.min_unique_chars = chars.to_string();
                        self.check_for_changes();
                        self.validate();
                    }
                }
                iced::Command::none()
            }

            SettingsMessage::DirectorySelected(path) => {
                if let Some(path) = path {
                    self.default_directory = path.to_string_lossy().to_string();
                    self.check_for_changes();
                    self.validate();
                }
                iced::Command::none()
            }

            // Actions
            SettingsMessage::Save => {
                if self.validation_errors.is_empty() {
                    self.is_saving = true;
                    // TODO: Implement save functionality
                    info!("Saving settings configuration");
                }
                iced::Command::none()
            }
            SettingsMessage::Reset => {
                self.reset_to_original();
                iced::Command::none()
            }
            SettingsMessage::Cancel => {
                // TODO: Signal to parent to close settings view
                iced::Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let header = container(
            row![
                text("Settings")
                    .size(crate::ui::theme::utils::typography::header_text_size())
                    .style(iced::theme::Text::Color(theme::DARK_TEXT)),
                Space::with_width(Length::Fill),
                if self.has_changes {
                    button("Reset")
                        .on_press(SettingsMessage::Reset)
                        .style(button_styles::secondary())
                        .padding(utils::standard_button_padding())
                } else {
                    button("Reset")
                        .style(button_styles::disabled())
                        .padding(utils::standard_button_padding())
                },
                Space::with_width(Length::Fixed(10.0)),
                button("Cancel")
                    .on_press(SettingsMessage::Cancel)
                    .style(button_styles::secondary())
                    .padding(utils::standard_button_padding()),
                Space::with_width(Length::Fixed(10.0)),
                {
                    let save_enabled = self.has_changes && self.validation_errors.is_empty();
                    info!(
                        "Save button: has_changes={}, errors={}, warnings={}, enabled={}",
                        self.has_changes,
                        self.validation_errors.len(),
                        self.validation_warnings.len(),
                        save_enabled
                    );

                    if save_enabled {
                        button("Save")
                            .on_press(SettingsMessage::Save)
                            .style(button_styles::primary())
                            .padding(utils::standard_button_padding())
                    } else {
                        button("Save")
                            .style(button_styles::disabled())
                            .padding(utils::standard_button_padding())
                    }
                }
            ]
            .align_items(Alignment::Center),
        )
        .padding([20, 20, 20, 20]) // Add top padding for header spacing
        .width(Length::Fill);

        let tabs = self.view_tabs();
        let content = self.view_current_tab();

        let main_content = column![
            header,
            tabs,
            Space::with_height(Length::Fixed(20.0)),
            content,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        container(scrollable(main_content))
            .padding(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_tabs(&self) -> Element<'_, SettingsMessage> {
        let tab_buttons: Vec<Element<'_, SettingsMessage>> = SettingsTab::all()
            .iter()
            .map(|&tab| {
                let style = if tab == self.current_tab {
                    button_styles::primary()
                } else {
                    button_styles::secondary()
                };

                button(
                    text(tab.label()).size(crate::ui::theme::utils::typography::normal_text_size()),
                )
                .on_press(SettingsMessage::SelectTab(tab))
                .style(style)
                .padding(utils::standard_button_padding())
                .into()
            })
            .collect();

        let tabs_row = tab_buttons.into_iter().fold(
            row![].spacing(5).align_items(Alignment::Center),
            |row, button| row.push(button),
        );

        container(tabs_row)
            .padding(utils::main_content_padding())
            .width(Length::Fill)
            .into()
    }

    fn view_current_tab(&self) -> Element<'_, SettingsMessage> {
        let content = match self.current_tab {
            SettingsTab::Interface => self.view_interface_settings(),
            SettingsTab::Application => self.view_application_settings(),
            SettingsTab::Repository => self.view_repository_settings(),
            SettingsTab::Security => self.view_security_settings(),
        };

        container(content)
            .padding(utils::main_content_padding())
            .width(Length::Fill)
            .into()
    }

    fn view_interface_settings(&self) -> Element<'_, SettingsMessage> {
        let interface_settings = column![
            text("Interface Settings")
                .size(crate::ui::theme::utils::typography::large_text_size())
                .style(iced::theme::Text::Color(theme::DARK_TEXT)),
            Space::with_height(Length::Fixed(10.0)),
        ]
        .spacing(10);

        let appearance_settings = column![
            text("Appearance")
                .size(crate::ui::theme::utils::typography::large_text_size())
                .style(iced::theme::Text::Color(theme::DARK_TEXT)),
            Space::with_height(Length::Fixed(10.0)),
            self.create_number_input_row(
                "Font Size:",
                &self.font_size,
                SettingsMessage::FontSizeChanged,
                SettingsMessage::FontSizeIncrement,
                SettingsMessage::FontSizeDecrement,
                "points (8.0 - 24.0)"
            ),
        ]
        .spacing(10);

        let startup_settings = column![
            text("Startup")
                .size(crate::ui::theme::utils::typography::large_text_size())
                .style(iced::theme::Text::Color(theme::DARK_TEXT)),
            Space::with_height(Length::Fixed(10.0)),
            self.create_checkbox_row(
                "Show setup wizard on startup if no repository is configured",
                self.show_wizard_on_startup,
                SettingsMessage::ShowWizardOnStartupToggled
            ),
        ]
        .spacing(10);

        column![
            interface_settings,
            Space::with_height(Length::Fixed(30.0)),
            appearance_settings,
            Space::with_height(Length::Fixed(30.0)),
            startup_settings,
        ]
        .spacing(0)
        .into()
    }

    fn view_application_settings(&self) -> Element<'_, SettingsMessage> {
        let security_settings = column![
            text("Security")
                .size(crate::ui::theme::utils::typography::large_text_size())
                .style(iced::theme::Text::Color(theme::DARK_TEXT)),
            Space::with_height(Length::Fixed(10.0)),
            self.create_number_input_row(
                "Auto-lock timeout (minutes):",
                &self.auto_lock_timeout,
                SettingsMessage::AutoLockTimeoutChanged,
                SettingsMessage::AutoLockTimeoutIncrement,
                SettingsMessage::AutoLockTimeoutDecrement,
                "0 = disabled"
            ),
            self.create_number_input_row(
                "Clipboard timeout (seconds):",
                &self.clipboard_timeout,
                SettingsMessage::ClipboardTimeoutChanged,
                SettingsMessage::ClipboardTimeoutIncrement,
                SettingsMessage::ClipboardTimeoutDecrement,
                ""
            ),
            self.create_checkbox_row(
                "Show password strength indicators",
                self.show_password_strength,
                SettingsMessage::ShowPasswordStrengthToggled
            ),
        ]
        .spacing(10);

        let system_settings = column![
            text("System Integration")
                .size(crate::ui::theme::utils::typography::large_text_size())
                .style(iced::theme::Text::Color(theme::DARK_TEXT)),
            Space::with_height(Length::Fixed(10.0)),
            self.create_checkbox_row(
                "Minimize to system tray on close",
                self.minimize_to_tray,
                SettingsMessage::MinimizeToTrayToggled
            ),
            self.create_checkbox_row(
                "Start minimized",
                self.start_minimized,
                SettingsMessage::StartMinimizedToggled
            ),
            self.create_checkbox_row(
                "Check for updates automatically",
                self.auto_check_updates,
                SettingsMessage::AutoCheckUpdatesToggled
            ),
        ]
        .spacing(10);

        column![
            security_settings,
            Space::with_height(Length::Fixed(30.0)),
            system_settings,
        ]
        .spacing(0)
        .into()
    }

    fn view_repository_settings(&self) -> Element<'_, SettingsMessage> {
        let repository_settings = column![
            text("Repository Management")
                .size(crate::ui::theme::utils::typography::large_text_size())
                .style(iced::theme::Text::Color(theme::DARK_TEXT)),
            Space::with_height(Length::Fixed(10.0)),
            self.create_directory_input_row(
                "Default directory:",
                &self.default_directory,
                SettingsMessage::DefaultDirectoryChanged
            ),
            self.create_checkbox_row(
                "Auto-detect repositories on startup",
                self.auto_detect,
                SettingsMessage::AutoDetectToggled
            ),
        ]
        .spacing(10);

        let backup_settings = column![
            text("Backup & Storage")
                .size(crate::ui::theme::utils::typography::large_text_size())
                .style(iced::theme::Text::Color(theme::DARK_TEXT)),
            Space::with_height(Length::Fixed(10.0)),
            self.create_checkbox_row(
                "Enable automatic backups",
                self.enable_backup,
                SettingsMessage::EnableBackupToggled
            ),
            if self.enable_backup {
                self.create_number_input_row(
                    "Number of backups to keep:",
                    &self.backup_count,
                    SettingsMessage::BackupCountChanged,
                    SettingsMessage::BackupCountIncrement,
                    SettingsMessage::BackupCountDecrement,
                    "backups (1-50)",
                )
            } else {
                Space::with_height(Length::Fixed(0.0)).into()
            },
        ]
        .spacing(10);

        column![
            repository_settings,
            Space::with_height(Length::Fixed(30.0)),
            backup_settings,
        ]
        .spacing(0)
        .into()
    }

    fn view_security_settings(&self) -> Element<'_, SettingsMessage> {
        let password_settings = column![
            text("Master Password Requirements")
                .size(crate::ui::theme::utils::typography::large_text_size())
                .style(iced::theme::Text::Color(theme::DARK_TEXT)),
            Space::with_height(Length::Fixed(10.0)),
            self.create_number_input_row(
                "Minimum length:",
                &self.min_password_length,
                SettingsMessage::MinPasswordLengthChanged,
                SettingsMessage::MinPasswordLengthIncrement,
                SettingsMessage::MinPasswordLengthDecrement,
                "characters"
            ),
            self.create_number_input_row(
                "Maximum length:",
                &self.max_password_length,
                SettingsMessage::MaxPasswordLengthChanged,
                SettingsMessage::MaxPasswordLengthIncrement,
                SettingsMessage::MaxPasswordLengthDecrement,
                "characters (0 = no limit)"
            ),
            self.create_number_input_row(
                "Minimum unique characters:",
                &self.min_unique_chars,
                SettingsMessage::MinUniqueCharsChanged,
                SettingsMessage::MinUniqueCharsIncrement,
                SettingsMessage::MinUniqueCharsDecrement,
                "characters"
            ),
            self.create_checkbox_row(
                "Require lowercase letters",
                self.require_lowercase,
                SettingsMessage::RequireLowercaseToggled
            ),
            self.create_checkbox_row(
                "Require uppercase letters",
                self.require_uppercase,
                SettingsMessage::RequireUppercaseToggled
            ),
            self.create_checkbox_row(
                "Require numeric characters",
                self.require_numeric,
                SettingsMessage::RequireNumericToggled
            ),
            self.create_checkbox_row(
                "Require special characters",
                self.require_special,
                SettingsMessage::RequireSpecialToggled
            ),
        ]
        .spacing(10);

        column![password_settings].spacing(0).into()
    }

    // Helper methods for creating form elements

    fn create_number_input_row<F>(
        &self,
        label: &str,
        value: &str,
        on_change: F,
        on_increment: SettingsMessage,
        on_decrement: SettingsMessage,
        placeholder: &str,
    ) -> Element<'_, SettingsMessage>
    where
        F: Fn(String) -> SettingsMessage + 'static,
    {
        let input_style = if self.is_field_invalid(label) {
            text_input_styles::invalid()
        } else {
            text_input_styles::standard()
        };

        row![
            container(text(label).size(crate::ui::theme::utils::typography::normal_text_size()))
                .width(Length::Fixed(200.0)),
            row![
                button(container(text("-")).center_x().center_y())
                    .on_press(on_decrement)
                    .style(button_styles::secondary())
                    .padding([4, 8])
                    .width(Length::Fixed(30.0))
                    .height(Length::Fixed(32.0)),
                text_input(placeholder, value)
                    .on_input(on_change)
                    .style(input_style)
                    .padding(utils::text_input_padding())
                    .size(crate::ui::theme::utils::typography::text_input_size())
                    .width(Length::Fixed(120.0)),
                button(container(text("+")).center_x().center_y())
                    .on_press(on_increment)
                    .style(button_styles::secondary())
                    .padding([4, 8])
                    .width(Length::Fixed(30.0))
                    .height(Length::Fixed(32.0)),
            ]
            .align_items(Alignment::Center)
            .spacing(2)
        ]
        .align_items(Alignment::Center)
        .spacing(10)
        .into()
    }

    fn create_directory_input_row<F>(
        &self,
        label: &str,
        value: &str,
        on_change: F,
    ) -> Element<'_, SettingsMessage>
    where
        F: Fn(String) -> SettingsMessage + 'static,
    {
        let input_style = if self.is_field_invalid(label) {
            text_input_styles::invalid()
        } else {
            text_input_styles::standard()
        };

        row![
            container(text(label).size(crate::ui::theme::utils::typography::normal_text_size()))
                .width(Length::Fixed(200.0)),
            text_input("Choose directory...", value)
                .on_input(on_change)
                .style(input_style)
                .padding(utils::text_input_padding())
                .size(crate::ui::theme::utils::typography::text_input_size())
                .width(Length::Fixed(300.0)),
            button("Browse")
                .on_press(SettingsMessage::BrowseDefaultDirectory)
                .style(button_styles::secondary())
                .padding(utils::small_button_padding()),
        ]
        .align_items(Alignment::Center)
        .spacing(10)
        .into()
    }

    fn create_checkbox_row<F>(
        &self,
        label: &str,
        value: bool,
        on_toggle: F,
    ) -> Element<'_, SettingsMessage>
    where
        F: Fn(bool) -> SettingsMessage + 'static,
    {
        let cb = checkbox(label, value).on_toggle(on_toggle);
        cb.into()
    }

    // Validation and state management
    fn validate(&mut self) {
        let old_error_count = self.validation_errors.len();
        let old_warning_count = self.validation_warnings.len();
        self.validation_errors.clear();
        self.validation_warnings.clear();

        // Validate font size
        if let Ok(size) = self.font_size.parse::<f32>() {
            if size < 8.0 || size > 24.0 {
                let error = "Font size must be between 8.0 and 24.0 points".to_string();
                info!("Validation error: {} (font_size={})", error, self.font_size);
                self.validation_errors.push(error);
            }
        } else if !self.font_size.is_empty() {
            let error = "Font size must be a valid number".to_string();
            info!(
                "Validation error: {} (font_size='{}')",
                error, self.font_size
            );
            self.validation_errors.push(error);
        }

        // Validate timeouts
        if let Ok(timeout) = self.auto_lock_timeout.parse::<u32>() {
            if timeout > 1440 {
                // More than 24 hours
                let error = "Auto-lock timeout cannot exceed 1440 minutes (24 hours)".to_string();
                info!(
                    "Validation error: {} (auto_lock_timeout={})",
                    error, self.auto_lock_timeout
                );
                self.validation_errors.push(error);
            }
        } else if !self.auto_lock_timeout.is_empty() {
            let error = "Auto-lock timeout must be a valid number".to_string();
            info!(
                "Validation error: {} (auto_lock_timeout='{}')",
                error, self.auto_lock_timeout
            );
            self.validation_errors.push(error);
        }

        if let Ok(timeout) = self.clipboard_timeout.parse::<u32>() {
            if timeout > 300 {
                // More than 5 minutes
                let error = "Clipboard timeout cannot exceed 300 seconds".to_string();
                info!(
                    "Validation error: {} (clipboard_timeout={})",
                    error, self.clipboard_timeout
                );
                self.validation_errors.push(error);
            }
        } else if !self.clipboard_timeout.is_empty() {
            let error = "Clipboard timeout must be a valid number".to_string();
            info!(
                "Validation error: {} (clipboard_timeout='{}')",
                error, self.clipboard_timeout
            );
            self.validation_errors.push(error);
        }

        // Validate backup count
        if let Ok(count) = self.backup_count.parse::<u32>() {
            if count < 1 || count > 50 {
                let error = "Number of backups must be between 1 and 50".to_string();
                info!(
                    "Validation error: {} (backup_count={})",
                    error, self.backup_count
                );
                self.validation_errors.push(error);
            }
        } else if !self.backup_count.is_empty() {
            let error = "Number of backups must be a valid number".to_string();
            info!(
                "Validation error: {} (backup_count='{}')",
                error, self.backup_count
            );
            self.validation_errors.push(error);
        }

        // Validate password requirements
        if let Ok(min_len) = self.min_password_length.parse::<usize>() {
            if min_len < 1 || min_len > 256 {
                let error = "Minimum password length must be between 1 and 256".to_string();
                info!(
                    "Validation error: {} (min_password_length={})",
                    error, self.min_password_length
                );
                self.validation_errors.push(error);
            }
        } else if !self.min_password_length.is_empty() {
            let error = "Minimum password length must be a valid number".to_string();
            info!(
                "Validation error: {} (min_password_length='{}')",
                error, self.min_password_length
            );
            self.validation_errors.push(error);
        }

        if let Ok(max_len) = self.max_password_length.parse::<usize>() {
            if max_len > 0 {
                if let Ok(min_len) = self.min_password_length.parse::<usize>() {
                    if max_len < min_len {
                        let error = "Maximum password length cannot be less than minimum length"
                            .to_string();
                        info!(
                            "Validation error: {} (max_password_length={}, min_password_length={})",
                            error, self.max_password_length, self.min_password_length
                        );
                        self.validation_errors.push(error);
                    }
                }
                if max_len > 1024 {
                    let error = "Maximum password length cannot exceed 1024".to_string();
                    info!(
                        "Validation error: {} (max_password_length={})",
                        error, self.max_password_length
                    );
                    self.validation_errors.push(error);
                }
            }
        } else if !self.max_password_length.is_empty() && self.max_password_length != "0" {
            let error = "Maximum password length must be a valid number".to_string();
            info!(
                "Validation error: {} (max_password_length='{}')",
                error, self.max_password_length
            );
            self.validation_errors.push(error);
        }

        if let Ok(min_unique) = self.min_unique_chars.parse::<usize>() {
            if min_unique > 64 {
                let error = "Minimum unique characters cannot exceed 64".to_string();
                info!(
                    "Validation error: {} (min_unique_chars={})",
                    error, self.min_unique_chars
                );
                self.validation_errors.push(error);
            }
        } else if !self.min_unique_chars.is_empty() {
            let error = "Minimum unique characters must be a valid number".to_string();
            info!(
                "Validation error: {} (min_unique_chars='{}')",
                error, self.min_unique_chars
            );
            self.validation_errors.push(error);
        }

        // Validate directory path (only if not empty and not the default from config)
        // We don't validate directory existence for paths loaded from config to avoid
        // blocking the Save button when directories are moved/deleted external to the app
        if !self.default_directory.is_empty() {
            let path = std::path::Path::new(&self.default_directory);
            // Only validate if the directory field has been manually changed from the original
            let dir_changed = self.default_directory
                != self
                    .original_config
                    .repository
                    .default_directory
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

            if dir_changed {
                if !path.exists() {
                    let error = "Default directory does not exist".to_string();
                    info!(
                        "Validation error: {} (default_directory='{}')",
                        error, self.default_directory
                    );
                    self.validation_errors.push(error);
                } else if !path.is_dir() {
                    let error = "Default directory path is not a directory".to_string();
                    info!(
                        "Validation error: {} (default_directory='{}')",
                        error, self.default_directory
                    );
                    self.validation_errors.push(error);
                }
            } else {
                // For paths from config, just log a warning but don't block saving
                if !path.exists() {
                    let warning = format!(
                        "Default directory does not exist: {}",
                        self.default_directory
                    );
                    info!("Warning: {} (not blocking save)", warning);
                    self.validation_warnings.push(warning);
                }
            }
        }

        info!(
            "Validation: {} -> {} errors, {} -> {} warnings",
            old_error_count,
            self.validation_errors.len(),
            old_warning_count,
            self.validation_warnings.len()
        );
    }

    fn is_field_invalid(&self, field_label: &str) -> bool {
        self.validation_errors
            .iter()
            .any(|error| error.to_lowercase().contains(&field_label.to_lowercase()))
    }

    fn check_for_changes(&mut self) {
        let _old_has_changes = self.has_changes;

        // Direct comparison of individual fields with detailed logging
        let font_size_changed = self.font_size != self.original_config.ui.font_size.to_string();
        let show_wizard_changed =
            self.show_wizard_on_startup != self.original_config.ui.show_wizard_on_startup;

        info!(
            "Font size: '{}' vs '{}' = {}",
            self.font_size, self.original_config.ui.font_size, font_size_changed
        );
        info!(
            "Show wizard: {} vs {} = {}",
            self.show_wizard_on_startup,
            self.original_config.ui.show_wizard_on_startup,
            show_wizard_changed
        );

        let ui_changed = font_size_changed || show_wizard_changed;

        let auto_lock_changed =
            self.auto_lock_timeout != self.original_config.app.auto_lock_timeout.to_string();
        let clipboard_changed =
            self.clipboard_timeout != self.original_config.app.clipboard_timeout.to_string();
        let backup_enabled_changed = self.enable_backup != self.original_config.app.enable_backup;
        let password_strength_changed =
            self.show_password_strength != self.original_config.app.show_password_strength;
        let minimize_tray_changed =
            self.minimize_to_tray != self.original_config.app.minimize_to_tray;
        let start_minimized_changed =
            self.start_minimized != self.original_config.app.start_minimized;
        let auto_updates_changed =
            self.auto_check_updates != self.original_config.app.auto_check_updates;

        info!(
            "Auto lock: '{}' vs '{}' = {}",
            self.auto_lock_timeout, self.original_config.app.auto_lock_timeout, auto_lock_changed
        );
        info!(
            "Clipboard: '{}' vs '{}' = {}",
            self.clipboard_timeout, self.original_config.app.clipboard_timeout, clipboard_changed
        );
        info!(
            "Enable backup: {} vs {} = {}",
            self.enable_backup, self.original_config.app.enable_backup, backup_enabled_changed
        );
        info!(
            "Show password strength: {} vs {} = {}",
            self.show_password_strength,
            self.original_config.app.show_password_strength,
            password_strength_changed
        );
        info!(
            "Minimize to tray: {} vs {} = {}",
            self.minimize_to_tray, self.original_config.app.minimize_to_tray, minimize_tray_changed
        );
        info!(
            "Start minimized: {} vs {} = {}",
            self.start_minimized, self.original_config.app.start_minimized, start_minimized_changed
        );
        info!(
            "Auto check updates: {} vs {} = {}",
            self.auto_check_updates,
            self.original_config.app.auto_check_updates,
            auto_updates_changed
        );

        let app_changed = auto_lock_changed
            || clipboard_changed
            || backup_enabled_changed
            || password_strength_changed
            || minimize_tray_changed
            || start_minimized_changed
            || auto_updates_changed;

        let original_default_dir = self
            .original_config
            .repository
            .default_directory
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let default_dir_changed = self.default_directory != original_default_dir;
        let auto_detect_changed = self.auto_detect != self.original_config.repository.auto_detect;

        info!(
            "Default dir: '{}' vs '{}' = {}",
            self.default_directory, original_default_dir, default_dir_changed
        );
        info!(
            "Auto detect: {} vs {} = {}",
            self.auto_detect, self.original_config.repository.auto_detect, auto_detect_changed
        );

        let repo_changed = default_dir_changed || auto_detect_changed;

        // Check security settings and backup count since they're not in FrontendConfig
        let min_len_changed = self.min_password_length != self.original_min_password_length;
        let lowercase_changed = self.require_lowercase != self.original_require_lowercase;
        let uppercase_changed = self.require_uppercase != self.original_require_uppercase;
        let numeric_changed = self.require_numeric != self.original_require_numeric;
        let special_changed = self.require_special != self.original_require_special;
        let max_len_changed = self.max_password_length != self.original_max_password_length;
        let unique_chars_changed = self.min_unique_chars != self.original_min_unique_chars;

        info!(
            "Min password length: '{}' vs '{}' = {}",
            self.min_password_length, self.original_min_password_length, min_len_changed
        );
        info!(
            "Require lowercase: {} vs {} = {}",
            self.require_lowercase, self.original_require_lowercase, lowercase_changed
        );
        info!(
            "Require uppercase: {} vs {} = {}",
            self.require_uppercase, self.original_require_uppercase, uppercase_changed
        );
        info!(
            "Require numeric: {} vs {} = {}",
            self.require_numeric, self.original_require_numeric, numeric_changed
        );
        info!(
            "Require special: {} vs {} = {}",
            self.require_special, self.original_require_special, special_changed
        );
        info!(
            "Max password length: '{}' vs '{}' = {}",
            self.max_password_length, self.original_max_password_length, max_len_changed
        );
        info!(
            "Min unique chars: '{}' vs '{}' = {}",
            self.min_unique_chars, self.original_min_unique_chars, unique_chars_changed
        );

        let security_changed = min_len_changed
            || lowercase_changed
            || uppercase_changed
            || numeric_changed
            || special_changed
            || max_len_changed
            || unique_chars_changed;

        let backup_changed = self.backup_count != self.original_backup_count;
        info!(
            "Backup count: '{}' vs '{}' = {}",
            self.backup_count, self.original_backup_count, backup_changed
        );

        self.has_changes =
            ui_changed || app_changed || repo_changed || security_changed || backup_changed;

        info!(
            "Change detection: ui={}, app={}, repo={}, security={}, backup={} -> has_changes={}",
            ui_changed,
            app_changed,
            repo_changed,
            security_changed,
            backup_changed,
            self.has_changes
        );
    }

    fn reset_to_original(&mut self) {
        let config = &self.original_config;

        // Reset UI settings
        self.font_size = config.ui.font_size.to_string();
        self.show_wizard_on_startup = config.ui.show_wizard_on_startup;

        // Reset app settings
        self.auto_lock_timeout = config.app.auto_lock_timeout.to_string();
        self.clipboard_timeout = config.app.clipboard_timeout.to_string();
        self.enable_backup = config.app.enable_backup;
        self.backup_count = self.original_backup_count.clone();
        self.show_password_strength = config.app.show_password_strength;
        self.minimize_to_tray = config.app.minimize_to_tray;
        self.start_minimized = config.app.start_minimized;
        self.auto_check_updates = config.app.auto_check_updates;

        // Reset repository settings
        self.default_directory = config
            .repository
            .default_directory
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        self.auto_detect = config.repository.auto_detect;

        // Reset security settings to originals
        self.min_password_length = self.original_min_password_length.clone();
        self.require_lowercase = self.original_require_lowercase;
        self.require_uppercase = self.original_require_uppercase;
        self.require_numeric = self.original_require_numeric;
        self.require_special = self.original_require_special;
        self.max_password_length = self.original_max_password_length.clone();
        self.min_unique_chars = self.original_min_unique_chars.clone();

        self.validation_errors.clear();
        self.has_changes = false;
        self.is_saving = false;
    }

    fn build_current_config(&self) -> FrontendConfig {
        FrontendConfig {
            repository: RepositoryConfig {
                path: self.original_config.repository.path.clone(),
                default_directory: if self.default_directory.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&self.default_directory))
                },
                recent_repositories: self.original_config.repository.recent_repositories.clone(),
                max_recent: self.original_config.repository.max_recent,
                auto_detect: self.auto_detect,
                search_directories: self.original_config.repository.search_directories.clone(),
            },
            ui: UiConfig {
                window_width: self.original_config.ui.window_width,
                window_height: self.original_config.ui.window_height,
                theme: self.original_config.ui.theme.clone(),
                show_wizard_on_startup: self.show_wizard_on_startup,
                font_size: self
                    .font_size
                    .parse()
                    .unwrap_or(self.original_config.ui.font_size),
                language: self.original_config.ui.language.clone(),
            },
            app: AppConfig {
                auto_lock_timeout: self
                    .auto_lock_timeout
                    .parse()
                    .unwrap_or(self.original_config.app.auto_lock_timeout),
                clipboard_timeout: self
                    .clipboard_timeout
                    .parse()
                    .unwrap_or(self.original_config.app.clipboard_timeout),
                enable_backup: self.enable_backup,
                show_passwords_default: self.original_config.app.show_passwords_default,
                show_password_strength: self.show_password_strength,
                minimize_to_tray: self.minimize_to_tray,
                start_minimized: self.start_minimized,
                auto_check_updates: self.auto_check_updates,
            },
            version: self.original_config.version.clone(),
        }
    }

    fn configs_are_equal(&self, config1: &FrontendConfig, config2: &FrontendConfig) -> bool {
        config1.ui.font_size == config2.ui.font_size
            && config1.ui.show_wizard_on_startup == config2.ui.show_wizard_on_startup
            && config1.app.auto_lock_timeout == config2.app.auto_lock_timeout
            && config1.app.clipboard_timeout == config2.app.clipboard_timeout
            && config1.app.enable_backup == config2.app.enable_backup
            && config1.app.show_password_strength == config2.app.show_password_strength
            && config1.app.minimize_to_tray == config2.app.minimize_to_tray
            && config1.app.start_minimized == config2.app.start_minimized
            && config1.app.auto_check_updates == config2.app.auto_check_updates
            && config1.repository.default_directory == config2.repository.default_directory
            && config1.repository.auto_detect == config2.repository.auto_detect
    }

    pub fn get_updated_config(&self) -> FrontendConfig {
        self.build_current_config()
    }

    pub fn has_validation_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    pub fn get_validation_errors(&self) -> &Vec<String> {
        &self.validation_errors
    }

    pub fn get_validation_warnings(&self) -> &Vec<String> {
        &self.validation_warnings
    }

    pub fn get_validation_summary(&self) -> String {
        let error_count = self.validation_errors.len();
        let warning_count = self.validation_warnings.len();

        if error_count == 0 && warning_count == 0 {
            "All settings are valid".to_string()
        } else if error_count > 0 && warning_count > 0 {
            format!(
                "{} error(s) and {} warning(s) found",
                error_count, warning_count
            )
        } else if error_count > 0 {
            format!("{} validation error(s) must be fixed", error_count)
        } else {
            format!("{} validation warning(s)", warning_count)
        }
    }
}
