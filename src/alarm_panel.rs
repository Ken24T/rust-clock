//! Alarm panel overlay — a floating panel for managing alarms and timers.
//!
//! Displayed when the user selects "Alarms & Timers" from the context menu.
//! Provides:
//! - Quick-timer preset buttons
//! - A form to create timers (duration) or alarms (specific time)
//! - Optional message field for notifications
//! - Active alarm list with edit and delete
//! - Clear done / close buttons

use chrono::{Datelike, Local};
use iced::alignment;
use iced::widget::{
    button, center, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Color, Element, Fill, Length, Padding};

use crate::alarm::{
    Alarm, AlarmDateMonth, AlarmForm, AlarmFormMode, AlarmManager, AlarmRepeatMode,
    ScheduleWeekday, TimerRepeatMode,
};
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

/// Maximum body height before the panel content becomes scrollable.
const PANEL_BODY_MAX_HEIGHT: f32 = 420.0;

/// Build the alarm panel overlay as an iced Element.
pub fn alarm_panel<'a>(
    manager: &'a AlarmManager,
    form: &'a AlarmForm,
    chrome: WindowChrome,
) -> Element<'a, Message> {
    let heading = text("Alarms & Timers").size(15).color(chrome.text);
    let header_row = row![heading].align_y(alignment::Vertical::Center);

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

    // -- Managed reminders list --
    let managed: Vec<&Alarm> = manager.all().iter().filter(|alarm| alarm.enabled).collect();
    let running: Vec<&Alarm> = managed
        .iter()
        .copied()
        .filter(|alarm| alarm.is_live())
        .collect();
    let paused: Vec<&Alarm> = managed
        .iter()
        .copied()
        .filter(|alarm| alarm.is_paused())
        .collect();
    let done: Vec<&Alarm> = managed
        .iter()
        .copied()
        .filter(|alarm| alarm.is_completed())
        .collect();

    let alarm_list = if managed.is_empty() {
        column![text("No active alarms or timers")
            .size(11)
            .color(chrome.muted_text)]
    } else {
        let mut sections: Vec<Element<'_, Message>> = Vec::new();

        if !running.is_empty() {
            sections.push(section_label("Running", running.len(), chrome));
            let items: Vec<Element<'_, Message>> = running
                .iter()
                .map(|alarm| alarm_row(alarm, chrome))
                .collect();
            sections.push(column(items).spacing(3).into());
        }

        if !paused.is_empty() {
            if !sections.is_empty() {
                sections.push(separator_widget(chrome));
            }

            sections.push(section_label("Paused", paused.len(), chrome));
            let items: Vec<Element<'_, Message>> = paused
                .iter()
                .map(|alarm| alarm_row(alarm, chrome))
                .collect();
            sections.push(column(items).spacing(3).into());
        }

        if !done.is_empty() {
            if !sections.is_empty() {
                sections.push(separator_widget(chrome));
            }

            sections.push(section_label("Done", done.len(), chrome));
            let items: Vec<Element<'_, Message>> =
                done.iter().map(|alarm| alarm_row(alarm, chrome)).collect();
            sections.push(column(items).spacing(3).into());
        }

        column(sections).spacing(5)
    };

    let summary_row: Vec<Element<'_, Message>> = vec![
        text("Reminders").size(12).color(chrome.muted_text).into(),
        reminder_count_chip("Running", running.len(), false, chrome),
        reminder_count_chip("Paused", paused.len(), true, chrome),
        reminder_count_chip("Done", done.len(), true, chrome),
    ];

    let mut action_row: Vec<Element<'_, Message>> = Vec::new();

    if manager.paused_count() > 0 {
        action_row.push(
            button(
                text("Resume All")
                    .size(9)
                    .align_x(alignment::Horizontal::Center),
            )
            .on_press(Message::ResumeAllPaused)
            .padding(Padding::from([1, 6]))
            .style(move |theme, status| active_mode_style(theme, status, chrome))
            .into(),
        );
    }

    if manager.pausable_count() > 0 {
        action_row.push(
            button(
                text("Pause All")
                    .size(9)
                    .align_x(alignment::Horizontal::Center),
            )
            .on_press(Message::PauseAllRunning)
            .padding(Padding::from([1, 6]))
            .style(move |theme, status| preset_button_style(theme, status, chrome))
            .into(),
        );
    }

    let summary_label = row(summary_row)
        .spacing(5)
        .align_y(alignment::Vertical::Center);

    let list_label: Element<'_, Message> = if action_row.is_empty() {
        summary_label.into()
    } else {
        column![
            summary_label,
            row(action_row)
                .spacing(5)
                .align_y(alignment::Vertical::Center)
        ]
        .spacing(4)
        .into()
    };

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

    let mut body_items: Vec<Element<'_, Message>> = vec![presets.into(), separator_widget(chrome)];
    body_items.extend(build_form_elements(form, chrome));
    body_items.push(separator_widget(chrome));
    body_items.push(list_label);
    body_items.push(alarm_list.into());

    let body_content = column(body_items).spacing(6);
    let body: Element<'_, Message> = if managed.is_empty() {
        body_content.into()
    } else {
        scrollable(body_content)
            .height(Length::Fixed(PANEL_BODY_MAX_HEIGHT))
            .into()
    };

    let panel_col = column![
        header_row,
        separator,
        body,
        separator_widget(chrome),
        row(bottom_row).spacing(6),
    ]
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
    .padding(Padding::from([3, 8]))
    .style(move |theme, status| submit_button_style(theme, status, chrome));
    let submit_btn: Element<'_, Message> = if form.can_submit() {
        submit_btn.on_press(Message::AlarmFormSubmit).into()
    } else {
        submit_btn.into()
    };

    let cancel_label = if is_editing { "Cancel" } else { "Clear" };
    let cancel_btn = button(
        text(cancel_label)
            .size(11)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormCancel)
    .padding(Padding::from([3, 8]))
    .style(move |theme, status| close_button_style(theme, status, chrome));

    let action_row: Vec<Element<'_, Message>> = vec![submit_btn, cancel_btn.into()];

    let mut elements: Vec<Element<'_, Message>> = vec![
        heading.into(),
        mode_row.into(),
        label_input.into(),
        message_input.into(),
    ];

    // Add mode-specific fields directly (flattened).
    match form.mode {
        AlarmFormMode::Timer => {
            let timer_minutes_input = text_input("Minutes", &form.timer_minutes)
                .size(11)
                .padding(Padding::from([3, 6]))
                .style(move |theme, status| form_input_style(theme, status, chrome));
            let cadence_input = text_input("Time cadence (minutes)", &form.timer_cadence_minutes)
                .size(11)
                .padding(Padding::from([3, 6]))
                .style(move |theme, status| form_input_style(theme, status, chrome));

            elements.push(
                text("Timer cadence")
                    .size(10)
                    .color(chrome.muted_text)
                    .into(),
            );
            elements.push(
                build_choice_row(
                    vec![
                        (
                            "Once",
                            form.timer_repeat == TimerRepeatMode::Once,
                            Message::AlarmFormSetTimerRepeat(TimerRepeatMode::Once),
                        ),
                        (
                            "Repeats",
                            form.timer_repeat == TimerRepeatMode::Repeating,
                            Message::AlarmFormSetTimerRepeat(TimerRepeatMode::Repeating),
                        ),
                    ],
                    chrome,
                )
                .into(),
            );
            elements.push(if form.timer_repeat == TimerRepeatMode::Once {
                timer_minutes_input
                    .on_input(Message::AlarmFormMinutesChanged)
                    .into()
            } else {
                timer_minutes_input.into()
            });
            elements.push(if form.timer_repeat == TimerRepeatMode::Repeating {
                cadence_input
                    .on_input(Message::AlarmFormCadenceMinutesChanged)
                    .into()
            } else {
                cadence_input.into()
            });
        }
        AlarmFormMode::Alarm => {
            elements.push(text("Schedule").size(10).color(chrome.muted_text).into());
            elements.push(
                build_choice_row(
                    vec![
                        (
                            "Once",
                            form.alarm_repeat == AlarmRepeatMode::Once,
                            Message::AlarmFormSetAlarmRepeat(AlarmRepeatMode::Once),
                        ),
                        (
                            "Daily",
                            form.alarm_repeat == AlarmRepeatMode::Daily,
                            Message::AlarmFormSetAlarmRepeat(AlarmRepeatMode::Daily),
                        ),
                        (
                            "Weekdays",
                            form.alarm_repeat == AlarmRepeatMode::Weekdays,
                            Message::AlarmFormSetAlarmRepeat(AlarmRepeatMode::Weekdays),
                        ),
                    ],
                    chrome,
                )
                .into(),
            );
            elements.push(
                build_choice_row(
                    vec![
                        (
                            "Weekly",
                            form.alarm_repeat == AlarmRepeatMode::Weekly,
                            Message::AlarmFormSetAlarmRepeat(AlarmRepeatMode::Weekly),
                        ),
                        (
                            "Custom Days",
                            form.alarm_repeat == AlarmRepeatMode::SelectedWeekdays,
                            Message::AlarmFormSetAlarmRepeat(AlarmRepeatMode::SelectedWeekdays),
                        ),
                    ],
                    chrome,
                )
                .into(),
            );
            elements.push(
                text_input("Time (HH:MM)", &form.alarm_time)
                    .on_input(Message::AlarmFormTimeChanged)
                    .size(11)
                    .padding(Padding::from([3, 6]))
                    .style(move |theme, status| form_input_style(theme, status, chrome))
                    .into(),
            );
            match form.alarm_repeat {
                AlarmRepeatMode::Once => {
                    let selected_date = form.alarm_date_or_today();
                    let is_today_selected = selected_date == Local::now().date_naive();
                    let year_options: Vec<i32> =
                        ((selected_date.year() - 1)..=(selected_date.year() + 5)).collect();
                    let day_options: Vec<u32> = (1..=form.alarm_date_day_count()).collect();
                    let today_btn = button(
                        text(if is_today_selected {
                            "Today"
                        } else {
                            "Use Today"
                        })
                        .size(10)
                        .align_x(alignment::Horizontal::Center),
                    )
                    .padding(Padding::from([2, 6]))
                    .style(move |theme, status| preset_button_style(theme, status, chrome));
                    let today_btn: Element<'_, Message> = if is_today_selected {
                        today_btn.into()
                    } else {
                        today_btn.on_press(Message::AlarmFormSetDateToday).into()
                    };

                    elements.push(
                        row![
                            text("Pick Date").size(10).color(chrome.muted_text),
                            text(selected_date.format("%Y-%m-%d").to_string())
                                .size(10)
                                .color(chrome.muted_text),
                            today_btn,
                        ]
                        .spacing(6)
                        .align_y(alignment::Vertical::Center)
                        .into(),
                    );
                    elements.push(
                        row![
                            pick_list(
                                year_options,
                                Some(form.alarm_date_year()),
                                Message::AlarmFormSetDateYear,
                            )
                            .text_size(11)
                            .style(move |theme, status| {
                                date_pick_list_style(theme, status, chrome)
                            })
                            .menu_style(move |theme| date_pick_list_menu_style(theme, chrome))
                            .width(Length::FillPortion(2)),
                            pick_list(
                                AlarmDateMonth::ALL,
                                Some(form.alarm_date_month()),
                                Message::AlarmFormSetDateMonth,
                            )
                            .text_size(11)
                            .style(move |theme, status| {
                                date_pick_list_style(theme, status, chrome)
                            })
                            .menu_style(move |theme| date_pick_list_menu_style(theme, chrome))
                            .width(Length::FillPortion(2)),
                            pick_list(
                                day_options,
                                Some(form.alarm_date_day()),
                                Message::AlarmFormSetDateDay,
                            )
                            .text_size(11)
                            .style(move |theme, status| {
                                date_pick_list_style(theme, status, chrome)
                            })
                            .menu_style(move |theme| date_pick_list_menu_style(theme, chrome))
                            .width(Length::FillPortion(1)),
                        ]
                        .spacing(4)
                        .into(),
                    );
                }
                AlarmRepeatMode::Weekly => {
                    elements.push(text("Weekday").size(10).color(chrome.muted_text).into());
                    elements.push(build_weekday_row(form.weekly_weekday, chrome).into());
                }
                AlarmRepeatMode::SelectedWeekdays => {
                    elements.push(text("Days").size(10).color(chrome.muted_text).into());
                    elements
                        .push(build_selected_weekday_rows(&form.selected_weekdays, chrome).into());
                }
                AlarmRepeatMode::Daily | AlarmRepeatMode::Weekdays => {}
            }
        }
    }

    elements.push(row(action_row).spacing(6).into());
    elements
}

// -- Alarm row -------------------------------------------------------------

/// A single alarm row: label, status, kind badge, and reminder actions.
fn alarm_row<'a>(alarm: &'a Alarm, chrome: WindowChrome) -> Element<'a, Message> {
    let status_colour = if alarm.is_paused() {
        chrome.muted_text
    } else if alarm.fired {
        chrome.success
    } else {
        chrome.text
    };

    let label = text(&alarm.label).size(11).color(status_colour);
    let remaining = text(alarm.remaining_display())
        .size(10)
        .color(if alarm.is_paused() {
            chrome.muted_text
        } else {
            chrome.accent
        });
    let kind = text(alarm.kind_label()).size(9).color(chrome.muted_text);

    // Show message excerpt if present.
    let mut info_items: Vec<Element<'_, Message>> =
        vec![label.into(), row![remaining, kind].spacing(6).into()];
    if let Some(detail) = alarm.detail_text() {
        info_items.push(text(detail).size(9).color(chrome.muted_text).into());
    }
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

    let toggle_btn: Element<'_, Message> = if alarm.can_resume() {
        button(
            text("Resume")
                .size(9)
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(Message::ResumeAlarm(alarm.id))
        .padding(Padding::from([1, 4]))
        .style(move |theme, status| active_mode_style(theme, status, chrome))
        .into()
    } else if alarm.can_pause(chrono::Local::now()) {
        button(text("Pause").size(9).align_x(alignment::Horizontal::Center))
            .on_press(Message::PauseAlarm(alarm.id))
            .padding(Padding::from([1, 4]))
            .style(move |theme, status| preset_button_style(theme, status, chrome))
            .into()
    } else {
        button(text("Done").size(9).align_x(alignment::Horizontal::Center))
            .padding(Padding::from([1, 4]))
            .style(move |theme, status| close_button_style(theme, status, chrome))
            .into()
    };

    let edit_btn = button(text("✎").size(10).align_x(alignment::Horizontal::Center))
        .on_press(Message::EditAlarm(alarm.id))
        .padding(Padding::from([1, 4]))
        .style(move |theme, status| edit_button_style(theme, status, chrome));

    let delete_btn = button(text("✕").size(10).align_x(alignment::Horizontal::Center))
        .on_press(Message::RemoveAlarm(alarm.id))
        .padding(Padding::from([1, 4]))
        .style(move |theme, status| delete_button_style(theme, status, chrome));

    let info_col = column(info_items).spacing(1);
    let btn_col = row![toggle_btn, edit_btn, delete_btn].spacing(2);

    container(row![info_col, btn_col].spacing(6))
        .padding(Padding::from([3, 6]))
        .style(move |theme| alarm_row_style(theme, chrome, alarm.is_paused()))
        .width(Fill)
        .into()
}

fn section_label<'a>(title: &'a str, count: usize, chrome: WindowChrome) -> Element<'a, Message> {
    row![
        text(title).size(10).color(chrome.muted_text),
        text(format!("{count}"))
            .size(10)
            .color(chrome.accent_soft_text)
    ]
    .spacing(6)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn reminder_count_chip<'a>(
    label: &'a str,
    count: usize,
    paused: bool,
    chrome: WindowChrome,
) -> Element<'a, Message> {
    let background = if paused {
        chrome.panel_background
    } else {
        chrome.accent_soft
    };
    let text_colour = if paused {
        chrome.muted_text
    } else {
        chrome.accent_soft_text
    };
    let border = if paused {
        chrome.panel_border
    } else {
        chrome.accent
    };

    container(text(format!("{label}: {count}")).size(9).color(text_colour))
        .padding(Padding::from([1, 6]))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(background)),
            border: iced::Border {
                color: border,
                width: 1.0,
                radius: 999.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn build_choice_row(
    items: Vec<(&'static str, bool, Message)>,
    chrome: WindowChrome,
) -> iced::widget::Row<'static, Message> {
    let buttons: Vec<Element<'static, Message>> = items
        .into_iter()
        .map(|(label, active, message)| {
            button(text(label).size(10).align_x(alignment::Horizontal::Center))
                .on_press(message)
                .padding(Padding::from([2, 6]))
                .style(move |theme, status| {
                    if active {
                        active_mode_style(theme, status, chrome)
                    } else {
                        preset_button_style(theme, status, chrome)
                    }
                })
                .into()
        })
        .collect();
    row(buttons).spacing(4)
}

fn build_weekday_row(
    selected: ScheduleWeekday,
    chrome: WindowChrome,
) -> iced::widget::Row<'static, Message> {
    let buttons: Vec<Element<'static, Message>> = ScheduleWeekday::ALL
        .into_iter()
        .map(|weekday| {
            button(
                text(weekday.short_label())
                    .size(10)
                    .align_x(alignment::Horizontal::Center),
            )
            .on_press(Message::AlarmFormSetWeeklyWeekday(weekday))
            .padding(Padding::from([2, 5]))
            .style(move |theme, status| {
                if weekday == selected {
                    active_mode_style(theme, status, chrome)
                } else {
                    preset_button_style(theme, status, chrome)
                }
            })
            .into()
        })
        .collect();
    row(buttons).spacing(3)
}

fn build_selected_weekday_rows(
    selected_days: &[ScheduleWeekday],
    chrome: WindowChrome,
) -> iced::widget::Column<'static, Message> {
    let top: Vec<Element<'static, Message>> = ScheduleWeekday::ALL[..4]
        .iter()
        .copied()
        .map(|weekday| build_selected_weekday_button(weekday, selected_days, chrome))
        .collect();
    let bottom: Vec<Element<'static, Message>> = ScheduleWeekday::ALL[4..]
        .iter()
        .copied()
        .map(|weekday| build_selected_weekday_button(weekday, selected_days, chrome))
        .collect();

    column![row(top).spacing(3), row(bottom).spacing(3)].spacing(3)
}

fn build_selected_weekday_button(
    weekday: ScheduleWeekday,
    selected_days: &[ScheduleWeekday],
    chrome: WindowChrome,
) -> Element<'static, Message> {
    let active = selected_days.contains(&weekday);
    button(
        text(weekday.short_label())
            .size(10)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AlarmFormToggleSelectedWeekday(weekday))
    .padding(Padding::from([2, 5]))
    .style(move |theme, status| {
        if active {
            active_mode_style(theme, status, chrome)
        } else {
            preset_button_style(theme, status, chrome)
        }
    })
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

fn alarm_row_style(_theme: &iced::Theme, chrome: WindowChrome, paused: bool) -> container::Style {
    let background = if paused {
        chrome.panel_background
    } else {
        chrome.surface
    };
    let border = if paused {
        chrome.panel_border
    } else {
        chrome.separator
    };

    container::Style {
        background: Some(iced::Background::Color(background)),
        border: iced::Border {
            color: border,
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
        button::Status::Disabled => (chrome.separator, chrome.muted_text),
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

fn date_pick_list_style(
    _theme: &iced::Theme,
    status: iced::widget::pick_list::Status,
    chrome: WindowChrome,
) -> iced::widget::pick_list::Style {
    let border_color = match status {
        iced::widget::pick_list::Status::Hovered
        | iced::widget::pick_list::Status::Opened { .. } => chrome.accent,
        iced::widget::pick_list::Status::Active => chrome.input_border,
    };

    iced::widget::pick_list::Style {
        text_color: chrome.text,
        placeholder_color: chrome.input_placeholder,
        handle_color: chrome.muted_text,
        background: iced::Background::Color(chrome.input_background),
        border: iced::Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
    }
}

fn date_pick_list_menu_style(
    _theme: &iced::Theme,
    chrome: WindowChrome,
) -> iced::widget::overlay::menu::Style {
    iced::widget::overlay::menu::Style {
        background: iced::Background::Color(chrome.panel_background),
        border: iced::Border {
            color: chrome.panel_border,
            width: 1.0,
            radius: 6.0.into(),
        },
        text_color: chrome.text,
        selected_text_color: chrome.accent_soft_text,
        selected_background: iced::Background::Color(chrome.accent_soft),
        shadow: iced::Shadow {
            color: chrome.panel_shadow,
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
    }
}
