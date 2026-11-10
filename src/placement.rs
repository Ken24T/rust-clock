//! Monitor-relative placement and recovery after desktop layout changes.

use iced::Point;
use serde::{Deserialize, Serialize};

use crate::platform::{Monitor, WorkArea};

/// A deliberate placement, measured from a named monitor's top-right corner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorPlacement {
    pub monitor: String,
    pub right: i32,
    pub top: i32,
}

impl MonitorPlacement {
    pub fn at_position(monitor: &Monitor, position: Point, size: f32) -> Self {
        Self {
            monitor: monitor.name.clone(),
            right: (monitor.area.x + monitor.area.width - position.x - size).round() as i32,
            top: (position.y - monitor.area.y).round() as i32,
        }
    }

    pub fn position(&self, monitors: &[Monitor], size: f32) -> Option<Point> {
        let WorkArea { x, y, width, .. } = monitors
            .iter()
            .find(|monitor| monitor.name == self.monitor)?
            .area;
        Some(Point::new(
            x + width - size - self.right as f32,
            y + self.top as f32,
        ))
    }
}

/// Automatic desktop moves never become a new user preference. Layout polling
/// continues for the lifetime of the clock, including after startup retries end.
#[derive(Debug, Default)]
pub struct PlacementRecovery {
    last_layout: Option<Vec<Monitor>>,
    drag_layout: Option<Vec<Monitor>>,
    dragging: bool,
    finishing_drag: bool,
    restore_pending: bool,
}

impl PlacementRecovery {
    pub fn new(layout: Option<Vec<Monitor>>) -> Self {
        Self {
            last_layout: layout,
            ..Self::default()
        }
    }

    pub fn begin_drag(&mut self, layout: Option<Vec<Monitor>>) {
        self.drag_layout = layout;
        self.dragging = true;
        self.finishing_drag = false;
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn start_finishing_drag(&mut self, button_down: Option<bool>) -> bool {
        if self.dragging && !self.finishing_drag && button_down == Some(false) {
            self.finishing_drag = true;
            true
        } else {
            false
        }
    }

    /// Discard a drag interrupted by a monitor change or a failed layout query.
    pub fn finish_drag(&mut self, layout: &Option<Vec<Monitor>>) -> bool {
        let deliberate = self.dragging && self.drag_layout.is_some() && self.drag_layout == *layout;
        self.dragging = false;
        self.finishing_drag = false;
        self.drag_layout = None;
        self.restore_pending = !deliberate;
        deliberate
    }

    pub fn unsolicited_move(&mut self) {
        if !self.dragging {
            self.restore_pending = true;
        }
    }

    pub fn should_restore(&mut self, layout: Option<Vec<Monitor>>) -> bool {
        let Some(layout) = layout else {
            return false;
        };
        let changed = self.last_layout.as_ref() != Some(&layout);
        self.last_layout = Some(layout);
        self.restore_pending |= changed;
        if self.dragging {
            if changed {
                self.drag_layout = None;
            }
            return false;
        }
        std::mem::take(&mut self.restore_pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(name: &str, x: f32, width: f32, height: f32) -> Monitor {
        Monitor {
            name: name.into(),
            area: WorkArea {
                x,
                y: 0.0,
                width,
                height,
            },
        }
    }

    fn layout() -> Vec<Monitor> {
        vec![
            monitor("HDMI-0", 1920.0, 3840.0, 2160.0),
            monitor("DP-4", 5760.0, 1080.0, 1920.0),
        ]
    }

    #[test]
    fn restores_the_same_corner_after_resize_and_monitor_rearrangement() {
        let mut monitors = layout();
        let saved = MonitorPlacement {
            monitor: "DP-4".into(),
            right: 12,
            top: 12,
        };
        assert_eq!(
            saved.position(&monitors, 165.0),
            Some(Point::new(6663.0, 12.0))
        );
        monitors[1].area.x = 3840.0;
        assert_eq!(
            saved.position(&monitors, 250.0),
            Some(Point::new(4658.0, 12.0))
        );
        assert_eq!(saved.position(&monitors[..1], 165.0), None);
        assert_eq!(
            saved.position(&layout(), 165.0),
            Some(Point::new(6663.0, 12.0))
        );
    }

    #[test]
    fn reconnects_are_detected_after_any_number_of_unchanged_polls() {
        let mut recovery = PlacementRecovery::new(Some(layout()));
        for _ in 0..100 {
            assert!(!recovery.should_restore(Some(layout())));
        }
        assert!(recovery.should_restore(Some(layout()[..1].to_vec())));
        assert!(!recovery.should_restore(Some(layout()[..1].to_vec())));
        assert!(recovery.should_restore(Some(layout())));
        recovery.unsolicited_move();
        assert!(recovery.should_restore(Some(layout())));
        assert!(!recovery.should_restore(Some(layout())));
    }

    #[test]
    fn only_an_explicit_drag_on_an_unchanged_layout_can_be_saved() {
        let mut recovery = PlacementRecovery::new(Some(layout()));
        assert!(!recovery.finish_drag(&Some(layout())));
        recovery.begin_drag(Some(layout()));
        recovery.unsolicited_move();
        assert!(!recovery.should_restore(Some(layout())));
        assert!(!recovery.start_finishing_drag(Some(true)));
        assert!(!recovery.start_finishing_drag(None));
        assert!(recovery.start_finishing_drag(Some(false)));
        assert!(!recovery.start_finishing_drag(Some(false)));
        assert!(recovery.finish_drag(&Some(layout())));
        recovery.begin_drag(Some(layout()));
        assert!(!recovery.finish_drag(&Some(layout()[..1].to_vec())));
        assert!(recovery.should_restore(Some(layout())));
    }

    #[test]
    fn dragging_saves_monitor_relative_offsets() {
        let saved = MonitorPlacement::at_position(&layout()[1], Point::new(6600.0, 42.0), 165.0);
        assert_eq!(saved.monitor, "DP-4");
        assert_eq!((saved.right, saved.top), (75, 42));
        assert_eq!(
            saved.position(&layout(), 165.0),
            Some(Point::new(6600.0, 42.0))
        );
    }
}
