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
const REDUCED_LANE_WIDTH_FACTOR: f32 = 0.96;
const REDUCED_LANE_HEIGHT_FACTOR: f32 = 0.14;
const REDUCED_LANE_VERTICAL_OFFSET_FACTOR: f32 = 0.58;
const REDUCED_LANE_TEXT_SIZE_FACTOR: f32 = 0.10;
const MINIMAL_INDICATOR_WIDTH_FACTOR: f32 = 0.40;
const MINIMAL_INDICATOR_HEIGHT_FACTOR: f32 = 0.14;
const MINIMAL_INDICATOR_VERTICAL_OFFSET_FACTOR: f32 = 0.54;
const MINIMAL_INDICATOR_TEXT_SIZE_FACTOR: f32 = 0.09;
const HOVER_DETAIL_WIDTH_FACTOR: f32 = 1.08;
const HOVER_DETAIL_HEIGHT_FACTOR: f32 = 0.36;
const HOVER_DETAIL_VERTICAL_GAP_FACTOR: f32 = 0.08;
const HOVER_DETAIL_TITLE_SIZE_FACTOR: f32 = 0.10;
const HOVER_DETAIL_SUBTITLE_SIZE_FACTOR: f32 = 0.075;
const HOVER_DETAIL_LINE_SPACING_FACTOR: f32 = 1.28;
const HOVER_DETAIL_HORIZONTAL_PADDING_FACTOR: f32 = 0.08;
const HOVER_DETAIL_VERTICAL_PADDING_FACTOR: f32 = 0.07;
const HOVER_DETAIL_TITLE_GAP_FACTOR: f32 = 0.06;
const HOVER_DETAIL_DIVIDER_GAP_FACTOR: f32 = 0.035;

#[derive(Debug, Clone, Copy, PartialEq)]
struct SummaryLaneLayout {
    bounds: Rectangle,
    text_positions: [Point; MAX_VISIBLE_SUMMARY_LINES],
    text_size: f32,
    max_chars: usize,
    line_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MinimalIndicatorLayout {
    bounds: Rectangle,
    text_position: Point,
    text_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverlayHitTarget {
    SummaryItem(Uuid),
    OverflowIndicator {
        anchor_id: Uuid,
        hidden_count: usize,
    },
    MinimalIndicator,
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
    detail_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HoverWindowContent {
    pub title: String,
    pub detail_lines: Vec<String>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum OverlayLayout {
    Summary(SummaryLaneLayout),
    MinimalIndicator(MinimalIndicatorLayout),
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

        let Some(layout) = overlay_layout(centre, radius, self.active_items.len()) else {
            return;
        };

        let style = HoverDetailStyle {
            face_colour: self.theme.face_colour.into(),
            border_colour: self.theme.border_colour.into(),
            text_colour: self.theme.numeral_colour.into(),
            shadow_colour: self.theme.shadow_colour.into(),
        };

        if hovered_target.is_some() {
            draw_hover_backdrop(frame, centre, radius, style);
        }

        match layout {
            OverlayLayout::Summary(layout) => {
                draw_summary_lane(
                    frame,
                    &self.active_items,
                    &layout,
                    style.text_colour,
                    style.shadow_colour,
                    radius,
                );

                if let Some(detail) =
                    hover_detail(&self.active_items, layout.line_count, hovered_target)
                {
                    draw_hover_detail(frame, &layout, radius, detail, style);
                }
            }
            OverlayLayout::MinimalIndicator(layout) => {
                draw_minimal_indicator(
                    frame,
                    &layout,
                    &minimal_indicator_text(self.active_items.len()),
                    style,
                    radius,
                );

                if hovered_target == Some(OverlayHitTarget::MinimalIndicator) {
                    draw_minimal_indicator_hover_detail(
                        frame,
                        &layout,
                        radius,
                        collapsed_hover_detail(&self.active_items),
                        style,
                    );
                }
            }
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

    pub(crate) fn hover_window_content(
        &self,
        radius: f32,
        hovered_target: Option<OverlayHitTarget>,
    ) -> Option<HoverWindowContent> {
        hover_window_content(&self.active_items, radius, hovered_target)
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

fn overlay_layout(centre: Point, radius: f32, item_count: usize) -> Option<OverlayLayout> {
    match overlay_layout_mode(radius)? {
        OverlayLayoutMode::FullLane => summary_lane_layout_for_mode(
            centre,
            radius,
            item_count.min(MAX_VISIBLE_SUMMARY_LINES),
            OverlayLayoutMode::FullLane,
        )
        .map(OverlayLayout::Summary),
        OverlayLayoutMode::ReducedLane => {
            summary_lane_layout_for_mode(centre, radius, 1, OverlayLayoutMode::ReducedLane)
                .map(OverlayLayout::Summary)
        }
        OverlayLayoutMode::MinimalIndicator => {
            minimal_indicator_layout(centre, radius).map(OverlayLayout::MinimalIndicator)
        }
    }
}

#[cfg(test)]
fn summary_lane_layout(centre: Point, radius: f32, line_count: usize) -> Option<SummaryLaneLayout> {
    if overlay_layout_mode(radius) != Some(OverlayLayoutMode::FullLane) {
        return None;
    }

    summary_lane_layout_for_mode(centre, radius, line_count, OverlayLayoutMode::FullLane)
}

fn summary_lane_layout_for_mode(
    centre: Point,
    radius: f32,
    line_count: usize,
    mode: OverlayLayoutMode,
) -> Option<SummaryLaneLayout> {
    let (
        width_factor,
        single_height_factor,
        multi_height_factor,
        vertical_offset_factor,
        text_size_factor,
    ) = match mode {
        OverlayLayoutMode::FullLane => (
            SUMMARY_LANE_WIDTH_FACTOR,
            SUMMARY_LANE_SINGLE_HEIGHT_FACTOR,
            SUMMARY_LANE_MULTI_HEIGHT_FACTOR,
            SUMMARY_LANE_VERTICAL_OFFSET_FACTOR,
            SUMMARY_LANE_TEXT_SIZE_FACTOR,
        ),
        OverlayLayoutMode::ReducedLane => (
            REDUCED_LANE_WIDTH_FACTOR,
            REDUCED_LANE_HEIGHT_FACTOR,
            REDUCED_LANE_HEIGHT_FACTOR,
            REDUCED_LANE_VERTICAL_OFFSET_FACTOR,
            REDUCED_LANE_TEXT_SIZE_FACTOR,
        ),
        OverlayLayoutMode::MinimalIndicator => return None,
    };

    let line_count = line_count.clamp(1, MAX_VISIBLE_SUMMARY_LINES);
    let width = radius * width_factor;
    let height = radius
        * if line_count == 1 {
            single_height_factor
        } else {
            multi_height_factor
        };
    let x = centre.x - width / 2.0;
    let y = centre.y + radius * vertical_offset_factor - height / 2.0;
    let text_size = (radius * text_size_factor).clamp(11.0, 22.0);
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

fn minimal_indicator_layout(centre: Point, radius: f32) -> Option<MinimalIndicatorLayout> {
    if overlay_layout_mode(radius) != Some(OverlayLayoutMode::MinimalIndicator) {
        return None;
    }

    let width = radius * MINIMAL_INDICATOR_WIDTH_FACTOR;
    let height = radius * MINIMAL_INDICATOR_HEIGHT_FACTOR;
    let x = centre.x - width / 2.0;
    let y = centre.y + radius * MINIMAL_INDICATOR_VERTICAL_OFFSET_FACTOR - height / 2.0;
    let text_size = (radius * MINIMAL_INDICATOR_TEXT_SIZE_FACTOR).clamp(10.0, 13.0);

    Some(MinimalIndicatorLayout {
        bounds: Rectangle {
            x,
            y,
            width,
            height,
        },
        text_position: Point::new(centre.x, y + height / 2.0),
        text_size,
    })
}

fn draw_summary_lane(
    frame: &mut Frame,
    items: &[FaceActiveItem],
    layout: &SummaryLaneLayout,
    text_colour: Color,
    shadow_colour: Color,
    radius: f32,
) {
    let summaries = summary_lane_texts(items, layout.max_chars, layout.line_count);
    let shadow_offset = radius * SUMMARY_LANE_SHADOW_OFFSET_FACTOR;

    for (index, summary) in summaries.iter().enumerate() {
        let position = layout.text_positions[index];

        frame.fill_text(canvas::Text {
            content: summary.clone(),
            position: Point::new(position.x + shadow_offset, position.y + shadow_offset),
            size: layout.text_size.into(),
            color: shadow_colour,
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..canvas::Text::default()
        });

        frame.fill_text(canvas::Text {
            content: summary.clone(),
            position,
            size: layout.text_size.into(),
            color: text_colour,
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..canvas::Text::default()
        });
    }
}

fn minimal_indicator_text(item_count: usize) -> String {
    match item_count {
        0 => String::new(),
        1..=9 => item_count.to_string(),
        _ => "9+".to_string(),
    }
}

fn draw_minimal_indicator(
    frame: &mut Frame,
    layout: &MinimalIndicatorLayout,
    text: &str,
    style: HoverDetailStyle,
    radius: f32,
) {
    let panel = Path::rectangle(
        Point::new(layout.bounds.x, layout.bounds.y),
        Size::new(layout.bounds.width, layout.bounds.height),
    );
    let mut background = style.face_colour;
    background.a = background.a.max(0.84);

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
    frame.fill_text(canvas::Text {
        content: text.to_string(),
        position: Point::new(
            layout.text_position.x + shadow_offset,
            layout.text_position.y + shadow_offset,
        ),
        size: layout.text_size.into(),
        color: style.shadow_colour,
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..canvas::Text::default()
    });

    frame.fill_text(canvas::Text {
        content: text.to_string(),
        position: layout.text_position,
        size: layout.text_size.into(),
        color: style.text_colour,
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..canvas::Text::default()
    });
}

fn draw_hover_backdrop(frame: &mut Frame, centre: Point, radius: f32, style: HoverDetailStyle) {
    let face = Path::circle(centre, radius);
    frame.fill(&face, hover_backdrop_colour(style));
    frame.stroke(
        &face,
        Stroke {
            style: stroke::Style::Solid(style.border_colour),
            width: 1.5,
            ..Stroke::default()
        },
    );
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

    let Some(layout) = overlay_layout(centre, radius, items.len()) else {
        return Vec::new();
    };

    let OverlayLayout::Summary(layout) = layout else {
        return vec![OverlayHitRegion {
            target: OverlayHitTarget::MinimalIndicator,
            bounds: match layout {
                OverlayLayout::MinimalIndicator(layout) => layout.bounds,
                OverlayLayout::Summary(_) => unreachable!(),
            },
        }];
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
        OverlayHitTarget::SummaryItem(_) | OverlayHitTarget::MinimalIndicator => 1,
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
    if trailing_suffix.is_empty() {
        return truncate_text(&item.label, max_chars);
    }

    let suffix_chars = trailing_suffix.chars().count();
    let minimum_label_chars = 4;

    if max_chars <= suffix_chars + minimum_label_chars {
        return truncate_text(trailing_suffix.trim_start(), max_chars);
    }

    let label_budget = max_chars.saturating_sub(suffix_chars);
    let label = truncate_text(&item.label, label_budget);

    format!("{label}{trailing_suffix}")
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
                detail_lines: item_detail_lines(item),
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
                    detail_lines: vec!["Hover individual items for quick detail".to_string()],
                })
        }
        OverlayHitTarget::MinimalIndicator => None,
    }
}

fn item_detail_lines(item: &FaceActiveItem) -> Vec<String> {
    let mut lines = Vec::with_capacity(2);
    lines.push(format!(
        "{} • {}",
        match item.kind {
            FaceActiveItemKind::Alarm => "alarm",
            FaceActiveItemKind::Timer => "timer",
        },
        item.remaining_text
    ));

    if let Some(description) = &item.description {
        lines.push(description.clone());
    }

    lines
}

fn collapsed_hover_detail(items: &[FaceActiveItem]) -> HoverDetail {
    HoverDetail {
        anchor_index: 0,
        title: format!(
            "{} active reminder{}",
            items.len(),
            if items.len() == 1 { "" } else { "s" }
        ),
        detail_lines: items
            .iter()
            .map(|item| format!("{} • {}", item.label, item.remaining_text))
            .collect(),
    }
}

fn hover_window_content(
    items: &[FaceActiveItem],
    radius: f32,
    hovered_target: Option<OverlayHitTarget>,
) -> Option<HoverWindowContent> {
    let detail = match overlay_layout_mode(radius)? {
        OverlayLayoutMode::FullLane => {
            hover_detail(items, MAX_VISIBLE_SUMMARY_LINES, hovered_target)
        }
        OverlayLayoutMode::ReducedLane => hover_detail(items, 1, hovered_target),
        OverlayLayoutMode::MinimalIndicator => match hovered_target {
            Some(OverlayHitTarget::MinimalIndicator) => Some(collapsed_hover_detail(items)),
            _ => None,
        },
    }?;

    Some(HoverWindowContent {
        title: detail.title,
        detail_lines: detail.detail_lines,
    })
}

fn draw_hover_detail(
    frame: &mut Frame,
    layout: &SummaryLaneLayout,
    radius: f32,
    detail: HoverDetail,
    style: HoverDetailStyle,
) {
    let (width, height) = hover_detail_size(radius, &detail);
    let gap = radius * HOVER_DETAIL_VERTICAL_GAP_FACTOR;
    let Point { x, y } = hover_detail_origin(
        layout.bounds,
        layout.text_positions[detail.anchor_index].x,
        width,
        height,
        gap,
    );

    draw_hover_detail_panel(
        frame,
        Point::new(x, y),
        width,
        height,
        radius,
        detail,
        style,
    );
}

fn draw_minimal_indicator_hover_detail(
    frame: &mut Frame,
    layout: &MinimalIndicatorLayout,
    radius: f32,
    detail: HoverDetail,
    style: HoverDetailStyle,
) {
    let (width, height) = hover_detail_size(radius, &detail);
    let gap = radius * HOVER_DETAIL_VERTICAL_GAP_FACTOR;
    let Point { x, y } =
        hover_detail_origin(layout.bounds, layout.text_position.x, width, height, gap);

    draw_hover_detail_panel(
        frame,
        Point::new(x, y),
        width,
        height,
        radius,
        detail,
        style,
    );
}

fn draw_hover_detail_panel(
    frame: &mut Frame,
    origin: Point,
    width: f32,
    height: f32,
    radius: f32,
    detail: HoverDetail,
    style: HoverDetailStyle,
) {
    let Point { x, y } = origin;
    let title_size = (radius * HOVER_DETAIL_TITLE_SIZE_FACTOR).clamp(12.0, 20.0);
    let subtitle_size = subtitle_size_for_radius(radius);
    let horizontal_padding = radius * HOVER_DETAIL_HORIZONTAL_PADDING_FACTOR;
    let vertical_padding = radius * HOVER_DETAIL_VERTICAL_PADDING_FACTOR;
    let title_gap = radius * HOVER_DETAIL_TITLE_GAP_FACTOR;
    let divider_gap = radius * HOVER_DETAIL_DIVIDER_GAP_FACTOR;
    let body_step = subtitle_size * HOVER_DETAIL_LINE_SPACING_FACTOR;

    let panel = Path::rectangle(Point::new(x, y), Size::new(width, height));
    let backplate = hover_panel_backplate(style);
    let background = hover_panel_background(style);

    frame.fill(&panel, backplate);
    frame.fill(&panel, background);
    frame.stroke(
        &panel,
        Stroke {
            style: stroke::Style::Solid(style.border_colour),
            width: 1.25,
            ..Stroke::default()
        },
    );

    let shadow_offset = radius * SUMMARY_LANE_SHADOW_OFFSET_FACTOR;
    let title_y = y + vertical_padding + title_size / 2.0;
    let divider_y = y + vertical_padding + title_size + title_gap;
    let mut current_y = divider_y + divider_gap + subtitle_size / 2.0;

    draw_hover_text_line(
        frame,
        &detail.title,
        Point::new(x + width / 2.0, title_y),
        title_size,
        style,
        shadow_offset,
        alignment::Horizontal::Center,
    );

    let divider = Path::line(
        Point::new(x + horizontal_padding, divider_y),
        Point::new(x + width - horizontal_padding, divider_y),
    );
    frame.stroke(
        &divider,
        Stroke {
            style: stroke::Style::Solid(style.border_colour),
            width: 1.0,
            ..Stroke::default()
        },
    );

    for line in &detail.detail_lines {
        draw_hover_text_line(
            frame,
            line,
            Point::new(x + horizontal_padding, current_y),
            subtitle_size,
            style,
            shadow_offset,
            alignment::Horizontal::Left,
        );
        current_y += body_step;
    }
}

fn hover_panel_backplate(style: HoverDetailStyle) -> Color {
    if relative_luminance(opaque(style.face_colour)) >= 0.6 {
        Color::from_rgb(0.94, 0.94, 0.92)
    } else {
        Color::from_rgb(0.14, 0.15, 0.17)
    }
}

fn hover_backdrop_colour(style: HoverDetailStyle) -> Color {
    let backplate = hover_panel_backplate(style);

    if relative_luminance(backplate) >= 0.6 {
        mix_colours(backplate, Color::from_rgb(1.0, 1.0, 1.0), 0.08)
    } else {
        mix_colours(backplate, Color::from_rgb(0.05, 0.06, 0.08), 0.10)
    }
}

fn hover_panel_background(style: HoverDetailStyle) -> Color {
    let base = hover_panel_backplate(style);
    let accent = if relative_luminance(base) >= 0.6 {
        mix_colours(base, opaque(style.border_colour), 0.10)
    } else {
        mix_colours(base, opaque(style.text_colour), 0.08)
    };

    Color { a: 1.0, ..accent }
}

fn mix_colours(base: Color, accent: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);

    Color {
        r: base.r + (accent.r - base.r) * amount,
        g: base.g + (accent.g - base.g) * amount,
        b: base.b + (accent.b - base.b) * amount,
        a: base.a + (accent.a - base.a) * amount,
    }
}

fn opaque(colour: Color) -> Color {
    Color { a: 1.0, ..colour }
}

fn relative_luminance(colour: Color) -> f32 {
    0.2126 * colour.r + 0.7152 * colour.g + 0.0722 * colour.b
}

fn hover_detail_size(radius: f32, detail: &HoverDetail) -> (f32, f32) {
    let title_size = (radius * HOVER_DETAIL_TITLE_SIZE_FACTOR).clamp(12.0, 20.0);
    let subtitle_size = subtitle_size_for_radius(radius);
    let horizontal_padding = radius * HOVER_DETAIL_HORIZONTAL_PADDING_FACTOR;
    let vertical_padding = radius * HOVER_DETAIL_VERTICAL_PADDING_FACTOR;
    let title_gap = radius * HOVER_DETAIL_TITLE_GAP_FACTOR;
    let divider_gap = radius * HOVER_DETAIL_DIVIDER_GAP_FACTOR;
    let body_step = subtitle_size * HOVER_DETAIL_LINE_SPACING_FACTOR;
    let longest_line_width = std::iter::once((&detail.title, title_size))
        .chain(detail.detail_lines.iter().map(|line| (line, subtitle_size)))
        .map(|(line, size)| estimated_text_width(line, size))
        .fold(0.0, f32::max);
    let min_width = radius * HOVER_DETAIL_WIDTH_FACTOR;
    let width = (longest_line_width + horizontal_padding * 2.0).max(min_width);
    let body_height = if detail.detail_lines.is_empty() {
        0.0
    } else {
        subtitle_size + body_step * detail.detail_lines.len().saturating_sub(1) as f32
    };
    let min_height = radius * HOVER_DETAIL_HEIGHT_FACTOR;
    let height =
        (vertical_padding * 2.0 + title_size + title_gap + divider_gap * 2.0 + body_height)
            .max(min_height);

    (width, height)
}

fn subtitle_size_for_radius(radius: f32) -> f32 {
    (radius * HOVER_DETAIL_SUBTITLE_SIZE_FACTOR).clamp(10.0, 15.0)
}

fn draw_hover_text_line(
    frame: &mut Frame,
    content: &str,
    position: Point,
    size: f32,
    style: HoverDetailStyle,
    shadow_offset: f32,
    align_x: alignment::Horizontal,
) {
    for (colour, draw_position) in [
        (
            style.shadow_colour,
            Point::new(position.x + shadow_offset, position.y + shadow_offset),
        ),
        (style.text_colour, position),
    ] {
        frame.fill_text(canvas::Text {
            content: content.to_string(),
            position: draw_position,
            size: size.into(),
            color: colour,
            align_x: align_x.into(),
            align_y: alignment::Vertical::Center,
            ..canvas::Text::default()
        });
    }
}

fn hover_detail_origin(
    anchor_bounds: Rectangle,
    anchor_x: f32,
    width: f32,
    height: f32,
    gap: f32,
) -> Point {
    let min_x = anchor_bounds.x;
    let max_x = anchor_bounds.x + anchor_bounds.width - width;
    let x = if max_x <= min_x {
        anchor_bounds.x + (anchor_bounds.width - width) / 2.0
    } else {
        (anchor_x - width / 2.0).clamp(min_x, max_x)
    };
    let y = anchor_bounds.y - height - gap;

    Point::new(x, y)
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
        collapsed_hover_detail, hover_backdrop_colour, hover_detail, hover_detail_origin,
        hover_detail_size, hover_panel_background, hover_panel_backplate, minimal_indicator_text,
        overlay_hit_regions, overlay_layout, overlay_layout_mode, rectangle_contains_point,
        summary_lane_layout, summary_lane_text, summary_lane_texts, HoverDetail, HoverDetailStyle,
        OverlayHitTarget, OverlayLayout, OverlayLayoutMode, HOVER_DETAIL_VERTICAL_GAP_FACTOR,
        HOVER_DETAIL_WIDTH_FACTOR,
    };
    use crate::alarm::{FaceActiveItem, FaceActiveItemKind};
    use chrono::Local;
    use iced::{Color, Point};
    use uuid::Uuid;

    fn sample_item(label: &str, remaining_text: &str) -> FaceActiveItem {
        FaceActiveItem {
            id: Uuid::new_v4(),
            label: label.to_string(),
            description: None,
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
    fn overlay_layout_uses_single_line_summary_in_reduced_mode() {
        let layout = overlay_layout(Point::new(100.0, 100.0), 84.0, 3)
            .expect("reduced sizes should still show a summary layout");

        let OverlayLayout::Summary(layout) = layout else {
            panic!("expected reduced mode to use a summary layout");
        };

        assert_eq!(layout.line_count, 1);
    }

    #[test]
    fn summary_lane_text_preserves_remaining_time() {
        let item = sample_item("Tea", "4m 10s");
        assert_eq!(summary_lane_text(&item, 24, ""), "Tea");
    }

    #[test]
    fn summary_lane_text_truncates_long_labels() {
        let item = sample_item("Very long reminder label", "12m 0s");
        let summary = summary_lane_text(&item, 18, "");

        assert!(!summary.contains("12m 0s"));
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
        assert_eq!(summaries[0], "Tea");
        assert_eq!(summaries[1], "Laundry +1 more");
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
    fn overlay_hit_regions_include_overflow_target_in_reduced_mode() {
        let items = vec![
            sample_item("Tea", "4m 10s"),
            sample_item("Laundry", "8m 0s"),
            sample_item("Meeting", "22m 0s"),
        ];
        let regions = overlay_hit_regions(&items, Point::new(100.0, 100.0), 84.0);

        assert_eq!(regions.len(), 2);
        assert!(matches!(
            regions[0].target,
            OverlayHitTarget::OverflowIndicator {
                hidden_count: 2,
                ..
            }
        ));
        assert!(matches!(
            regions[1].target,
            OverlayHitTarget::SummaryItem(_)
        ));
    }

    #[test]
    fn overlay_hit_regions_include_minimal_indicator_target() {
        let items = vec![sample_item("Tea", "4m 10s")];
        let regions = overlay_hit_regions(&items, Point::new(75.0, 75.0), 71.25);

        assert_eq!(regions.len(), 1);
        assert!(matches!(
            regions[0].target,
            OverlayHitTarget::MinimalIndicator
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
                detail_lines: vec!["timer • 4m 10s".to_string()],
            }
        );
    }

    #[test]
    fn hover_detail_includes_description_when_present() {
        let mut item = sample_item("Tea", "4m 10s");
        item.description = Some("Steep the green tea".to_string());

        let detail = hover_detail(
            &[item.clone()],
            1,
            Some(OverlayHitTarget::SummaryItem(item.id)),
        )
        .expect("expected item hover detail");

        assert_eq!(
            detail.detail_lines,
            vec![
                "timer • 4m 10s".to_string(),
                "Steep the green tea".to_string()
            ]
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
                detail_lines: vec!["Hover individual items for quick detail".to_string()],
            }
        );
    }

    #[test]
    fn collapsed_hover_detail_lists_all_active_items() {
        let items = vec![
            sample_item("Tea", "4m 10s"),
            sample_item("Laundry", "8m 0s"),
        ];

        let detail = collapsed_hover_detail(&items);

        assert_eq!(detail.title, "2 active reminders");
        assert_eq!(
            detail.detail_lines,
            vec!["Tea • 4m 10s".to_string(), "Laundry • 8m 0s".to_string()]
        );
    }

    #[test]
    fn hover_detail_origin_is_safe_when_panel_is_wider_than_lane() {
        let centre = Point::new(125.0, 125.0);
        let layout = summary_lane_layout(centre, 119.0, 1).expect("expected full lane layout");
        let detail = HoverDetail {
            anchor_index: 0,
            title: "Tea".to_string(),
            detail_lines: vec!["timer • 4m 10s".to_string()],
        };
        let (width, height) = hover_detail_size(119.0, &detail);
        let gap = 119.0 * HOVER_DETAIL_VERTICAL_GAP_FACTOR;

        let origin = hover_detail_origin(
            layout.bounds,
            layout.text_positions[detail.anchor_index].x,
            width,
            height,
            gap,
        );

        assert!(origin.x.is_finite());
        assert!(origin.y.is_finite());
        assert_eq!(
            origin.x,
            layout.bounds.x + (layout.bounds.width - width) / 2.0
        );
    }

    #[test]
    fn hover_detail_size_grows_for_multi_line_lists() {
        let single = HoverDetail {
            anchor_index: 0,
            title: "Tea".to_string(),
            detail_lines: vec!["timer • 4m 10s".to_string()],
        };
        let multi = HoverDetail {
            anchor_index: 0,
            title: "4 active reminders".to_string(),
            detail_lines: vec![
                "Tea • 4m 10s".to_string(),
                "Laundry • 8m 0s".to_string(),
                "Meeting • 22m 0s".to_string(),
            ],
        };

        let (single_width, single_height) = hover_detail_size(71.25, &single);
        let (multi_width, multi_height) = hover_detail_size(71.25, &multi);

        assert!(single_width >= 71.25 * HOVER_DETAIL_WIDTH_FACTOR);
        assert!(multi_height > single_height);
        assert!(multi_width >= single_width);
    }

    #[test]
    fn hover_panel_background_is_opaque_and_tinted_for_light_themes() {
        let style = HoverDetailStyle {
            face_colour: Color::from_rgba(1.0, 1.0, 1.0, 0.9),
            border_colour: Color::from_rgba(0.2, 0.2, 0.2, 1.0),
            text_colour: Color::from_rgba(0.0, 0.0, 0.0, 1.0),
            shadow_colour: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
        };
        let backplate = hover_panel_backplate(style);
        let background = hover_panel_background(style);

        assert_eq!(backplate.a, 1.0);
        assert_eq!(background.a, 1.0);
        assert_eq!(backplate, Color::from_rgb(0.94, 0.94, 0.92));
        assert!(background.r <= backplate.r);
        assert!(background.g <= backplate.g);
        assert!(background.b <= backplate.b);
    }

    #[test]
    fn hover_panel_background_is_opaque_and_tinted_for_dark_themes() {
        let style = HoverDetailStyle {
            face_colour: Color::from_rgba(0.12, 0.12, 0.15, 0.92),
            border_colour: Color::from_rgba(0.4, 0.4, 0.45, 1.0),
            text_colour: Color::from_rgba(0.85, 0.85, 0.85, 1.0),
            shadow_colour: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
        };
        let backplate = hover_panel_backplate(style);
        let background = hover_panel_background(style);

        assert_eq!(backplate.a, 1.0);
        assert_eq!(background.a, 1.0);
        assert_eq!(backplate, Color::from_rgb(0.14, 0.15, 0.17));
        assert!(background.r >= backplate.r);
        assert!(background.g >= backplate.g);
        assert!(background.b >= backplate.b);
    }

    #[test]
    fn hover_backdrop_colour_stays_opaque_for_light_themes() {
        let colour = hover_backdrop_colour(HoverDetailStyle {
            face_colour: Color::from_rgba(1.0, 1.0, 1.0, 0.9),
            border_colour: Color::from_rgba(0.2, 0.2, 0.2, 1.0),
            text_colour: Color::from_rgba(0.0, 0.0, 0.0, 1.0),
            shadow_colour: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
        });

        assert_eq!(colour.a, 1.0);
        assert!(colour.r >= 0.94);
        assert!(colour.g >= 0.94);
        assert!(colour.b >= 0.92);
    }

    #[test]
    fn hover_backdrop_colour_stays_opaque_for_dark_themes() {
        let colour = hover_backdrop_colour(HoverDetailStyle {
            face_colour: Color::from_rgba(0.12, 0.12, 0.15, 0.92),
            border_colour: Color::from_rgba(0.4, 0.4, 0.45, 1.0),
            text_colour: Color::from_rgba(0.85, 0.85, 0.85, 1.0),
            shadow_colour: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
        });

        assert_eq!(colour.a, 1.0);
        assert!(colour.r <= 0.14);
        assert!(colour.g <= 0.15);
        assert!(colour.b <= 0.17);
    }

    #[test]
    fn minimal_indicator_text_reports_active_count() {
        assert_eq!(minimal_indicator_text(1), "1");
        assert_eq!(minimal_indicator_text(4), "4");
        assert_eq!(minimal_indicator_text(12), "9+");
    }
}
