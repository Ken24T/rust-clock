//! Right-click context menu for the clock desklet.
//!
//! Renders a semi-transparent panel with menu items for theme selection,
//! size adjustment, display toggles, and quitting.

use iced::alignment;
use iced::widget::{button, center, column, container, row, text};
use iced::{Color, Element, Fill, Length, Padding};

use crate::alarm::AlarmManager;
use crate::config::AppConfig;
use crate::Message;

// -- Theme name list (must match theme.rs presets) -------------------------

const THEMES: &[&str] = &["classic", "dark", "minimal", "transparent"];
const SIZES: &[(u32, &str)] = &[(150, "Small"), (250, "Medium"), (350, "Large")];

// -- Context menu widget ---------------------------------------------------

/// A floating context menu shown on right-click.
pub struct ContextMenu<'a> {
    config: &'a AppConfig,
    alarm_manager: &'a AlarmManager,
}

impl<'a> ContextMenu<'a> {
    pub fn widget(config: &'a AppConfig, alarm_manager: &'a AlarmManager) -> Element<'a, Message> {
        let menu = Self {
            config,
            alarm_manager,
        };
        menu.build()
    }

    fn build(self) -> Element<'a, Message> {
        let heading = text("Rust Clock")
            .size(14)
            .color(Color::from_rgb(0.9, 0.9, 0.9));

        let close_btn = button(text("✕").size(12).align_x(alignment::Horizontal::Center))
            .on_press(Message::DismissMenu)
            .padding(Padding::from([1, 5]))
            .style(menu_button_style);

        let header_row = row![heading, close_btn]
            .spacing(6)
            .align_y(alignment::Vertical::Center);

        let separator = container(text("").size(1))
            .width(Fill)
            .height(1)
            .style(separator_style);

        // -- Theme picker row --
        let theme_label = text("Theme").size(12).color(Color::from_rgb(0.7, 0.7, 0.7));

        let theme_buttons: Vec<Element<'_, Message>> = THEMES
            .iter()
            .map(|name| {
                let is_active = self.config.theme == *name && self.config.theme_config.is_none();
                let label = capitalise(name);
                let btn = button(text(label).size(11).align_x(alignment::Horizontal::Center))
                    .on_press(Message::SetTheme(name.to_string()))
                    .padding(Padding::from([2, 6]));

                if is_active {
                    btn.style(active_button_style).into()
                } else {
                    btn.style(menu_button_style).into()
                }
            })
            .collect();

        let theme_row = column![theme_label, row(theme_buttons).spacing(4)].spacing(2);

        // -- Size picker row --
        let size_label = text("Size").size(12).color(Color::from_rgb(0.7, 0.7, 0.7));

        let size_buttons: Vec<Element<'_, Message>> = SIZES
            .iter()
            .map(|(sz, label)| {
                let is_active = self.config.size == *sz;
                let btn = button(text(*label).size(11).align_x(alignment::Horizontal::Center))
                    .on_press(Message::SetSize(*sz))
                    .padding(Padding::from([2, 6]));

                if is_active {
                    btn.style(active_button_style).into()
                } else {
                    btn.style(menu_button_style).into()
                }
            })
            .collect();

        let size_row = column![size_label, row(size_buttons).spacing(4)].spacing(2);

        // -- Toggle items --
        let date_toggle = menu_toggle("Show Date", self.config.show_date, Message::ToggleDate);
        let smooth_toggle = menu_toggle(
            "Smooth Seconds",
            self.config.smooth_seconds,
            Message::ToggleSmoothSeconds,
        );
        let seconds_toggle = menu_toggle(
            "Show Seconds",
            self.config.show_seconds,
            Message::ToggleSeconds,
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
        .style(menu_button_style);

        let quit_btn = button(text("Quit").size(12).align_x(alignment::Horizontal::Center))
            .on_press(Message::Quit)
            .padding(Padding::from([3, 8]))
            .width(Fill)
            .style(quit_button_style);

        let close_menu_btn = button(
            text("Close")
                .size(12)
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(Message::DismissMenu)
        .padding(Padding::from([3, 8]))
        .width(Fill)
        .style(menu_button_style);

        let menu_col = column![
            header_row,
            separator,
            theme_row,
            size_row,
            date_toggle,
            smooth_toggle,
            seconds_toggle,
            separator_widget(),
            alarm_btn,
            separator_widget(),
            close_menu_btn,
            quit_btn,
        ]
        .spacing(6)
        .padding(10)
        .width(Length::Shrink);

        let panel = container(menu_col)
            .style(menu_panel_style)
            .width(Length::Shrink)
            .height(Length::Shrink);

        // Centre the menu over the clock face.
        center(panel).width(Fill).height(Fill).into()
    }
}

// -- Helper widgets --------------------------------------------------------

/// A toggle menu item: label with a check mark indicator.
fn menu_toggle(label: &str, enabled: bool, message: Message) -> Element<'_, Message> {
    let indicator = if enabled { "✓" } else { "  " };
    let display = format!("{indicator}  {label}");
    button(text(display).size(12).align_x(alignment::Horizontal::Left))
        .on_press(message)
        .padding(Padding::from([3, 8]))
        .width(Fill)
        .style(menu_button_style)
        .into()
}

/// A thin horizontal separator line.
fn separator_widget<'a>() -> Element<'a, Message> {
    container(text("").size(1))
        .width(Fill)
        .height(1)
        .style(separator_style)
        .into()
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

fn menu_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.12, 0.12, 0.15, 0.92,
        ))),
        border: iced::Border {
            color: Color::from_rgba(0.4, 0.4, 0.45, 0.8),
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: Some(Color::WHITE),
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn separator_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.5, 0.5, 0.5, 0.4,
        ))),
        ..container::Style::default()
    }
}

fn menu_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgba(0.3, 0.3, 0.35, 0.6),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::from_rgb(0.9, 0.9, 0.9),
        border: iced::Border {
            radius: 4.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn active_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.25, 0.5, 0.8, 0.7,
        ))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 4.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn quit_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgba(0.8, 0.2, 0.2, 0.7),
        _ => Color::from_rgba(0.5, 0.15, 0.15, 0.5),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::from_rgb(1.0, 0.9, 0.9),
        border: iced::Border {
            radius: 4.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}
