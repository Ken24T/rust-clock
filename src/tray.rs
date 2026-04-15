#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    FocusClock,
    ShowAlarmPanel,
    AddQuickTimer(u64),
    Quit,
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
const QUICK_TIMER_PRESETS: &[(u64, &str)] = &[
    (60, "1 min"),
    (300, "5 min"),
    (600, "10 min"),
    (1800, "30 min"),
    (3600, "1 hour"),
];

#[cfg(target_os = "linux")]
mod linux {
    use std::sync::mpsc::{self, Receiver, Sender};

    use ksni::blocking::{Handle, TrayMethods};

    use super::{TrayCommand, QUICK_TIMER_PRESETS};

    pub struct SystemTrayHandle {
        handle: Handle<ClockTray>,
    }

    impl SystemTrayHandle {
        pub fn shutdown(self) {
            let _ = self.handle.shutdown();
        }
    }

    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    pub fn start_system_tray() -> Option<(SystemTrayHandle, Receiver<TrayCommand>)> {
        let (command_sender, command_receiver) = mpsc::channel();

        let tray = ClockTray { command_sender };

        match tray.assume_sni_available(true).spawn() {
            Ok(handle) => Some((SystemTrayHandle { handle }, command_receiver)),
            Err(error) => {
                eprintln!("Failed to start system tray icon: {error}");
                None
            }
        }
    }

    struct ClockTray {
        command_sender: Sender<TrayCommand>,
    }

    impl ksni::Tray for ClockTray {
        fn id(&self) -> String {
            "rust-clock".to_string()
        }

        fn title(&self) -> String {
            "Rust Clock".to_string()
        }

        fn icon_name(&self) -> String {
            "preferences-system-time".to_string()
        }

        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: "Rust Clock".into(),
                description: "Analog clock desklet".into(),
                ..Default::default()
            }
        }

        fn activate(&mut self, _x: i32, _y: i32) {
            let _ = self.command_sender.send(TrayCommand::FocusClock);
        }

        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            use ksni::menu::{StandardItem, SubMenu};

            let quick_timer_items = QUICK_TIMER_PRESETS
                .iter()
                .map(|(secs, label)| {
                    StandardItem {
                        label: (*label).into(),
                        icon_name: "alarm-symbolic".into(),
                        activate: Box::new(move |tray: &mut Self| {
                            let _ = tray.command_sender.send(TrayCommand::AddQuickTimer(*secs));
                        }),
                        ..Default::default()
                    }
                    .into()
                })
                .collect();

            vec![
                StandardItem {
                    label: "Alarms & Timers".into(),
                    icon_name: "alarm-symbolic".into(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.command_sender.send(TrayCommand::ShowAlarmPanel);
                    }),
                    ..Default::default()
                }
                .into(),
                SubMenu {
                    label: "Quick Timer".into(),
                    submenu: quick_timer_items,
                    ..Default::default()
                }
                .into(),
                SubMenu {
                    label: "Actions".into(),
                    submenu: vec![StandardItem {
                        label: "Quit".into(),
                        icon_name: "application-exit".into(),
                        activate: Box::new(|tray: &mut Self| {
                            let _ = tray.command_sender.send(TrayCommand::Quit);
                        }),
                        ..Default::default()
                    }
                    .into()],
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    pub use SystemTrayHandle as HandleType;
}

#[cfg(target_os = "windows")]
mod windows {
    use std::sync::mpsc::{self, Receiver};

    use tray_icon::{
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
        Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
    };
    use windows_sys::Win32::{
        System::Threading::GetCurrentThreadId,
        UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, PeekMessageW, PostThreadMessageW, TranslateMessage, MSG,
            PM_NOREMOVE, WM_QUIT,
        },
    };

    use crate::app_icon;

    use super::{TrayCommand, QUICK_TIMER_PRESETS};

    const MENU_ID_SHOW_CLOCK: &str = "show-clock";
    const MENU_ID_SHOW_ALARMS: &str = "show-alarms";
    const MENU_ID_QUIT: &str = "quit";
    const MENU_ID_TIMER_PREFIX: &str = "timer-";

    pub struct SystemTrayHandle {
        thread_id: u32,
        join_handle: std::thread::JoinHandle<()>,
    }

    impl SystemTrayHandle {
        pub fn shutdown(self) {
            unsafe {
                if PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0) == 0 {
                    eprintln!("Windows tray thread was already gone before shutdown request");
                }
            }
            let _ = self.join_handle.join();
        }
    }

    pub fn start_system_tray() -> Option<(SystemTrayHandle, Receiver<TrayCommand>)> {
        let (command_sender, command_receiver) = mpsc::channel();
        let (startup_sender, startup_receiver) = mpsc::sync_channel(1);

        let join_handle = std::thread::spawn(move || {
            if let Err(error) = tray_thread_main(command_sender, startup_sender) {
                eprintln!("Failed to start Windows tray icon: {error}");
            }
        });

        match startup_receiver.recv() {
            Ok(Ok(thread_id)) => Some((
                SystemTrayHandle {
                    thread_id,
                    join_handle,
                },
                command_receiver,
            )),
            Ok(Err(error)) => {
                eprintln!("Failed to initialise Windows tray icon: {error}");
                let _ = join_handle.join();
                None
            }
            Err(error) => {
                eprintln!("Failed to receive Windows tray startup result: {error}");
                let _ = join_handle.join();
                None
            }
        }
    }

    fn tray_thread_main(
        command_sender: mpsc::Sender<TrayCommand>,
        startup_sender: mpsc::SyncSender<Result<u32, String>>,
    ) -> Result<(), String> {
        unsafe {
            let mut message: MSG = std::mem::zeroed();
            let _ = PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);
        }

        let thread_id = unsafe { GetCurrentThreadId() };
        let resources = WindowsTrayResources::new(command_sender)?;
        let _ = startup_sender.send(Ok(thread_id));

        let mut message: MSG = unsafe { std::mem::zeroed() };
        loop {
            let status = unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) };
            if status == -1 {
                return Err("Win32 tray message loop failed".to_string());
            }
            if status == 0 {
                break;
            }

            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }

            resources.drain_events();
        }

        Ok(())
    }

    struct WindowsTrayResources {
        _tray_icon: tray_icon::TrayIcon,
        _menu: Menu,
        _show_clock: MenuItem,
        _show_alarms: MenuItem,
        _quick_timers: Submenu,
        _quick_timer_items: Vec<MenuItem>,
        _quit: MenuItem,
        command_sender: mpsc::Sender<TrayCommand>,
    }

    impl WindowsTrayResources {
        fn new(command_sender: mpsc::Sender<TrayCommand>) -> Result<Self, String> {
            let menu = Menu::new();
            let show_clock = MenuItem::with_id(MENU_ID_SHOW_CLOCK, "Show Clock", true, None);
            let show_alarms = MenuItem::with_id(MENU_ID_SHOW_ALARMS, "Alarms & Timers", true, None);
            let quick_timer_items: Vec<MenuItem> = QUICK_TIMER_PRESETS
                .iter()
                .map(|(secs, label)| {
                    MenuItem::with_id(format!("{MENU_ID_TIMER_PREFIX}{secs}"), *label, true, None)
                })
                .collect();
            let quick_timer_item_refs: Vec<&dyn tray_icon::menu::IsMenuItem> = quick_timer_items
                .iter()
                .map(|item| item as &dyn tray_icon::menu::IsMenuItem)
                .collect();
            let quick_timers = Submenu::with_items("Quick Timer", true, &quick_timer_item_refs)
                .map_err(|error| error.to_string())?;
            let separator = PredefinedMenuItem::separator();
            let quit = MenuItem::with_id(MENU_ID_QUIT, "Quit", true, None);

            menu.append_items(&[&show_clock, &show_alarms, &quick_timers, &separator, &quit])
                .map_err(|error| error.to_string())?;

            let tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(menu.clone()))
                .with_menu_on_left_click(false)
                .with_tooltip("Rust Clock")
                .with_icon(create_windows_tray_icon()?)
                .build()
                .map_err(|error| error.to_string())?;

            Ok(Self {
                _tray_icon: tray_icon,
                _menu: menu,
                _show_clock: show_clock,
                _show_alarms: show_alarms,
                _quick_timers: quick_timers,
                _quick_timer_items: quick_timer_items,
                _quit: quit,
                command_sender,
            })
        }

        fn drain_events(&self) {
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    let _ = self.command_sender.send(TrayCommand::FocusClock);
                }
            }

            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if let Some(command) = tray_command_from_menu_id(event.id.as_ref()) {
                    let _ = self.command_sender.send(command);
                }
            }
        }
    }

    fn tray_command_from_menu_id(id: &str) -> Option<TrayCommand> {
        match id {
            MENU_ID_SHOW_CLOCK => Some(TrayCommand::FocusClock),
            MENU_ID_SHOW_ALARMS => Some(TrayCommand::ShowAlarmPanel),
            MENU_ID_QUIT => Some(TrayCommand::Quit),
            _ => id
                .strip_prefix(MENU_ID_TIMER_PREFIX)
                .and_then(|secs| secs.parse::<u64>().ok())
                .map(TrayCommand::AddQuickTimer),
        }
    }

    fn create_windows_tray_icon() -> Result<Icon, String> {
        Icon::from_rgba(
            app_icon::clock_face_icon_rgba(app_icon::CLOCK_FACE_ICON_SIZE),
            app_icon::CLOCK_FACE_ICON_SIZE,
            app_icon::CLOCK_FACE_ICON_SIZE,
        )
        .map_err(|error| error.to_string())
    }

    pub use SystemTrayHandle as HandleType;

    #[cfg(test)]
    mod tests {
        use super::{tray_command_from_menu_id, TrayCommand};

        #[test]
        fn maps_show_clock_menu_id() {
            assert_eq!(
                tray_command_from_menu_id("show-clock"),
                Some(TrayCommand::FocusClock)
            );
        }

        #[test]
        fn maps_show_alarms_menu_id() {
            assert_eq!(
                tray_command_from_menu_id("show-alarms"),
                Some(TrayCommand::ShowAlarmPanel)
            );
        }

        #[test]
        fn maps_quit_menu_id() {
            assert_eq!(tray_command_from_menu_id("quit"), Some(TrayCommand::Quit));
        }

        #[test]
        fn maps_quick_timer_menu_id() {
            assert_eq!(
                tray_command_from_menu_id("timer-600"),
                Some(TrayCommand::AddQuickTimer(600))
            );
        }

        #[test]
        fn rejects_invalid_timer_menu_id() {
            assert_eq!(tray_command_from_menu_id("timer-nope"), None);
        }

        #[test]
        fn rejects_unknown_menu_id() {
            assert_eq!(tray_command_from_menu_id("something-else"), None);
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod fallback {
    use std::sync::mpsc::Receiver;

    use super::TrayCommand;

    pub struct SystemTrayHandle;

    impl SystemTrayHandle {
        pub fn shutdown(self) {}
    }

    pub fn start_system_tray() -> Option<(SystemTrayHandle, Receiver<TrayCommand>)> {
        None
    }

    pub use SystemTrayHandle as HandleType;
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
use fallback as implementation;
#[cfg(target_os = "linux")]
use linux as implementation;
#[cfg(target_os = "windows")]
use windows as implementation;

pub use implementation::HandleType as SystemTrayHandle;

#[cfg_attr(not(any(target_os = "linux", target_os = "windows")), allow(dead_code))]
pub fn start_system_tray() -> Option<(SystemTrayHandle, std::sync::mpsc::Receiver<TrayCommand>)> {
    implementation::start_system_tray()
}
