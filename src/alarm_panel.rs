//! Alarm panel overlay — a floating panel for managing alarms and timers.
//!
//! Displayed when the user selects "Alarms & Timers" from the context menu.
//! Provides:
//! - Quick-timer preset buttons
//! - A form to create timers (duration) or alarms (specific time)
//! - Optional message field for notifications
//! - Active alarm list with edit and delete
//! - Clear done / close buttons

use iced::alignment;
use iced::widget::{button, center, column, container, row, scrollable, text, text_input};
use iced::{Color, Element, Fill, Length, Padding};

use crate::alarm::{Alarm, AlarmForm, AlarmFormMode, AlarmManager};
use crate::Message;

/// Quick-timer preset durations (seconds, label).
const TIMER_PRESETS: &[(u64, &str)] = &[
    (60, "1 min"),
    (300, "5 min"),
    (600, "10 min"),
    (900, "15 min"),
    (1800, "30 min"),
    (3600, "1 hour"),
];

/// Build the alarm panel overlay as an iced Element.
pub fn alarm_panel<'a>(manager: &'a AlarmManager, form: &'a AlarmForm) -> Element<'a, Message> {
    let heading = text("Alarms & Timers")
        .size(15)
        .color(Color::from_rgb(0.9, 0.9, 0.9));

    let separator = separator_widget();

    // -- Quick timer presets --
    let preset_label = text("Quick Timer")
        .size(12)
        .color(Color::from_rgb(0.7, 0.7, 0.7));

    let preset_row_1: Vec<Element<'_, Message>> = TIMER_PRESETS[..3]
        .iter()
        .map(|(secs, label)| {
            button(text(*label).size(11).align_x(alignment::Horizontal::Center))
                .on_press(Message::AddQuickTimer(*secs))
                .padding(Padding::from([3, 6]))
                .style(preset_button_style)
                .into()
        })
        .collect();

    let preset_row_2: Vec<Element<'_, Message>> = TIMER_PRESETS[3..]
        .iter()
        .map(|(secs, label)| {
            button(text(*label).size(11).align_x(alignment::Horizontal::Center))
                .on_press(Message::AddQuickTimer(*secs))
                .padding(Padding::from([3, 6]))
                .style(preset_button_style)
                .into()
        })
        .collect();

    let presets = column![
        preset_label,
        row(preset_row_1).spacing(4),
        row(preset_row_2).spacing(4),
    ]
    .spacing(3);

    // -- Create / edit form --
    let form_section = build_form(form);

    // -- Active alarms list --
    let active: Vec<&Alarm> = manager.all().iter().filter(|a| a.enabled).collect();

    let alarm_list = if active.is_empty() {
        column![text("No active alarms or timers")
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5))]
    } else {
        let items: Vec<Element<'_, Message>> =
            active.iter().map(|alarm| alarm_row(alarm)).collect();
        column(items).spacing(3)
    };

    let list_label = text("Active")
        .size(12)
        .color(Color::from_rgb(0.7, 0.7, 0.7));

    // -- Clear / close buttons --
    let mut bottom_row: Vec<Element<'_, Message>> = Vec::new();

    if manager.all().iter().any(|a| a.fired) {
        bottom_row.push(
            button(
                text("Clear Done")
                    .size(11)
                    .align_x(alignment::Horizontal::Center),
            )
            .on_press(Message::ClearFiredAlarms)
            .padding(Padding::from([3, 8]))
            .style(preset_button_style)
            .into(),
        );
    }

    bottom_row.push(
        button(
            text("Close")
                .size(11)
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(Message::DismissAlarmPanel)
        .padding(Padding::from([3, 8]))
        .style(close_button_style)
        .into(),
    );

    let panel_col = column![
        heading,
        separator,
        presets,
        separator_widget(),
        form_section,
        separator_widget(),
        list_label,
        scrollable(alarm_list).height(Length::Shrink),
        separator_widget(),
        row(bottom_row).spacing(6),
    ]
    .spacing(6)
    .padding(12)
    .width(Length::Fixed(260.0));

    let panel = container(panel_col)
        .style(panel_style)
        .width(Length::Shrink)
        .height(Length::Shrink);

    center(panel).width(Fill).height(Fill).into()
}

// -- Form builder ----------------------------------------------------------

/// Build the create/edit form section.
fn build_form(form: &AlarmForm) -> Element<'_, Message> {
    let is_editing = form.editing.is_some();
    let form_heading = if is_editing {
        "Edit Alarm"
    } else {
        "New Alarm"
    };
    let heading = text(form_heading)
        .size(12)
        .color(Color::from_rgb(0.7, 0.7, 0.7));

    // -- Mode toggle: Timer | Alarm --
    let timer_btn = button(
        text("Timer")
            .size(11)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormSetMode(AlarmFormMode::Timer))
    .padding(Padding::from([2, 8]))
    .style(if form.mode == AlarmFormMode::Timer {
        active_mode_style
    } else {
        preset_button_style
    });

    let alarm_btn = button(
        text("Alarm")
            .size(11)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormSetMode(AlarmFormMode::Alarm))
    .padding(Padding::from([2, 8]))
    .style(if form.mode == AlarmFormMode::Alarm {
        active_mode_style
    } else {
        preset_button_style
    });

    let mode_row = row![timer_btn, alarm_btn].spacing(4);

    // -- Label input --
    let label_input = text_input("Label (e.g. Tea timer)", &form.label)
        .on_input(Message::AlarmFormLabelChanged)
        .size(11)
        .padding(Padding::from([3, 6]))
        .style(form_input_style);

    // -- Message input --
    let message_input = text_input("Notification message (optional)", &form.message)
        .on_input(Message::AlarmFormMessageChanged)
        .size(11)
        .padding(Padding::from([3, 6]))
        .style(form_input_style);

    // -- Mode-specific fields --
    let mode_fields: Element<'_, Message> = match form.mode {
        AlarmFormMode::Timer => {
            let minutes_input = text_input("Minutes", &form.timer_minutes)
                .on_input(Message::AlarmFormMinutesChanged)
                .size(11)
                .padding(Padding::from([3, 6]))
                .style(form_input_style);
            column![minutes_input].spacing(3).into()
        }
        AlarmFormMode::Alarm => {
            let time_input = text_input("Time (HH:MM)", &form.alarm_time)
                .on_input(Message::AlarmFormTimeChanged)
                .size(11)
                .padding(Padding::from([3, 6]))
                .style(form_input_style);
            let date_input = text_input("Date (YYYY-MM-DD, blank=today)", &form.alarm_date)
                .on_input(Message::AlarmFormDateChanged)
                .size(11)
                .padding(Padding::from([3, 6]))
                .style(form_input_style);
            column![time_input, date_input].spacing(3).into()
        }
    };

    // -- Submit / cancel buttons --
    let submit_label = if is_editing { "Save" } else { "Add" };
    let submit_btn = button(
        text(submit_label)
            .size(11)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormSubmit)
    .padding(Padding::from([3, 8]))
    .style(submit_button_style);

    let cancel_label = if is_editing { "Cancel" } else { "Clear" };
    let cancel_btn = button(
        text(cancel_label)
            .size(11)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormCancel)
    .padding(Padding::from([3, 8]))
    .style(close_button_style);

    let action_row: Vec<Element<'_, Message>> = vec![submit_btn.into(), cancel_btn.into()];

    column![
        heading,
        mode_row,
        label_input,
        message_input,
        mode_fields,
        row(action_row).spacing(6),
    ]
    .spacing(4)
    .into()
}

// -- Alarm row -------------------------------------------------------------

/// A single alarm row: label, remaining time, kind badge, edit and delete buttons.
fn alarm_row<'a>(alarm: &'a Alarm) -> Element<'a, Message> {
    let status_colour = if alarm.fired {
        Color::from_rgb(0.4, 0.8, 0.4)
    } else {
        Color::from_rgb(0.9, 0.9, 0.9)
    };

    let label = text(&alarm.label).size(11).color(status_colour);
    let remaining = text(alarm.remaining_display())
        .size(10)
        .color(Color::from_rgb(0.6, 0.8, 1.0));
    let kind = text(alarm.kind_label())
        .size(9)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

    // Show message excerpt if present.
    let mut info_items: Vec<Element<'_, Message>> =
        vec![label.into(), row![remaining, kind].spacing(6).into()];
    if let Some(msg) = &alarm.message {
        if !msg.is_empty() {
            let excerpt = if msg.len() > 30 {
                format!("{}…", &msg[..30])
            } else {
                msg.clone()
            };
            info_items.push(
                text(excerpt)
                    .size(9)
                    .color(Color::from_rgb(0.6, 0.6, 0.5))
                    .into(),
            );
        }
    }

    let edit_btn = button(text("✎").size(10).align_x(alignment::Horizontal::Center))
        .on_press(Message::EditAlarm(alarm.id))
        .padding(Padding::from([1, 4]))
        .style(edit_button_style);

    let delete_btn = button(text("✕").size(10).align_x(alignment::Horizontal::Center))
        .on_press(Message::RemoveAlarm(alarm.id))
        .padding(Padding::from([1, 4]))
        .style(delete_button_style);

    let info_col = column(info_items).spacing(1);
    let btn_col = row![edit_btn, delete_btn].spacing(2);

    container(row![info_col, btn_col].spacing(6))
        .padding(Padding::from([3, 6]))
        .style(alarm_row_style)
        .width(Fill)
        .into()
}

/// Thin horizontal separator.
fn separator_widget<'a>() -> Element<'a, Message> {
    container(text("").size(1))
        .width(Fill)
        .height(1)
        .style(separator_style)
        .into()
}

// -- Styles ----------------------------------------------------------------

fn panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.10, 0.10, 0.14, 0.94,
        ))),
        border: iced::Border {
            color: Color::from_rgba(0.35, 0.4, 0.5, 0.8),
            width: 1.0,
            radius: 10.0.into(),
        },
        text_color: Some(Color::WHITE),
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn alarm_row_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.18, 0.18, 0.22, 0.6,
        ))),
        border: iced::Border {
            radius: 4.0.into(),
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}

fn separator_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.5, 0.5, 0.5, 0.3,
        ))),
        ..container::Style::default()
    }
}

fn preset_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgba(0.25, 0.45, 0.7, 0.7),
        _ => Color::from_rgba(0.2, 0.2, 0.28, 0.6),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::from_rgb(0.85, 0.9, 1.0),
        border: iced::Border {
            radius: 4.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn active_mode_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.25, 0.5, 0.8, 0.8,
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

fn submit_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgba(0.2, 0.6, 0.3, 0.8),
        _ => Color::from_rgba(0.15, 0.45, 0.25, 0.6),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::from_rgb(0.9, 1.0, 0.9),
        border: iced::Border {
            radius: 4.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn close_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgba(0.35, 0.35, 0.4, 0.7),
        _ => Color::from_rgba(0.25, 0.25, 0.3, 0.5),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::from_rgb(0.8, 0.8, 0.8),
        border: iced::Border {
            radius: 4.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn edit_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgba(0.3, 0.45, 0.7, 0.7),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::from_rgb(0.5, 0.6, 0.8),
        border: iced::Border {
            radius: 3.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn delete_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgba(0.7, 0.2, 0.2, 0.7),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::from_rgb(0.7, 0.4, 0.4),
        border: iced::Border {
            radius: 3.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn form_input_style(_theme: &iced::Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(Color::from_rgba(0.15, 0.15, 0.2, 0.8)),
        border: iced::Border {
            color: Color::from_rgba(0.4, 0.4, 0.5, 0.6),
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: Color::from_rgb(0.6, 0.6, 0.6),
        placeholder: Color::from_rgb(0.45, 0.45, 0.5),
        value: Color::from_rgb(0.9, 0.9, 0.9),
        selection: Color::from_rgba(0.3, 0.5, 0.8, 0.5),
    }
}
