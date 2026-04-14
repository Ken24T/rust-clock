use std::sync::OnceLock;

use iced::{window, Task};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::wl_registry,
    Connection, Dispatch, QueueHandle,
};

use crate::tray::{self, SystemTrayHandle, TrayCommand};

use super::{PlatformCapabilities, WorkArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WaylandSupport {
    running: bool,
    layer_shell: bool,
}

#[derive(Debug, Default)]
struct WaylandRegistryState;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandRegistryState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

pub fn capabilities() -> PlatformCapabilities {
    let wayland_support = wayland_support();
    let wayland_session = wayland_support.running;
    let layer_shell_supported = wayland_support.layer_shell;

    PlatformCapabilities {
        system_tray: true,
        notifications: true,
        desktop_window_hints: !wayland_session,
        sticky_workspace: !wayland_session || layer_shell_supported,
        skip_taskbar: !wayland_session || layer_shell_supported,
        tray_only_main_window: wayland_session && !layer_shell_supported,
        layer_shell_main_window: layer_shell_supported,
        detached_hover_window: !layer_shell_supported,
    }
}

fn is_wayland_session() -> bool {
    wayland_support().running
}

fn wayland_support() -> WaylandSupport {
    static SUPPORT: OnceLock<WaylandSupport> = OnceLock::new();

    *SUPPORT.get_or_init(detect_wayland_support)
}

fn detect_wayland_support() -> WaylandSupport {
    let running = is_wayland_session_from_env(
        std::env::var_os("XDG_SESSION_TYPE").as_deref(),
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
    );

    let layer_shell = running
        && query_wayland_global_names()
            .is_some_and(|globals| global_names_include_layer_shell(&globals));

    WaylandSupport {
        running,
        layer_shell,
    }
}

fn is_wayland_session_from_env(
    xdg_session_type: Option<&std::ffi::OsStr>,
    wayland_display: Option<&std::ffi::OsStr>,
) -> bool {
    xdg_session_type
        .map(|value| value.to_string_lossy().eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
        || wayland_display.is_some()
}

fn query_wayland_global_names() -> Option<Vec<String>> {
    let connection = Connection::connect_to_env().ok()?;
    let (globals, _queue) = registry_queue_init::<WaylandRegistryState>(&connection).ok()?;

    Some(
        globals
            .contents()
            .clone_list()
            .into_iter()
            .map(|global| global.interface)
            .collect(),
    )
}

fn global_names_include_layer_shell(globals: &[String]) -> bool {
    globals.iter().any(|interface| {
        matches!(
            interface.as_str(),
            "zwlr_layer_shell_v1" | "ext_layer_shell_v1"
        )
    })
}

pub fn work_area_for_point(_x: f32, _y: f32) -> Option<WorkArea> {
    if is_wayland_session() {
        return None;
    }

    use x11rb::connection::Connection;
    use x11rb::protocol::randr::ConnectionExt as _;

    let (conn, screen_num) = x11rb::rust_connection::RustConnection::connect(None).ok()?;
    let screen = conn.setup().roots.get(screen_num)?;

    if let Ok(cookie) = conn.randr_get_monitors(screen.root, true) {
        if let Ok(reply) = cookie.reply() {
            let monitors = reply
                .monitors
                .iter()
                .filter(|monitor| monitor.width > 0 && monitor.height > 0)
                .map(|monitor| WorkArea {
                    x: monitor.x as f32,
                    y: monitor.y as f32,
                    width: monitor.width as f32,
                    height: monitor.height as f32,
                })
                .collect::<Vec<_>>();

            if let Some(work_area) = select_work_area(&monitors, _x, _y) {
                return Some(work_area);
            }
        }
    }

    Some(WorkArea {
        x: 0.0,
        y: 0.0,
        width: screen.width_in_pixels as f32,
        height: screen.height_in_pixels as f32,
    })
}

fn select_work_area(work_areas: &[WorkArea], x: f32, y: f32) -> Option<WorkArea> {
    work_areas
        .iter()
        .copied()
        .find(|area| point_in_work_area(*area, x, y))
        .or_else(|| {
            work_areas.iter().copied().min_by(|left, right| {
                distance_sq_to_work_area(*left, x, y)
                    .partial_cmp(&distance_sq_to_work_area(*right, x, y))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
}

fn point_in_work_area(area: WorkArea, x: f32, y: f32) -> bool {
    x >= area.x && x <= area.x + area.width && y >= area.y && y <= area.y + area.height
}

fn distance_sq_to_work_area(area: WorkArea, x: f32, y: f32) -> f32 {
    let dx = if x < area.x {
        area.x - x
    } else if x > area.x + area.width {
        x - (area.x + area.width)
    } else {
        0.0
    };
    let dy = if y < area.y {
        area.y - y
    } else if y > area.y + area.height {
        y - (area.y + area.height)
    } else {
        0.0
    };

    dx * dx + dy * dy
}

pub fn send_notification(summary: &str, body: &str) {
    // Use notify-send directly — notify-rust's zbus backend can silently
    // fail to display on some desktops (e.g. Cinnamon).
    match std::process::Command::new("notify-send")
        .arg("--app-name=Rust Clock")
        .arg("-t")
        .arg("10000")
        .arg(summary)
        .arg(body)
        .spawn()
    {
        Ok(mut child) => {
            std::thread::spawn(move || match child.wait() {
                Ok(status) if !status.success() => {
                    eprintln!("notify-send exited with status: {status}");
                }
                Ok(_) => {}
                Err(error) => eprintln!("Failed to wait for notify-send: {error}"),
            });
        }
        Err(error) => eprintln!("Failed to send notification: {error}"),
    }
}

pub fn start_system_tray() -> Option<(SystemTrayHandle, std::sync::mpsc::Receiver<TrayCommand>)> {
    tray::start_system_tray()
}

pub fn configure_main_window_settings(settings: &mut window::Settings) {
    settings.level = window::Level::AlwaysOnBottom;
    settings.platform_specific.application_id = "rust-clock".to_string();
}

pub fn configure_control_window_settings(settings: &mut window::Settings) {
    settings.level = window::Level::AlwaysOnTop;
    settings.platform_specific.application_id = "rust-clock".to_string();
}

pub fn apply_startup_window_hints(id: window::Id) -> Task<crate::Message> {
    window::run(id, |native_window| {
        if let Err(error) = apply_main_window_hints(native_window) {
            eprintln!("Failed to apply Linux window hints: {error}");
        }
    })
    .discard()
}

pub fn apply_control_window_hints(id: window::Id) -> Task<crate::Message> {
    window::run(id, |native_window| {
        if let Err(error) = apply_utility_window_hints(native_window) {
            eprintln!("Failed to apply control window hints: {error}");
        }
    })
    .discard()
}

fn apply_main_window_hints(
    native_window: &dyn window::Window,
) -> Result<(), Box<dyn std::error::Error>> {
    apply_linux_window_hints(
        native_window,
        b"_NET_WM_WINDOW_TYPE_UTILITY",
        &[
            b"_NET_WM_STATE_SKIP_TASKBAR",
            b"_NET_WM_STATE_SKIP_PAGER",
            b"_NET_WM_STATE_BELOW",
            b"_NET_WM_STATE_STICKY",
        ],
    )
}

fn apply_utility_window_hints(
    native_window: &dyn window::Window,
) -> Result<(), Box<dyn std::error::Error>> {
    apply_linux_window_hints(
        native_window,
        b"_NET_WM_WINDOW_TYPE_UTILITY",
        &[b"_NET_WM_STATE_SKIP_TASKBAR", b"_NET_WM_STATE_SKIP_PAGER"],
    )
}

fn apply_linux_window_hints(
    native_window: &dyn window::Window,
    window_type_name: &[u8],
    state_names: &[&[u8]],
) -> Result<(), Box<dyn std::error::Error>> {
    use iced::window::raw_window_handle::RawWindowHandle;

    let raw_window = native_window.window_handle()?.as_raw();
    let window_id = match raw_window {
        RawWindowHandle::Xlib(handle) => u32::try_from(handle.window)
            .map_err(|_| format!("Xlib window id out of range: {}", handle.window))?,
        RawWindowHandle::Xcb(handle) => handle.window.get(),
        RawWindowHandle::Wayland(_) => return Ok(()),
        _ => return Ok(()),
    };

    apply_x11_window_hints(window_id, window_type_name, state_names)
}

fn apply_x11_window_hints(
    window_id: u32,
    window_type_name: &[u8],
    state_names: &[&[u8]],
) -> Result<(), Box<dyn std::error::Error>> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{
        AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask, PropMode,
    };
    use x11rb::rust_connection::RustConnection;
    use x11rb::wrapper::ConnectionExt as _;

    let (conn, screen_num) = RustConnection::connect(None)?;
    let root = conn.setup().roots[screen_num].root;

    let net_wm_state = intern_atom(&conn, b"_NET_WM_STATE")?;
    let net_wm_window_type = intern_atom(&conn, b"_NET_WM_WINDOW_TYPE")?;
    let window_type = intern_atom(&conn, window_type_name)?;
    let states: Result<Vec<u32>, Box<dyn std::error::Error>> = state_names
        .iter()
        .map(|state| intern_atom(&conn, state))
        .collect();
    let states = states?;

    conn.change_property32(
        PropMode::REPLACE,
        window_id,
        net_wm_window_type,
        AtomEnum::ATOM,
        &[window_type],
    )?;

    conn.change_property32(
        PropMode::REPLACE,
        window_id,
        net_wm_state,
        AtomEnum::ATOM,
        &states,
    )?;

    for &atom in &states {
        let event = ClientMessageEvent::new(32, window_id, net_wm_state, [1, atom, 0, 0, 0]);

        conn.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
    }

    conn.flush()?;

    if x11_window_hints_match(
        &conn,
        window_id,
        net_wm_window_type,
        net_wm_state,
        window_type,
        &states,
    )? {
        return Ok(());
    }

    apply_xprop_window_hints(window_id, window_type_name, state_names)?;

    if x11_window_hints_match(
        &conn,
        window_id,
        net_wm_window_type,
        net_wm_state,
        window_type,
        &states,
    )? {
        Ok(())
    } else {
        Err("Linux window hints did not stick after fallback".into())
    }
}

fn x11_window_hints_match(
    conn: &x11rb::rust_connection::RustConnection,
    window_id: u32,
    net_wm_window_type: u32,
    net_wm_state: u32,
    expected_window_type: u32,
    expected_states: &[u32],
) -> Result<bool, Box<dyn std::error::Error>> {
    let window_types = read_atom_list_property(conn, window_id, net_wm_window_type)?;
    let states = read_atom_list_property(conn, window_id, net_wm_state)?;

    Ok(window_types.contains(&expected_window_type)
        && expected_states.iter().all(|state| states.contains(state)))
}

fn read_atom_list_property(
    conn: &x11rb::rust_connection::RustConnection,
    window_id: u32,
    property: u32,
) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _};

    let reply = conn
        .get_property(false, window_id, property, AtomEnum::ATOM, 0, u32::MAX)?
        .reply()?;

    Ok(reply
        .value32()
        .map(|values| values.collect())
        .unwrap_or_default())
}

fn apply_xprop_window_hints(
    window_id: u32,
    window_type_name: &[u8],
    state_names: &[&[u8]],
) -> Result<(), Box<dyn std::error::Error>> {
    let window_id = format!("0x{window_id:x}");
    let window_type_name = std::str::from_utf8(window_type_name)?;
    let states = state_names
        .iter()
        .map(|state| std::str::from_utf8(state))
        .collect::<Result<Vec<_>, _>>()?
        .join(",");

    let window_type_status = std::process::Command::new("xprop")
        .args([
            "-id",
            &window_id,
            "-f",
            "_NET_WM_WINDOW_TYPE",
            "32a",
            "-set",
            "_NET_WM_WINDOW_TYPE",
            window_type_name,
        ])
        .status()?;

    if !window_type_status.success() {
        return Err(
            format!("xprop failed to set _NET_WM_WINDOW_TYPE: {window_type_status}").into(),
        );
    }

    let state_status = std::process::Command::new("xprop")
        .args([
            "-id",
            &window_id,
            "-f",
            "_NET_WM_STATE",
            "32a",
            "-set",
            "_NET_WM_STATE",
            &states,
        ])
        .status()?;

    if !state_status.success() {
        return Err(format!("xprop failed to set _NET_WM_STATE: {state_status}").into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        distance_sq_to_work_area, global_names_include_layer_shell,
        is_wayland_session_from_env, point_in_work_area, select_work_area,
    };
    use crate::platform::WorkArea;

    #[test]
    fn selects_monitor_containing_point() {
        let work_areas = [
            WorkArea {
                x: 0.0,
                y: 922.0,
                width: 3840.0,
                height: 2160.0,
            },
            WorkArea {
                x: 3840.0,
                y: 384.0,
                width: 6144.0,
                height: 3456.0,
            },
        ];

        let selected =
            select_work_area(&work_areas, 5000.0, 1000.0).expect("monitor should resolve");
        assert_eq!(selected, work_areas[1]);
    }

    #[test]
    fn selects_nearest_monitor_when_point_is_in_gap() {
        let work_areas = [
            WorkArea {
                x: 0.0,
                y: 922.0,
                width: 3840.0,
                height: 2160.0,
            },
            WorkArea {
                x: 3840.0,
                y: 384.0,
                width: 6144.0,
                height: 3456.0,
            },
        ];

        let gap_point = (200.0, 100.0);
        assert!(!point_in_work_area(work_areas[0], gap_point.0, gap_point.1));
        assert!(!point_in_work_area(work_areas[1], gap_point.0, gap_point.1));

        let selected = select_work_area(&work_areas, gap_point.0, gap_point.1)
            .expect("nearest monitor should resolve");
        assert_eq!(selected, work_areas[0]);
        assert!(
            distance_sq_to_work_area(work_areas[0], gap_point.0, gap_point.1)
                < distance_sq_to_work_area(work_areas[1], gap_point.0, gap_point.1)
        );
    }

    #[test]
    fn wayland_session_detects_wayland_session_type() {
        assert!(is_wayland_session_from_env(
            Some(std::ffi::OsStr::new("wayland")),
            None,
        ));
    }

    #[test]
    fn wayland_session_detects_wayland_display() {
        assert!(is_wayland_session_from_env(
            Some(std::ffi::OsStr::new("x11")),
            Some(std::ffi::OsStr::new("wayland-0")),
        ));
    }

    #[test]
    fn wayland_session_rejects_plain_x11_environment() {
        assert!(!is_wayland_session_from_env(
            Some(std::ffi::OsStr::new("x11")),
            None,
        ));
    }

    #[test]
    fn layer_shell_detection_accepts_wlr_protocol_name() {
        assert!(global_names_include_layer_shell(&[
            "wl_compositor".to_string(),
            "zwlr_layer_shell_v1".to_string(),
        ]));
    }

    #[test]
    fn layer_shell_detection_accepts_ext_protocol_name() {
        assert!(global_names_include_layer_shell(&[
            "wl_compositor".to_string(),
            "ext_layer_shell_v1".to_string(),
        ]));
    }

    #[test]
    fn layer_shell_detection_rejects_missing_protocol() {
        assert!(!global_names_include_layer_shell(&[
            "wl_compositor".to_string(),
            "wl_subcompositor".to_string(),
        ]));
    }
}

fn intern_atom(
    conn: &x11rb::rust_connection::RustConnection,
    name: &[u8],
) -> Result<u32, Box<dyn std::error::Error>> {
    use x11rb::protocol::xproto::ConnectionExt as _;

    Ok(conn.intern_atom(false, name)?.reply()?.atom)
}
