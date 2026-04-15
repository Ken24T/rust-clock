//! Right-click context menu for the clock desklet.
//!
//! Renders a semi-transparent panel with menu items for theme selection,
//! size adjustment, display toggles, and quitting.

use iced::alignment;
use iced::widget::{button, center, column, container, row, text};
use iced::{Element, Fill, Length, Padding};

use crate::alarm::AlarmManager;
use crate::config::{AppConfig, ClockSizePreset, OPACITY_STEP_PERCENT, SIZE_ADJUST_STEP_PERCENT};
use crate::theme::WindowChrome;
use crate::Message;

// -- Theme name list (must match theme.rs presets) -------------------------

const THEMES: &[&str] = &["classic", "dark", "minimal", "transparent"];
const SIZES: &[(ClockSizePreset, &str)] = &[
    (ClockSizePreset::Small, "Small"),
    (ClockSizePreset::Medium, "Medium"),
    (ClockSizePreset::Large, "Large"),
];

// -- Context menu widget ---------------------------------------------------

/// A floating context menu shown on right-click.
pub struct ContextMenu<'a> {
    config: &'a AppConfig,
    alarm_manager: &'a AlarmManager,
}

impl<'a> ContextMenu<'a> {
    pub fn widget(
        config: &'a AppConfig,
        alarm_manager: &'a AlarmManager,
        chrome: WindowChrome,
    ) -> Element<'a, Message> {
        let menu = Self {
            config,
            alarm_manager,
        };
        menu.build(chrome)
    }

    fn build(self, chrome: WindowChrome) -> Element<'a, Message> {
        let heading = text("Rust Clock").size(14).color(chrome.text);
        let version = container(
            text(format!("v{}", env!("CARGO_PKG_VERSION")))
                .size(10)
                .color(chrome.muted_text),
        )
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right);

        let header_row = row![heading, version]
            .spacing(6)
            .align_y(alignment::Vertical::Center);

        let separator = container(text("").size(1))
            .width(Fill)
            .height(1)
            .style(move |theme| separator_style(theme, chrome));

        // -- Theme picker row --
        let theme_label = text("Theme").size(12).color(chrome.muted_text);

        let theme_buttons: Vec<Element<'_, Message>> = THEMES
            .iter()
            .map(|name| {
                let is_active = self.config.theme == *name && self.config.theme_config.is_none();
                let label = capitalise(name);
                let btn = button(text(label).size(11).align_x(alignment::Horizontal::Center))
                    .on_press(Message::SetTheme(name.to_string()))
                    .padding(Padding::from([2, 6]));

                if is_active {
                    btn.style(move |theme, status| active_button_style(theme, status, chrome))
                        .into()
                } else {
                    btn.style(move |theme, status| menu_button_style(theme, status, chrome))
                        .into()
                }
            })
            .collect();

        let theme_row = column![theme_label, row(theme_buttons).spacing(4)].spacing(2);

        // -- Opacity row --
        let opacity_label = text("Opacity").size(12).color(chrome.muted_text);
        let opacity_controls = row![
            step_button(
                "-",
                self.config
                    .can_adjust_opacity_percent(-OPACITY_STEP_PERCENT)
                    .then_some(Message::AdjustOpacity(-OPACITY_STEP_PERCENT),),
                chrome,
            ),
            text(format!("{}%", self.config.opacity_percent))
                .size(11)
                .color(chrome.text),
            step_button(
                "+",
                self.config
                    .can_adjust_opacity_percent(OPACITY_STEP_PERCENT)
                    .then_some(Message::AdjustOpacity(OPACITY_STEP_PERCENT)),
                chrome,
            ),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center);
        let opacity_row = column![opacity_label, opacity_controls].spacing(2);

        // -- Size picker row --
        let size_label = text("Size").size(12).color(chrome.muted_text);

        let size_buttons: Vec<Element<'_, Message>> = SIZES
            .iter()
            .map(|(preset, label)| {
                let is_active = self.config.active_size_preset() == *preset;
                let btn = button(text(*label).size(11).align_x(alignment::Horizontal::Center))
                    .on_press(Message::SetSizePreset(*preset))
                    .padding(Padding::from([2, 6]));

                if is_active {
                    btn.style(move |theme, status| active_button_style(theme, status, chrome))
                        .into()
                } else {
                    btn.style(move |theme, status| menu_button_style(theme, status, chrome))
                        .into()
                }
            })
            .collect();

        let size_row = column![size_label, row(size_buttons).spacing(4)].spacing(2);

        let size_adjust_label = text(format!(
            "Fine Tune {} • {} px",
            self.config.size_adjustment_label(),
            self.config.size
        ))
        .size(12)
        .color(chrome.muted_text);

        let size_adjust_controls = row![
            step_button(
                "-10%",
                self.config
                    .can_adjust_size_adjust_percent(-SIZE_ADJUST_STEP_PERCENT)
                    .then_some(Message::AdjustSize(-SIZE_ADJUST_STEP_PERCENT)),
                chrome,
            ),
            text(self.config.active_size_preset().label())
                .size(11)
                .color(chrome.text),
            step_button(
                "+10%",
                self.config
                    .can_adjust_size_adjust_percent(SIZE_ADJUST_STEP_PERCENT)
                    .then_some(Message::AdjustSize(SIZE_ADJUST_STEP_PERCENT)),
                chrome,
            ),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center);

        let size_adjust_row = column![size_adjust_label, size_adjust_controls].spacing(2);

        // -- Toggle items --
        let date_toggle = menu_toggle(
            "Show Date",
            self.config.show_date,
            Message::ToggleDate,
            chrome,
        );
        let smooth_toggle = menu_toggle(
            "Smooth Seconds",
            self.config.smooth_seconds,
            Message::ToggleSmoothSeconds,
            chrome,
        );
        let seconds_toggle = menu_toggle(
            "Show Seconds",
            self.config.show_seconds,
            Message::ToggleSeconds,
            chrome,
        );

        // -- Quit --
        let alarm_count = self.alarm_manager.active_count();
        let alarm_label = if alarm_count > 0 {
            format!("Alarms & Timers ({alarm_count})")
        } else {
            "Alarms & Timers".to_string()
        };
        let alarm_btn = button(
            text(alarm_label)
                .size(12)
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(Message::ShowAlarmPanel)
        .padding(Padding::from([3, 8]))
        .width(Fill)
        .style(move |theme, status| menu_button_style(theme, status, chrome));

        let quit_btn = button(text("Quit").size(12).align_x(alignment::Horizontal::Center))
            .on_press(Message::Quit)
            .padding(Padding::from([3, 8]))
            .width(Fill)
            .style(move |theme, status| quit_button_style(theme, status, chrome));

        let close_menu_btn = button(
            text("Close")
                .size(12)
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(Message::DismissMenu)
        .padding(Padding::from([3, 8]))
        .width(Fill)
        .style(move |theme, status| menu_button_style(theme, status, chrome));

        let menu_col = column![
            header_row,
            separator,
            theme_row,
            opacity_row,
            size_row,
            size_adjust_row,
            date_toggle,
            smooth_toggle,
            seconds_toggle,
            separator_widget(chrome),
            alarm_btn,
            separator_widget(chrome),
            close_menu_btn,
            quit_btn,
        ]
        .spacing(6)
        .padding(10)
        .width(Length::Shrink);

        let panel = container(menu_col)
            .style(move |theme| menu_panel_style(theme, chrome))
            .width(Length::Shrink)
            .height(Length::Shrink);

        // Centre the menu over the clock face.
        center(panel).width(Fill).height(Fill).into()
    }
}

// -- Helper widgets --------------------------------------------------------

/// A toggle menu item: label with a check mark indicator.
fn menu_toggle(
    label: &str,
    enabled: bool,
    message: Message,
    chrome: WindowChrome,
) -> Element<'_, Message> {
    let indicator = if enabled { "✓" } else { "  " };
    let display = format!("{indicator}  {label}");
    button(text(display).size(12).align_x(alignment::Horizontal::Left))
        .on_press(message)
        .padding(Padding::from([3, 8]))
        .width(Fill)
        .style(move |theme, status| menu_button_style(theme, status, chrome))
        .into()
}

/// A thin horizontal separator line.
fn separator_widget<'a>(chrome: WindowChrome) -> Element<'a, Message> {
    container(text("").size(1))
        .width(Fill)
        .height(1)
        .style(move |theme| separator_style(theme, chrome))
        .into()
}

fn step_button<'a>(
    label: &'a str,
    message: Option<Message>,
    chrome: WindowChrome,
) -> Element<'a, Message> {
    let button = button(text(label).size(11).align_x(alignment::Horizontal::Center))
        .padding(Padding::from([2, 8]))
        .width(Length::Shrink)
        .style(move |theme, status| menu_button_style(theme, status, chrome));

    match message {
        Some(message) => button.on_press(message).into(),
        None => button.into(),
    }
}

/// Capitalise the first letter of a string.
fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

// -- Container / button styles (using iced 0.14 style closures) ------------

fn menu_panel_style(_theme: &iced::Theme, chrome: WindowChrome) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(chrome.panel_background)),
        border: iced::Border {
            color: chrome.panel_border,
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: Some(chrome.text),
        shadow: iced::Shadow {
            color: chrome.panel_shadow,
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        snap: false,
    }
}

fn separator_style(_theme: &iced::Theme, chrome: WindowChrome) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(chrome.separator)),
        ..container::Style::default()
    }
}

fn menu_button_style(
    _theme: &iced::Theme,
    status: button::Status,
    chrome: WindowChrome,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => chrome.surface_hover,
        button::Status::Disabled => chrome.separator,
        _ => chrome.surface,
    };
    let text_color = match status {
        button::Status::Disabled => chrome.muted_text,
        _ => chrome.text,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color,
        border: iced::Border {
            radius: 4.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn active_button_style(
    _theme: &iced::Theme,
    _status: button::Status,
    chrome: WindowChrome,
) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(chrome.accent_soft)),
        text_color: chrome.accent_soft_text,
        border: iced::Border {
            color: chrome.accent,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn quit_button_style(
    _theme: &iced::Theme,
    status: button::Status,
    chrome: WindowChrome,
) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (chrome.danger, chrome.danger_text),
        _ => (chrome.danger_soft, chrome.danger_soft_text),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color,
        border: iced::Border {
            color: chrome.panel_border,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}
