use iced::{window, Task};

use crate::tray::{self, SystemTrayHandle, TrayCommand};

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
    settings.platform_specific.application_id = "rust-clock".to_string();
}

pub fn configure_control_window_settings(settings: &mut window::Settings) {
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

    for atom in states {
        let event = ClientMessageEvent::new(32, window_id, net_wm_state, [1, atom, 0, 0, 0]);

        conn.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
    }

    conn.flush()?;
    Ok(())
}

fn intern_atom(
    conn: &x11rb::rust_connection::RustConnection,
    name: &[u8],
) -> Result<u32, Box<dyn std::error::Error>> {
    use x11rb::protocol::xproto::ConnectionExt as _;

    Ok(conn.intern_atom(false, name)?.reply()?.atom)
}
