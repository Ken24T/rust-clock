//! Overlay drawing helpers for active alarm/timer summaries.

use iced::alignment;
use iced::widget::canvas::{self, stroke, Frame, Path, Stroke};
use iced::{Color, Point, Rectangle, Size};
use uuid::Uuid;

use crate::alarm::{FaceActiveItem, FaceActiveItemKind};

use super::ClockFace;

const MIN_MINIMAL_INDICATOR_RADIUS: f32 = 60.0;
const MIN_REDUCED_LANE_RADIUS: f32 = 78.0;
const MIN_FULL_SUMMARY_LANE_RADIUS: f32 = 90.0;
const MAX_VISIBLE_SUMMARY_LINES: usize = 2;
const SUMMARY_LANE_WIDTH_FACTOR: f32 = 1.06;
const SUMMARY_LANE_SINGLE_HEIGHT_FACTOR: f32 = 0.16;
const SUMMARY_LANE_MULTI_HEIGHT_FACTOR: f32 = 0.25;
const SUMMARY_LANE_VERTICAL_OFFSET_FACTOR: f32 = 0.52;
const SUMMARY_LANE_TEXT_SIZE_FACTOR: f32 = 0.11;
const SUMMARY_LANE_SHADOW_OFFSET_FACTOR: f32 = 0.01;
const SUMMARY_LANE_LINE_SPACING_FACTOR: f32 = 1.05;
const SUMMARY_LANE_TEXT_WIDTH_FACTOR: f32 = 0.62;
const SUMMARY_LANE_LINE_HIT_HEIGHT_FACTOR: f32 = 1.35;
const HOVER_DETAIL_WIDTH_FACTOR: f32 = 1.08;
const HOVER_DETAIL_HEIGHT_FACTOR: f32 = 0.28;
const HOVER_DETAIL_VERTICAL_GAP_FACTOR: f32 = 0.08;
const HOVER_DETAIL_TITLE_SIZE_FACTOR: f32 = 0.10;
const HOVER_DETAIL_SUBTITLE_SIZE_FACTOR: f32 = 0.075;
const HOVER_DETAIL_LINE_SPACING_FACTOR: f32 = 1.15;

#[derive(Debug, Clone, Copy, PartialEq)]
struct SummaryLaneLayout {
    bounds: Rectangle,
    text_positions: [Point; MAX_VISIBLE_SUMMARY_LINES],
    text_size: f32,
    max_chars: usize,
    line_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OverlayHitTarget {
    SummaryItem(Uuid),
    OverflowIndicator {
        anchor_id: Uuid,
        hidden_count: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct OverlayHitRegion {
    target: OverlayHitTarget,
    bounds: Rectangle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HoverDetail {
    anchor_index: usize,
    title: String,
    subtitle: String,
}

#[derive(Debug, Clone, Copy)]
struct HoverDetailStyle {
    face_colour: Color,
    border_colour: Color,
    text_colour: Color,
    shadow_colour: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayLayoutMode {
    FullLane,
    ReducedLane,
    MinimalIndicator,
}

impl ClockFace {
    /// Draw overlays that sit above the face and hands.
    pub(super) fn draw_overlay(
        &self,
        frame: &mut Frame,
        centre: Point,
        radius: f32,
        hovered_target: Option<OverlayHitTarget>,
    ) {
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

        if let Some(detail) = hover_detail(&self.active_items, layout.line_count, hovered_target) {
            draw_hover_detail(
                frame,
                &layout,
                radius,
                detail,
                HoverDetailStyle {
                    face_colour: self.theme.face_colour.into(),
                    border_colour: self.theme.border_colour.into(),
                    text_colour: self.theme.numeral_colour.into(),
                    shadow_colour: self.theme.shadow_colour.into(),
                },
            );
        }
    }

    pub(super) fn overlay_hit_target(
        &self,
        cursor_position: Point,
        centre: Point,
        radius: f32,
    ) -> Option<OverlayHitTarget> {
        overlay_hit_regions(&self.active_items, centre, radius)
            .into_iter()
            .find(|region| rectangle_contains_point(region.bounds, cursor_position))
            .map(|region| region.target)
    }
}

fn overlay_layout_mode(radius: f32) -> Option<OverlayLayoutMode> {
    if radius >= MIN_FULL_SUMMARY_LANE_RADIUS {
        Some(OverlayLayoutMode::FullLane)
    } else if radius >= MIN_REDUCED_LANE_RADIUS {
        Some(OverlayLayoutMode::ReducedLane)
    } else if radius >= MIN_MINIMAL_INDICATOR_RADIUS {
        Some(OverlayLayoutMode::MinimalIndicator)
    } else {
        None
    }
}

fn summary_lane_layout(centre: Point, radius: f32, line_count: usize) -> Option<SummaryLaneLayout> {
    if overlay_layout_mode(radius) != Some(OverlayLayoutMode::FullLane) {
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

fn overlay_hit_regions(
    items: &[FaceActiveItem],
    centre: Point,
    radius: f32,
) -> Vec<OverlayHitRegion> {
    if items.is_empty() {
        return Vec::new();
    }

    let visible_line_count = items.len().min(MAX_VISIBLE_SUMMARY_LINES);
    let Some(layout) = summary_lane_layout(centre, radius, visible_line_count) else {
        return Vec::new();
    };

    let summaries = summary_lane_texts(items, layout.max_chars, layout.line_count);
    let extra_count = items.len().saturating_sub(layout.line_count);
    let mut regions = Vec::new();

    for (index, item) in items.iter().take(layout.line_count).enumerate() {
        let line_bounds = summary_line_hit_bounds(&layout, index);

        if index + 1 == layout.line_count && extra_count > 0 {
            let overflow_suffix = format!(" +{extra_count} more");
            let overflow_bounds = overflow_hit_bounds(
                &layout,
                line_bounds,
                index,
                &summaries[index],
                &overflow_suffix,
            );

            regions.push(OverlayHitRegion {
                target: OverlayHitTarget::OverflowIndicator {
                    anchor_id: item.id,
                    hidden_count: extra_count,
                },
                bounds: overflow_bounds,
            });
        }

        regions.push(OverlayHitRegion {
            target: OverlayHitTarget::SummaryItem(item.id),
            bounds: line_bounds,
        });
    }

    regions.sort_by_key(|region| match region.target {
        OverlayHitTarget::OverflowIndicator { .. } => 0,
        OverlayHitTarget::SummaryItem(_) => 1,
    });

    regions
}

fn summary_line_hit_bounds(layout: &SummaryLaneLayout, index: usize) -> Rectangle {
    let position = layout.text_positions[index];
    let line_height = layout.text_size * SUMMARY_LANE_LINE_HIT_HEIGHT_FACTOR;

    Rectangle {
        x: layout.bounds.x,
        y: position.y - line_height / 2.0,
        width: layout.bounds.width,
        height: line_height,
    }
}

fn overflow_hit_bounds(
    layout: &SummaryLaneLayout,
    line_bounds: Rectangle,
    index: usize,
    summary_text: &str,
    overflow_suffix: &str,
) -> Rectangle {
    let total_width = estimated_text_width(summary_text, layout.text_size);
    let overflow_width = estimated_text_width(overflow_suffix, layout.text_size);
    let line_centre_x = layout.text_positions[index].x;
    let overflow_centre_x = line_centre_x + total_width / 2.0 - overflow_width / 2.0;
    let overflow_x = (overflow_centre_x - overflow_width / 2.0).max(layout.bounds.x);
    let overflow_right = (overflow_x + overflow_width).min(layout.bounds.x + layout.bounds.width);

    Rectangle {
        x: overflow_x,
        y: line_bounds.y,
        width: (overflow_right - overflow_x).max(0.0),
        height: line_bounds.height,
    }
}

fn estimated_text_width(text: &str, text_size: f32) -> f32 {
    text.chars().count() as f32 * text_size * SUMMARY_LANE_TEXT_WIDTH_FACTOR
}

fn rectangle_contains_point(bounds: Rectangle, point: Point) -> bool {
    point.x >= bounds.x
        && point.x <= bounds.x + bounds.width
        && point.y >= bounds.y
        && point.y <= bounds.y + bounds.height
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

fn hover_detail(
    items: &[FaceActiveItem],
    visible_line_count: usize,
    hovered_target: Option<OverlayHitTarget>,
) -> Option<HoverDetail> {
    let hovered_target = hovered_target?;
    let visible_items = items.iter().take(visible_line_count).collect::<Vec<_>>();

    match hovered_target {
        OverlayHitTarget::SummaryItem(id) => visible_items
            .iter()
            .enumerate()
            .find(|(_, item)| item.id == id)
            .map(|(anchor_index, item)| HoverDetail {
                anchor_index,
                title: item.label.clone(),
                subtitle: format!(
                    "{} • {}",
                    match item.kind {
                        FaceActiveItemKind::Alarm => "alarm",
                        FaceActiveItemKind::Timer => "timer",
                    },
                    item.remaining_text
                ),
            }),
        OverlayHitTarget::OverflowIndicator {
            anchor_id,
            hidden_count,
        } => {
            if hidden_count == 0 {
                return None;
            }

            visible_items
                .iter()
                .enumerate()
                .find(|(_, item)| item.id == anchor_id)
                .map(|(anchor_index, _)| HoverDetail {
                    anchor_index,
                    title: format!(
                        "{hidden_count} more active reminder{}",
                        if hidden_count == 1 { "" } else { "s" }
                    ),
                    subtitle: "Open Alarms & Timers to view the full list".to_string(),
                })
        }
    }
}

fn draw_hover_detail(
    frame: &mut Frame,
    layout: &SummaryLaneLayout,
    radius: f32,
    detail: HoverDetail,
    style: HoverDetailStyle,
) {
    let width = radius * HOVER_DETAIL_WIDTH_FACTOR;
    let height = radius * HOVER_DETAIL_HEIGHT_FACTOR;
    let gap = radius * HOVER_DETAIL_VERTICAL_GAP_FACTOR;
    let title_size = (radius * HOVER_DETAIL_TITLE_SIZE_FACTOR).clamp(12.0, 20.0);
    let subtitle_size = (radius * HOVER_DETAIL_SUBTITLE_SIZE_FACTOR).clamp(10.0, 15.0);
    let anchor_x = layout.text_positions[detail.anchor_index].x;
    let min_x = layout.bounds.x;
    let max_x = layout.bounds.x + layout.bounds.width - width;
    let x = (anchor_x - width / 2.0).clamp(min_x, max_x);
    let y = layout.bounds.y - height - gap;

    let panel = Path::rectangle(Point::new(x, y), Size::new(width, height));
    let mut background = style.face_colour;
    background.a = background.a.max(0.88);

    frame.fill(&panel, background);
    frame.stroke(
        &panel,
        Stroke {
            style: stroke::Style::Solid(style.border_colour),
            width: 1.0,
            ..Stroke::default()
        },
    );

    let shadow_offset = radius * SUMMARY_LANE_SHADOW_OFFSET_FACTOR;
    let centre_x = x + width / 2.0;
    let centre_y = y + height / 2.0;
    let line_offset = title_size * HOVER_DETAIL_LINE_SPACING_FACTOR / 2.0;
    let title_position = Point::new(centre_x, centre_y - line_offset);
    let subtitle_position = Point::new(centre_x, centre_y + line_offset * 0.75);

    for (content, position, size) in [
        (detail.title, title_position, title_size),
        (detail.subtitle, subtitle_position, subtitle_size),
    ] {
        frame.fill_text(canvas::Text {
            content: content.clone(),
            position: Point::new(position.x + shadow_offset, position.y + shadow_offset),
            size: size.into(),
            color: style.shadow_colour,
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..canvas::Text::default()
        });

        frame.fill_text(canvas::Text {
            content,
            position,
            size: size.into(),
            color: style.text_colour,
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..canvas::Text::default()
        });
    }
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
    use super::{
        hover_detail, overlay_hit_regions, overlay_layout_mode, rectangle_contains_point,
        summary_lane_layout, summary_lane_text, summary_lane_texts, HoverDetail, OverlayHitTarget,
        OverlayLayoutMode,
    };
    use crate::alarm::{FaceActiveItem, FaceActiveItemKind};
    use chrono::Local;
    use iced::Point;
    use uuid::Uuid;

    fn sample_item(label: &str, remaining_text: &str) -> FaceActiveItem {
        FaceActiveItem {
            id: Uuid::new_v4(),
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
    fn overlay_layout_mode_uses_full_lane_for_medium_and_up() {
        assert_eq!(
            overlay_layout_mode(119.0),
            Some(OverlayLayoutMode::FullLane)
        );
        assert_eq!(
            overlay_layout_mode(166.0),
            Some(OverlayLayoutMode::FullLane)
        );
    }

    #[test]
    fn overlay_layout_mode_uses_reduced_lane_for_intermediate_sizes() {
        assert_eq!(
            overlay_layout_mode(84.0),
            Some(OverlayLayoutMode::ReducedLane)
        );
    }

    #[test]
    fn overlay_layout_mode_uses_minimal_indicator_for_small_clock_sizes() {
        assert_eq!(
            overlay_layout_mode(71.25),
            Some(OverlayLayoutMode::MinimalIndicator)
        );
    }

    #[test]
    fn overlay_layout_mode_hides_overlay_below_minimal_threshold() {
        assert_eq!(overlay_layout_mode(55.0), None);
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

    #[test]
    fn overlay_hit_regions_include_rows_for_visible_items() {
        let items = vec![
            sample_item("Tea", "4m 10s"),
            sample_item("Laundry", "8m 0s"),
        ];
        let regions = overlay_hit_regions(&items, Point::new(125.0, 125.0), 119.0);

        assert_eq!(regions.len(), 2);
        assert!(matches!(
            regions[0].target,
            OverlayHitTarget::SummaryItem(_)
        ));
        assert!(matches!(
            regions[1].target,
            OverlayHitTarget::SummaryItem(_)
        ));
    }

    #[test]
    fn overlay_hit_regions_include_overflow_target_before_row_target() {
        let items = vec![
            sample_item("Tea", "4m 10s"),
            sample_item("Laundry", "8m 0s"),
            sample_item("Meeting", "22m 0s"),
        ];
        let regions = overlay_hit_regions(&items, Point::new(125.0, 125.0), 119.0);

        assert_eq!(regions.len(), 3);
        assert!(matches!(
            regions[0].target,
            OverlayHitTarget::OverflowIndicator {
                hidden_count: 1,
                ..
            }
        ));
        assert!(matches!(
            regions[1].target,
            OverlayHitTarget::SummaryItem(_)
        ));
        assert!(matches!(
            regions[2].target,
            OverlayHitTarget::SummaryItem(_)
        ));
    }

    #[test]
    fn overflow_region_contains_its_estimated_text_area() {
        let items = vec![
            sample_item("Tea", "4m 10s"),
            sample_item("Laundry", "8m 0s"),
            sample_item("Meeting", "22m 0s"),
        ];
        let regions = overlay_hit_regions(&items, Point::new(125.0, 125.0), 119.0);
        let overflow_region = regions
            .iter()
            .find(|region| matches!(region.target, OverlayHitTarget::OverflowIndicator { .. }))
            .expect("expected overflow region");

        let sample_point = Point::new(
            overflow_region.bounds.x + overflow_region.bounds.width / 2.0,
            overflow_region.bounds.y + overflow_region.bounds.height / 2.0,
        );

        assert!(rectangle_contains_point(
            overflow_region.bounds,
            sample_point
        ));
    }

    #[test]
    fn hover_detail_uses_visible_item_label_and_status() {
        let items = vec![sample_item("Tea", "4m 10s")];
        let detail = hover_detail(&items, 1, Some(OverlayHitTarget::SummaryItem(items[0].id)))
            .expect("expected item hover detail");

        assert_eq!(
            detail,
            HoverDetail {
                anchor_index: 0,
                title: "Tea".to_string(),
                subtitle: "timer • 4m 10s".to_string(),
            }
        );
    }

    #[test]
    fn hover_detail_uses_aggregate_text_for_overflow_target() {
        let items = vec![
            sample_item("Tea", "4m 10s"),
            sample_item("Laundry", "8m 0s"),
            sample_item("Meeting", "22m 0s"),
        ];
        let detail = hover_detail(
            &items,
            2,
            Some(OverlayHitTarget::OverflowIndicator {
                anchor_id: items[1].id,
                hidden_count: 1,
            }),
        )
        .expect("expected overflow hover detail");

        assert_eq!(
            detail,
            HoverDetail {
                anchor_index: 1,
                title: "1 more active reminder".to_string(),
                subtitle: "Open Alarms & Timers to view the full list".to_string(),
            }
        );
    }
}
