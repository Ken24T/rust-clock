//! Detached hover panel for reminder details.

use iced::alignment;
use iced::widget::{column, container, text};
use iced::{Color, Element, Fill, Padding};

use crate::clock_face::HoverWindowContent;
use crate::Message;

pub fn hover_panel(content: &HoverWindowContent) -> Element<'_, Message> {
    let body_lines = content.detail_lines.iter().fold(
        column![text(&content.title)
            .size(16)
            .color(Color::from_rgb(0.12, 0.12, 0.14))
            .align_x(alignment::Horizontal::Left)]
        .spacing(8),
        |column, line| {
            column.push(
                text(line)
                    .size(13)
                    .color(Color::from_rgb(0.18, 0.18, 0.20))
                    .align_x(alignment::Horizontal::Left),
            )
        },
    );

    container(body_lines)
        .width(Fill)
        .height(Fill)
        .padding(Padding::from([12, 14]))
        .style(panel_style)
        .into()
}

fn panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.96, 0.96, 0.94))),
        border: iced::Border {
            color: Color::from_rgb(0.42, 0.42, 0.40),
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: Some(Color::from_rgb(0.12, 0.12, 0.14)),
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.18),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        snap: false,
    }
}
