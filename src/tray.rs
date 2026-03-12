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

#[cfg(not(target_os = "linux"))]
mod linux {
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

pub use linux::HandleType as SystemTrayHandle;

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn start_system_tray() -> Option<(SystemTrayHandle, std::sync::mpsc::Receiver<TrayCommand>)> {
    linux::start_system_tray()
}
