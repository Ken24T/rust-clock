//! Detached hover panel for reminder details.

use iced::alignment;
use iced::widget::{column, container, text};
use iced::{Element, Fill, Padding};

use crate::clock_face::HoverWindowContent;
use crate::theme::WindowChrome;
use crate::Message;

pub fn hover_panel(content: &HoverWindowContent, chrome: WindowChrome) -> Element<'_, Message> {
    let body_lines = content.detail_lines.iter().fold(
        column![text(&content.title)
            .size(16)
            .color(chrome.text)
            .align_x(alignment::Horizontal::Left)]
        .spacing(8),
        |column, line| {
            column.push(
                text(line)
                    .size(13)
                    .color(chrome.muted_text)
                    .align_x(alignment::Horizontal::Left),
            )
        },
    );

    container(body_lines)
        .width(Fill)
        .height(Fill)
        .padding(Padding::from([12, 14]))
        .style(move |theme| panel_style(theme, chrome))
        .into()
}

fn panel_style(_theme: &iced::Theme, chrome: WindowChrome) -> container::Style {
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
