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
use crate::theme::WindowChrome;
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
pub fn alarm_panel<'a>(
    manager: &'a AlarmManager,
    form: &'a AlarmForm,
    chrome: WindowChrome,
) -> Element<'a, Message> {
    let heading = text("Alarms & Timers").size(15).color(chrome.text);

    let close_btn = button(text("✕").size(12).align_x(alignment::Horizontal::Center))
        .on_press(Message::DismissAlarmPanel)
        .padding(Padding::from([1, 5]))
        .style(move |theme, status| delete_button_style(theme, status, chrome));

    let header_row = row![heading, close_btn]
        .spacing(6)
        .align_y(alignment::Vertical::Center);

    let separator = separator_widget(chrome);

    // -- Quick timer presets --
    let preset_label = text("Quick Timer").size(12).color(chrome.muted_text);

    let preset_row_1: Vec<Element<'_, Message>> = TIMER_PRESETS[..3]
        .iter()
        .map(|(secs, label)| {
            button(text(*label).size(11).align_x(alignment::Horizontal::Center))
                .on_press(Message::AddQuickTimer(*secs))
                .padding(Padding::from([3, 6]))
                .style(move |theme, status| preset_button_style(theme, status, chrome))
                .into()
        })
        .collect();

    let preset_row_2: Vec<Element<'_, Message>> = TIMER_PRESETS[3..]
        .iter()
        .map(|(secs, label)| {
            button(text(*label).size(11).align_x(alignment::Horizontal::Center))
                .on_press(Message::AddQuickTimer(*secs))
                .padding(Padding::from([3, 6]))
                .style(move |theme, status| preset_button_style(theme, status, chrome))
                .into()
        })
        .collect();

    let presets = column![
        preset_label,
        row(preset_row_1).spacing(4),
        row(preset_row_2).spacing(4),
    ]
    .spacing(3);

    // -- Active alarms list --
    let active: Vec<&Alarm> = manager.all().iter().filter(|a| a.enabled).collect();

    let alarm_list = if active.is_empty() {
        column![text("No active alarms or timers")
            .size(11)
            .color(chrome.muted_text)]
    } else {
        let items: Vec<Element<'_, Message>> = active
            .iter()
            .map(|alarm| alarm_row(alarm, chrome))
            .collect();
        column(items).spacing(3)
    };

    let list_label = text("Active").size(12).color(chrome.muted_text);

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
            .style(move |theme, status| preset_button_style(theme, status, chrome))
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
        .style(move |theme, status| close_button_style(theme, status, chrome))
        .into(),
    );

    let mut panel_items: Vec<Element<'_, Message>> = vec![
        header_row.into(),
        separator,
        presets.into(),
        separator_widget(chrome),
    ];
    panel_items.extend(build_form_elements(form, chrome));
    panel_items.push(separator_widget(chrome));
    panel_items.push(list_label.into());
    panel_items.push(scrollable(alarm_list).height(Length::Shrink).into());
    panel_items.push(separator_widget(chrome));
    panel_items.push(row(bottom_row).spacing(6).into());

    let panel_col = column(panel_items)
        .spacing(6)
        .padding(12)
        .width(Length::Fixed(260.0));

    let panel = container(panel_col)
        .style(move |theme| panel_style(theme, chrome))
        .width(Length::Shrink)
        .height(Length::Shrink);

    center(panel).width(Fill).height(Fill).into()
}

// -- Form builder ----------------------------------------------------------

/// Build the create/edit form elements as a flat list (avoids nested columns
/// which can prevent button click propagation in iced).
fn build_form_elements(form: &AlarmForm, chrome: WindowChrome) -> Vec<Element<'_, Message>> {
    let is_editing = form.editing.is_some();
    let form_heading = if is_editing {
        "Edit Alarm"
    } else {
        "New Alarm"
    };
    let heading = text(form_heading).size(12).color(chrome.muted_text);

    // -- Mode toggle: Timer | Alarm --
    let timer_btn = button(
        text("Timer")
            .size(11)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormSetMode(AlarmFormMode::Timer))
    .padding(Padding::from([2, 8]))
    .style(move |theme, status| {
        if form.mode == AlarmFormMode::Timer {
            active_mode_style(theme, status, chrome)
        } else {
            preset_button_style(theme, status, chrome)
        }
    });

    let alarm_btn = button(
        text("Alarm")
            .size(11)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormSetMode(AlarmFormMode::Alarm))
    .padding(Padding::from([2, 8]))
    .style(move |theme, status| {
        if form.mode == AlarmFormMode::Alarm {
            active_mode_style(theme, status, chrome)
        } else {
            preset_button_style(theme, status, chrome)
        }
    });

    let mode_row = row![timer_btn, alarm_btn].spacing(4);

    // -- Label input --
    let label_input = text_input("Label (e.g. Tea timer)", &form.label)
        .on_input(Message::AlarmFormLabelChanged)
        .size(11)
        .padding(Padding::from([3, 6]))
        .style(move |theme, status| form_input_style(theme, status, chrome));

    // -- Message input --
    let message_input = text_input("Notification message (optional)", &form.message)
        .on_input(Message::AlarmFormMessageChanged)
        .size(11)
        .padding(Padding::from([3, 6]))
        .style(move |theme, status| form_input_style(theme, status, chrome));

    // -- Submit / cancel buttons --
    let submit_label = if is_editing { "Save" } else { "Add" };
    let submit_btn = button(
        text(submit_label)
            .size(11)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormSubmit)
    .padding(Padding::from([3, 8]))
    .style(move |theme, status| submit_button_style(theme, status, chrome));

    let cancel_label = if is_editing { "Cancel" } else { "Clear" };
    let cancel_btn = button(
        text(cancel_label)
            .size(11)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormCancel)
    .padding(Padding::from([3, 8]))
    .style(move |theme, status| close_button_style(theme, status, chrome));

    let action_row: Vec<Element<'_, Message>> = vec![submit_btn.into(), cancel_btn.into()];

    let mut elements: Vec<Element<'_, Message>> = vec![
        heading.into(),
        mode_row.into(),
        label_input.into(),
        message_input.into(),
    ];

    // Add mode-specific fields directly (flattened).
    match form.mode {
        AlarmFormMode::Timer => {
            elements.push(
                text_input("Minutes", &form.timer_minutes)
                    .on_input(Message::AlarmFormMinutesChanged)
                    .size(11)
                    .padding(Padding::from([3, 6]))
                    .style(move |theme, status| form_input_style(theme, status, chrome))
                    .into(),
            );
        }
        AlarmFormMode::Alarm => {
            elements.push(
                text_input("Time (HH:MM)", &form.alarm_time)
                    .on_input(Message::AlarmFormTimeChanged)
                    .size(11)
                    .padding(Padding::from([3, 6]))
                    .style(move |theme, status| form_input_style(theme, status, chrome))
                    .into(),
            );
            elements.push(
                text_input("Date (YYYY-MM-DD, blank=today)", &form.alarm_date)
                    .on_input(Message::AlarmFormDateChanged)
                    .size(11)
                    .padding(Padding::from([3, 6]))
                    .style(move |theme, status| form_input_style(theme, status, chrome))
                    .into(),
            );
        }
    }

    elements.push(row(action_row).spacing(6).into());
    elements
}

// -- Alarm row -------------------------------------------------------------

/// A single alarm row: label, remaining time, kind badge, edit and delete buttons.
fn alarm_row<'a>(alarm: &'a Alarm, chrome: WindowChrome) -> Element<'a, Message> {
    let status_colour = if alarm.fired {
        chrome.success
    } else {
        chrome.text
    };

    let label = text(&alarm.label).size(11).color(status_colour);
    let remaining = text(alarm.remaining_display())
        .size(10)
        .color(chrome.accent);
    let kind = text(alarm.kind_label()).size(9).color(chrome.muted_text);

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
            info_items.push(text(excerpt).size(9).color(chrome.muted_text).into());
        }
    }

    let edit_btn = button(text("✎").size(10).align_x(alignment::Horizontal::Center))
        .on_press(Message::EditAlarm(alarm.id))
        .padding(Padding::from([1, 4]))
        .style(move |theme, status| edit_button_style(theme, status, chrome));

    let delete_btn = button(text("✕").size(10).align_x(alignment::Horizontal::Center))
        .on_press(Message::RemoveAlarm(alarm.id))
        .padding(Padding::from([1, 4]))
        .style(move |theme, status| delete_button_style(theme, status, chrome));

    let info_col = column(info_items).spacing(1);
    let btn_col = row![edit_btn, delete_btn].spacing(2);

    container(row![info_col, btn_col].spacing(6))
        .padding(Padding::from([3, 6]))
        .style(move |theme| alarm_row_style(theme, chrome))
        .width(Fill)
        .into()
}

/// Thin horizontal separator.
fn separator_widget<'a>(chrome: WindowChrome) -> Element<'a, Message> {
    container(text("").size(1))
        .width(Fill)
        .height(1)
        .style(move |theme| separator_style(theme, chrome))
        .into()
}

// -- Styles ----------------------------------------------------------------

fn panel_style(_theme: &iced::Theme, chrome: WindowChrome) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(chrome.panel_background)),
        border: iced::Border {
            color: chrome.panel_border,
            width: 1.0,
            radius: 10.0.into(),
        },
        text_color: Some(chrome.text),
        shadow: iced::Shadow {
            color: chrome.panel_shadow,
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 10.0,
        },
        snap: false,
    }
}

fn alarm_row_style(_theme: &iced::Theme, chrome: WindowChrome) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(chrome.surface)),
        border: iced::Border {
            color: chrome.separator,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    }
}

fn separator_style(_theme: &iced::Theme, chrome: WindowChrome) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(chrome.separator)),
        ..container::Style::default()
    }
}

fn preset_button_style(
    _theme: &iced::Theme,
    status: button::Status,
    chrome: WindowChrome,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => chrome.surface_hover,
        _ => chrome.surface,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: chrome.text,
        border: iced::Border {
            color: chrome.panel_border,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn active_mode_style(
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

fn submit_button_style(
    _theme: &iced::Theme,
    status: button::Status,
    chrome: WindowChrome,
) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (chrome.success, chrome.success_text),
        _ => (chrome.success_soft, chrome.success_soft_text),
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

fn close_button_style(
    _theme: &iced::Theme,
    status: button::Status,
    chrome: WindowChrome,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => chrome.surface_hover,
        _ => chrome.surface,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: chrome.text,
        border: iced::Border {
            color: chrome.panel_border,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn edit_button_style(
    _theme: &iced::Theme,
    status: button::Status,
    chrome: WindowChrome,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => chrome.surface_hover,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: chrome.accent,
        border: iced::Border {
            radius: 3.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn delete_button_style(
    _theme: &iced::Theme,
    status: button::Status,
    chrome: WindowChrome,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => chrome.danger,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: chrome.danger,
        border: iced::Border {
            radius: 3.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

fn form_input_style(
    _theme: &iced::Theme,
    _status: text_input::Status,
    chrome: WindowChrome,
) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(chrome.input_background),
        border: iced::Border {
            color: chrome.input_border,
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: chrome.muted_text,
        placeholder: chrome.input_placeholder,
        value: chrome.text,
        selection: chrome.selection,
    }
}
