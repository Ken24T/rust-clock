# Windows Installer

Rust Clock ships with an Inno Setup installer definition for Windows.

## What It Produces

Running the installer build creates a versioned setup executable under `dist/windows/`.

The installed app layout is:

- executable: `%LocalAppData%\Programs\Rust Clock\rust-clock.exe`
- Start Menu shortcut: `Rust Clock`
- optional desktop shortcut
- optional startup shortcut for sign-in launch

The installer is per-user and does not require administrator privileges.

## Build Prerequisites

- Rust stable toolchain
- PowerShell
- Inno Setup 6

Install Inno Setup with:

```powershell
winget install JRSoftware.InnoSetup
```

## Build Command

From the repo root:

```powershell
pwsh -File .\installer\windows\build-installer.ps1
```

The build helper will:

1. read the app version from `Cargo.toml`
2. run `cargo build --release`
3. compile `installer/windows/rust-clock.iss`
4. emit `dist/windows/rust-clock-setup-<version>.exe`

## Installer Behaviour

The installer:

- installs the release executable into `%LocalAppData%\Programs\Rust Clock`
- creates a Start Menu shortcut by default
- offers optional desktop and startup shortcuts
- registers a normal uninstall entry
- offers to launch Rust Clock after installation

## Notes

- Rust Clock stores user settings and alarm data through `directories::ProjectDirs`, so installation does not move existing config into the app folder.
- The installer packages only the Windows release executable because the current app embeds its window and tray icon resources directly in the binary.