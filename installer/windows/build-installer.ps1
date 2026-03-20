param(
    [string]$OutputDir,
    [string]$InstallerCompilerPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptRoot '..\..')

if (-not $OutputDir) {
    $OutputDir = Join-Path $repoRoot 'dist\windows'
}

if (-not $InstallerCompilerPath) {
    $InstallerCompilerPath = @(
        (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
        'C:\Program Files (x86)\Inno Setup 6\ISCC.exe',
        'C:\Program Files\Inno Setup 6\ISCC.exe'
    ) | Where-Object { Test-Path $_ } | Select-Object -First 1
}

if (-not $InstallerCompilerPath) {
    throw 'Inno Setup 6 was not found. Install it with `winget install JRSoftware.InnoSetup` or pass -InstallerCompilerPath.'
}

$cargoTomlPath = Join-Path $repoRoot 'Cargo.toml'
$cargoToml = Get-Content $cargoTomlPath -Raw

if ($cargoToml -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
    throw "Could not read the package version from $cargoTomlPath"
}

$appVersion = $Matches[1]
$sourceExe = Join-Path $repoRoot 'target\release\rust-clock.exe'
$installerScript = Join-Path $repoRoot 'installer\windows\rust-clock.iss'

Push-Location $repoRoot
try {
    Write-Host "Building Rust Clock $appVersion release binary..."
    & cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw 'cargo build --release failed.'
    }
}
finally {
    Pop-Location
}

if (-not (Test-Path $sourceExe)) {
    throw "Release executable not found at $sourceExe"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

Write-Host "Compiling installer with Inno Setup..."
& $InstallerCompilerPath "/DAppVersion=$appVersion" "/DSourceExe=$sourceExe" "/DOutputDir=$OutputDir" $installerScript
if ($LASTEXITCODE -ne 0) {
    throw 'Inno Setup compilation failed.'
}

$artifact = Get-ChildItem -Path $OutputDir -Filter "rust-clock-setup-$appVersion.exe" | Select-Object -First 1
if (-not $artifact) {
    throw "Installer artifact rust-clock-setup-$appVersion.exe was not found in $OutputDir"
}

Write-Host 'Installer build complete:'
$artifact | Select-Object FullName, Length, LastWriteTime