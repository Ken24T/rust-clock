//! Overlay drawing helpers for active alarm/timer summaries.

use iced::alignment;
use iced::widget::canvas::{self, Frame};
use iced::{Point, Rectangle};

use crate::alarm::FaceActiveItem;

use super::ClockFace;

const MIN_SUMMARY_LANE_RADIUS: f32 = 90.0;
const SUMMARY_LANE_WIDTH_FACTOR: f32 = 1.06;
const SUMMARY_LANE_HEIGHT_FACTOR: f32 = 0.16;
const SUMMARY_LANE_VERTICAL_OFFSET_FACTOR: f32 = 0.52;
const SUMMARY_LANE_TEXT_SIZE_FACTOR: f32 = 0.11;
const SUMMARY_LANE_SHADOW_OFFSET_FACTOR: f32 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq)]
struct SummaryLaneLayout {
    bounds: Rectangle,
    text_position: Point,
    text_size: f32,
    max_chars: usize,
}

impl ClockFace {
    /// Draw overlays that sit above the face and hands.
    pub(super) fn draw_overlay(&self, frame: &mut Frame, centre: Point, radius: f32) {
        let Some(item) = self.active_items.first() else {
            return;
        };

        let Some(layout) = summary_lane_layout(centre, radius) else {
            return;
        };

        let summary = summary_lane_text(item, layout.max_chars);
        let shadow_offset = radius * SUMMARY_LANE_SHADOW_OFFSET_FACTOR;

        frame.fill_text(canvas::Text {
            content: summary.clone(),
            position: Point::new(
                layout.text_position.x + shadow_offset,
                layout.text_position.y + shadow_offset,
            ),
            size: layout.text_size.into(),
            color: self.theme.shadow_colour.into(),
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..canvas::Text::default()
        });

        frame.fill_text(canvas::Text {
            content: summary,
            position: layout.text_position,
            size: layout.text_size.into(),
            color: self.theme.numeral_colour.into(),
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..canvas::Text::default()
        });
    }
}

fn summary_lane_layout(centre: Point, radius: f32) -> Option<SummaryLaneLayout> {
    if radius < MIN_SUMMARY_LANE_RADIUS {
        return None;
    }

    let width = radius * SUMMARY_LANE_WIDTH_FACTOR;
    let height = radius * SUMMARY_LANE_HEIGHT_FACTOR;
    let x = centre.x - width / 2.0;
    let y = centre.y + radius * SUMMARY_LANE_VERTICAL_OFFSET_FACTOR - height / 2.0;
    let text_size = (radius * SUMMARY_LANE_TEXT_SIZE_FACTOR).clamp(12.0, 22.0);
    let max_chars = ((width / (text_size * 0.62)).floor() as usize).max(12);

    Some(SummaryLaneLayout {
        bounds: Rectangle {
            x,
            y,
            width,
            height,
        },
        text_position: Point::new(x + width / 2.0, y + height / 2.0),
        text_size,
        max_chars,
    })
}

fn summary_lane_text(item: &FaceActiveItem, max_chars: usize) -> String {
    let suffix = format!(" - {}", item.remaining_text);
    let minimum_label_chars = 6;

    if max_chars <= suffix.chars().count() + minimum_label_chars {
        return item.remaining_text.clone();
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
    use super::{summary_lane_layout, summary_lane_text};
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
        let layout = summary_lane_layout(centre, 119.0).expect("medium clock should show a lane");

        assert!(layout.bounds.y > centre.y);
        assert!(layout.text_size >= 12.0);
        assert!(layout.max_chars >= 12);
    }

    #[test]
    fn summary_lane_layout_is_hidden_for_small_clock() {
        let centre = Point::new(75.0, 75.0);
        assert!(summary_lane_layout(centre, 71.0).is_none());
    }

    #[test]
    fn summary_lane_text_preserves_remaining_time() {
        let item = sample_item("Tea", "4m 10s");
        assert_eq!(summary_lane_text(&item, 24), "Tea - 4m 10s");
    }

    #[test]
    fn summary_lane_text_truncates_long_labels() {
        let item = sample_item("Very long reminder label", "12m 0s");
        let summary = summary_lane_text(&item, 18);

        assert!(summary.ends_with(" - 12m 0s"));
        assert!(summary.contains("..."));
    }
}
