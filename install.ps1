$ErrorActionPreference = 'Stop'

$InstallDir = "$env:USERPROFILE\AppData\Local\MovieBox-Tui"
$ExePath = "$InstallDir\moviebox-tui.exe"
$ZipFile = "$env:TEMP\MovieBox_Windows_x64.zip"
$Url = "https://github.com/mesamirh/MovieBox-Tui/releases/latest/download/MovieBox_Windows_x64.zip"

$IsUpdate = Test-Path $ExePath

if ($IsUpdate) {
    Write-Host ">> Updating MovieBox-Tui to the latest version..." -ForegroundColor Yellow
} else {
    Write-Host ">> Installing MovieBox-Tui..." -ForegroundColor Blue
}

Write-Host ">> Downloading latest release..." -ForegroundColor Blue
Invoke-WebRequest -Uri $Url -OutFile $ZipFile -UseBasicParsing

if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

Write-Host ">> Extracting files..." -ForegroundColor Blue
Expand-Archive -Path $ZipFile -DestinationPath $InstallDir -Force

if ($IsUpdate) {
    Remove-Item $ExePath -Force -ErrorAction SilentlyContinue
}

Rename-Item -Path "$InstallDir\MovieBox.exe" -NewName "moviebox-tui.exe" -Force -ErrorAction SilentlyContinue
Remove-Item $ZipFile -Force

$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notmatch [regex]::Escape($InstallDir)) {
    Write-Host ">> Adding to PATH..." -ForegroundColor Blue
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host "Warning: Please restart your PowerShell window for the PATH changes to take effect." -ForegroundColor Yellow
}

if ($IsUpdate) {
    Write-Host "Success! MovieBox-Tui has been updated. Run 'moviebox-tui' to start." -ForegroundColor Green
} else {
    Write-Host "Success! MovieBox-Tui is installed. Run 'moviebox-tui' to start." -ForegroundColor Green
}
