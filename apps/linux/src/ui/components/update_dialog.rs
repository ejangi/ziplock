//! Update Dialog Component
//!
//! This component displays information about available updates and provides
//! appropriate installation instructions based on the detected installation method.

use crate::services::{InstallationMethod, UpdateCheckResult};
use iced::{
    alignment::Horizontal,
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Element, Length,
};

use crate::ui::theme::{button_styles, container_styles, utils};

/// Messages for the update dialog
#[derive(Debug, Clone)]
pub enum UpdateDialogMessage {
    /// Close the dialog
    Close,
    /// Open the release page in browser
    OpenReleasePage,
    /// Copy download command to clipboard
    CopyCommand,
}

/// Update dialog component
#[derive(Debug)]
pub struct UpdateDialog {
    update_result: UpdateCheckResult,
}

impl UpdateDialog {
    /// Create a new update dialog
    pub fn new(update_result: UpdateCheckResult) -> Self {
        Self { update_result }
    }

    /// Create the update dialog view
    pub fn view(&self) -> Element<'_, UpdateDialogMessage> {
        let title = text("Update Available")
            .size(utils::typography::header_text_size())
            .horizontal_alignment(Horizontal::Center);

        let version_info = self.create_version_info();
        let release_notes = self.create_release_notes();
        let installation_instructions = self.create_installation_instructions();
        let action_buttons = self.create_action_buttons();

        let content = column![
            title,
            Space::with_height(Length::Fixed(20.0)),
            version_info,
            Space::with_height(Length::Fixed(15.0)),
            release_notes,
            Space::with_height(Length::Fixed(15.0)),
            installation_instructions,
            Space::with_height(Length::Fixed(20.0)),
            action_buttons,
        ]
        .spacing(0)
        .padding(30)
        .max_width(600)
        .width(Length::Fill);

        container(content)
            .style(container_styles::modal())
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    /// Create version information display
    fn create_version_info(&self) -> Element<'_, UpdateDialogMessage> {
        let current_version = format!("Current: {}", self.update_result.current_version);
        let latest_version = format!(
            "Latest: {}",
            self.update_result
                .latest_version
                .as_ref()
                .unwrap_or(&"Unknown".to_string())
        );

        row![
            container(
                text(&current_version)
                    .size(utils::typography::normal_text_size())
                    .horizontal_alignment(Horizontal::Center)
            )
            .width(Length::FillPortion(1)),
            container(
                text("→")
                    .size(utils::typography::large_text_size())
                    .horizontal_alignment(Horizontal::Center)
            )
            .width(Length::Fixed(40.0)),
            container(
                text(&latest_version)
                    .size(utils::typography::normal_text_size())
                    .horizontal_alignment(Horizontal::Center)
            )
            .width(Length::FillPortion(1)),
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
    }

    /// Create release notes section
    fn create_release_notes(&self) -> Element<'_, UpdateDialogMessage> {
        let notes_title = text("What's New")
            .size(utils::typography::large_text_size())
            .horizontal_alignment(Horizontal::Left);

        let notes_content = if let Some(release) = &self.update_result.latest_release {
            let body = if let Some(body_text) = &release.body {
                if body_text.is_empty() {
                    "No release notes available.".to_string()
                } else {
                    // Simple markdown-to-text conversion for display
                    body_text
                        .lines()
                        .filter(|line| !line.trim().starts_with('#'))
                        .map(|line| line.trim_start_matches("- ").trim())
                        .collect::<Vec<&str>>()
                        .join("\n")
                }
            } else {
                "No release notes available.".to_string()
            };

            text(body).size(utils::typography::normal_text_size())
        } else {
            text("Release information not available.").size(utils::typography::normal_text_size())
        };

        let scrollable_notes = scrollable(
            container(notes_content)
                .padding(15)
                .style(container_styles::card())
                .width(Length::Fill),
        )
        .height(Length::Fixed(150.0));

        column![
            notes_title,
            Space::with_height(Length::Fixed(10.0)),
            scrollable_notes
        ]
        .spacing(0)
        .into()
    }

    /// Create installation instructions
    fn create_installation_instructions(&self) -> Element<'_, UpdateDialogMessage> {
        let instructions_title = text("Installation Instructions")
            .size(utils::typography::large_text_size())
            .horizontal_alignment(Horizontal::Left);

        let installation_method = &self.update_result.installation_method;
        let default_version = "latest".to_string();
        let _latest_version = self
            .update_result
            .latest_version
            .as_ref()
            .unwrap_or(&default_version);

        let instructions_text = installation_method.update_instructions();

        let method_label = match installation_method {
            InstallationMethod::DebianPackage => "Detected: Debian/Ubuntu Package",
            InstallationMethod::ArchAUR => "Detected: Arch Linux (AUR)",
            InstallationMethod::Manual => "Detected: Manual Installation",
            InstallationMethod::Unknown => "Installation Method: Unknown",
        };

        let method_info = text(method_label)
            .size(utils::typography::small_text_size())
            .horizontal_alignment(Horizontal::Left);

        let instructions_content =
            text(instructions_text).size(utils::typography::small_text_size());

        let scrollable_instructions = scrollable(
            container(
                column![
                    method_info,
                    Space::with_height(Length::Fixed(10.0)),
                    instructions_content
                ]
                .spacing(0),
            )
            .padding(15)
            .style(container_styles::card())
            .width(Length::Fill),
        )
        .height(Length::Fixed(120.0));

        column![
            instructions_title,
            Space::with_height(Length::Fixed(10.0)),
            scrollable_instructions
        ]
        .spacing(0)
        .into()
    }

    /// Create action buttons
    fn create_action_buttons(&self) -> Element<'_, UpdateDialogMessage> {
        let close_button = button(text("Close").horizontal_alignment(Horizontal::Center))
            .on_press(UpdateDialogMessage::Close)
            .padding(12)
            .style(button_styles::secondary())
            .width(Length::Fixed(100.0));

        let mut buttons = vec![close_button];

        // Add "View Release" button if we have a release URL
        if let Some(release) = &self.update_result.latest_release {
            if let Some(url) = &release.html_url {
                if !url.is_empty() {
                    let release_button =
                        button(text("View Release").horizontal_alignment(Horizontal::Center))
                            .on_press(UpdateDialogMessage::OpenReleasePage)
                            .padding(12)
                            .width(Length::Fixed(120.0));

                    buttons.insert(0, release_button);
                }
            }
        }

        // Add copy command button for certain installation methods
        if matches!(
            self.update_result.installation_method,
            InstallationMethod::DebianPackage | InstallationMethod::ArchAUR
        ) {
            let copy_button = button(text("Copy Command").horizontal_alignment(Horizontal::Center))
                .on_press(UpdateDialogMessage::CopyCommand)
                .padding(12)
                .style(button_styles::secondary())
                .width(Length::Fixed(130.0));

            buttons.insert(1, copy_button);
        }

        let button_elements: Vec<Element<'_, UpdateDialogMessage>> =
            buttons.into_iter().map(|b| b.into()).collect();
        let button_row = row(button_elements)
            .spacing(15)
            .align_items(Alignment::Center);

        container(button_row).width(Length::Fill).center_x().into()
    }

    /// Get the command to copy to clipboard based on installation method
    pub fn get_copy_command(update_result: &UpdateCheckResult) -> Option<String> {
        let default_version = "latest".to_string();
        let latest_version = update_result
            .latest_version
            .as_ref()
            .unwrap_or(&default_version);

        match update_result.installation_method {
            InstallationMethod::DebianPackage => Some(format!(
                "wget https://github.com/ejangi/ziplock/releases/download/v{}/ziplock_{}_amd64.deb && sudo dpkg -i ziplock_{}_amd64.deb",
                latest_version, latest_version, latest_version
            )),
            InstallationMethod::ArchAUR => Some("yay -Syu ziplock".to_string()),
            _ => None,
        }
    }

    /// Get the release page URL
    pub fn get_release_url(update_result: &UpdateCheckResult) -> Option<String> {
        update_result
            .latest_release
            .as_ref()
            .and_then(|release| release.html_url.clone())
            .filter(|url| !url.is_empty())
    }

    /// Get the update result
    pub fn update_result(&self) -> &UpdateCheckResult {
        &self.update_result
    }
}
