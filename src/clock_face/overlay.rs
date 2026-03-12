//! Overlay drawing helpers for active alarm/timer summaries.

use iced::alignment;
use iced::widget::canvas::{self, Frame};
use iced::{Point, Rectangle};

use crate::alarm::FaceActiveItem;

use super::ClockFace;

const MIN_SUMMARY_LANE_RADIUS: f32 = 90.0;
const MAX_VISIBLE_SUMMARY_LINES: usize = 2;
const SUMMARY_LANE_WIDTH_FACTOR: f32 = 1.06;
const SUMMARY_LANE_SINGLE_HEIGHT_FACTOR: f32 = 0.16;
const SUMMARY_LANE_MULTI_HEIGHT_FACTOR: f32 = 0.25;
const SUMMARY_LANE_VERTICAL_OFFSET_FACTOR: f32 = 0.52;
const SUMMARY_LANE_TEXT_SIZE_FACTOR: f32 = 0.11;
const SUMMARY_LANE_SHADOW_OFFSET_FACTOR: f32 = 0.01;
const SUMMARY_LANE_LINE_SPACING_FACTOR: f32 = 1.05;

#[derive(Debug, Clone, Copy, PartialEq)]
struct SummaryLaneLayout {
    bounds: Rectangle,
    text_positions: [Point; MAX_VISIBLE_SUMMARY_LINES],
    text_size: f32,
    max_chars: usize,
    line_count: usize,
}

impl ClockFace {
    /// Draw overlays that sit above the face and hands.
    pub(super) fn draw_overlay(&self, frame: &mut Frame, centre: Point, radius: f32) {
        if self.active_items.is_empty() {
            return;
        }

        let visible_line_count = self.active_items.len().min(MAX_VISIBLE_SUMMARY_LINES);

        let Some(layout) = summary_lane_layout(centre, radius, visible_line_count) else {
            return;
        };

        let summaries = summary_lane_texts(&self.active_items, layout.max_chars, layout.line_count);
        let shadow_offset = radius * SUMMARY_LANE_SHADOW_OFFSET_FACTOR;

        for (index, summary) in summaries.iter().enumerate() {
            let position = layout.text_positions[index];

            frame.fill_text(canvas::Text {
                content: summary.clone(),
                position: Point::new(position.x + shadow_offset, position.y + shadow_offset),
                size: layout.text_size.into(),
                color: self.theme.shadow_colour.into(),
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..canvas::Text::default()
            });

            frame.fill_text(canvas::Text {
                content: summary.clone(),
                position,
                size: layout.text_size.into(),
                color: self.theme.numeral_colour.into(),
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }
    }
}

fn summary_lane_layout(centre: Point, radius: f32, line_count: usize) -> Option<SummaryLaneLayout> {
    if radius < MIN_SUMMARY_LANE_RADIUS {
        return None;
    }

    let line_count = line_count.clamp(1, MAX_VISIBLE_SUMMARY_LINES);
    let width = radius * SUMMARY_LANE_WIDTH_FACTOR;
    let height = radius
        * if line_count == 1 {
            SUMMARY_LANE_SINGLE_HEIGHT_FACTOR
        } else {
            SUMMARY_LANE_MULTI_HEIGHT_FACTOR
        };
    let x = centre.x - width / 2.0;
    let y = centre.y + radius * SUMMARY_LANE_VERTICAL_OFFSET_FACTOR - height / 2.0;
    let text_size = (radius * SUMMARY_LANE_TEXT_SIZE_FACTOR).clamp(12.0, 22.0);
    let max_chars = ((width / (text_size * 0.62)).floor() as usize).max(12);
    let text_positions = lane_text_positions(x, y, width, height, text_size, line_count);

    Some(SummaryLaneLayout {
        bounds: Rectangle {
            x,
            y,
            width,
            height,
        },
        text_positions,
        text_size,
        max_chars,
        line_count,
    })
}

fn lane_text_positions(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text_size: f32,
    line_count: usize,
) -> [Point; MAX_VISIBLE_SUMMARY_LINES] {
    let centre_x = x + width / 2.0;
    let centre_y = y + height / 2.0;

    if line_count == 1 {
        [
            Point::new(centre_x, centre_y),
            Point::new(centre_x, centre_y),
        ]
    } else {
        let offset = text_size * SUMMARY_LANE_LINE_SPACING_FACTOR / 2.0;
        [
            Point::new(centre_x, centre_y - offset),
            Point::new(centre_x, centre_y + offset),
        ]
    }
}

fn summary_lane_texts(
    items: &[FaceActiveItem],
    max_chars: usize,
    line_count: usize,
) -> Vec<String> {
    let visible_items = items.iter().take(line_count).collect::<Vec<_>>();
    let extra_count = items.len().saturating_sub(visible_items.len());

    visible_items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let overflow_suffix = if index + 1 == line_count && extra_count > 0 {
                format!(" +{extra_count} more")
            } else {
                String::new()
            };

            summary_lane_text(item, max_chars, &overflow_suffix)
        })
        .collect()
}

fn summary_lane_text(item: &FaceActiveItem, max_chars: usize, trailing_suffix: &str) -> String {
    let suffix = format!(" - {}{trailing_suffix}", item.remaining_text);
    let minimum_label_chars = 6;

    if max_chars <= suffix.chars().count() + minimum_label_chars {
        return truncate_text(
            &format!("{}{}", item.remaining_text, trailing_suffix),
            max_chars,
        );
    }

    let label_budget = max_chars.saturating_sub(suffix.chars().count());
    let label = truncate_text(&item.label, label_budget);

    format!("{label}{suffix}")
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    let text_len = text.chars().count();
    if text_len <= max_chars {
        return text.to_string();
    }

    if max_chars <= 3 {
        return text.chars().take(max_chars).collect();
    }

    let visible_chars = max_chars - 3;
    let prefix: String = text.chars().take(visible_chars).collect();
    format!("{prefix}...")
}

#[cfg(test)]
mod tests {
    use super::{summary_lane_layout, summary_lane_text, summary_lane_texts};
    use crate::alarm::{FaceActiveItem, FaceActiveItemKind};
    use chrono::Local;
    use iced::Point;
    use uuid::Uuid;

    fn sample_item(label: &str, remaining_text: &str) -> FaceActiveItem {
        FaceActiveItem {
            id: Uuid::nil(),
            label: label.to_string(),
            kind: FaceActiveItemKind::Timer,
            target: Local::now(),
            remaining_text: remaining_text.to_string(),
        }
    }

    #[test]
    fn summary_lane_layout_exists_for_medium_clock() {
        let centre = Point::new(125.0, 125.0);
        let layout =
            summary_lane_layout(centre, 119.0, 1).expect("medium clock should show a lane");

        assert!(layout.bounds.y > centre.y);
        assert!(layout.text_size >= 12.0);
        assert!(layout.max_chars >= 12);
        assert_eq!(layout.line_count, 1);
    }

    #[test]
    fn summary_lane_layout_is_hidden_for_small_clock() {
        let centre = Point::new(75.0, 75.0);
        assert!(summary_lane_layout(centre, 71.0, 1).is_none());
    }

    #[test]
    fn summary_lane_text_preserves_remaining_time() {
        let item = sample_item("Tea", "4m 10s");
        assert_eq!(summary_lane_text(&item, 24, ""), "Tea - 4m 10s");
    }

    #[test]
    fn summary_lane_text_truncates_long_labels() {
        let item = sample_item("Very long reminder label", "12m 0s");
        let summary = summary_lane_text(&item, 18, "");

        assert!(summary.ends_with(" - 12m 0s"));
        assert!(summary.contains("..."));
    }

    #[test]
    fn summary_lane_texts_include_overflow_indicator_on_last_line() {
        let items = vec![
            sample_item("Tea", "4m 10s"),
            sample_item("Laundry", "8m 0s"),
            sample_item("Meeting", "22m 0s"),
        ];

        let summaries = summary_lane_texts(&items, 28, 2);

        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0], "Tea - 4m 10s");
        assert!(summaries[1].ends_with(" - 8m 0s +1 more"));
    }

    #[test]
    fn summary_lane_layout_supports_two_lines() {
        let centre = Point::new(125.0, 125.0);
        let layout =
            summary_lane_layout(centre, 119.0, 2).expect("medium clock should show two lines");

        assert_eq!(layout.line_count, 2);
        assert!(layout.text_positions[0].y < layout.text_positions[1].y);
        assert!(layout.bounds.height > 20.0);
    }
}
