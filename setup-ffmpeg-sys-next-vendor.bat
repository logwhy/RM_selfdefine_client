@echo off
setlocal enabledelayedexpansion

set "PROJECT_ROOT=D:\RM_self_define\hero-deploy-client"
set "TAURI_DIR=%PROJECT_ROOT%\src-tauri"
set "VENDOR_DIR=%TAURI_DIR%\vendor\ffmpeg-sys-next-7.1.0"

echo [INFO] Setup local patched ffmpeg-sys-next vendor...

set "SRC_CRATE="
for /d %%D in ("%USERPROFILE%\.cargo\registry\src\*\ffmpeg-sys-next-7.1.0") do (
  set "SRC_CRATE=%%~fD"
)

if "%SRC_CRATE%"=="" (
  echo [ERROR] Cannot find ffmpeg-sys-next-7.1.0 in Cargo registry.
  echo [HINT] It should exist because your previous build downloaded it.
  echo [HINT] If it really does not exist, temporarily remove [patch.crates-io] from src-tauri\Cargo.toml, then run:
  echo        cd /d %TAURI_DIR%
  echo        cargo update -p ffmpeg-sys-next --precise 7.1.0
  echo        cargo fetch
  exit /b 1
)

echo [INFO] Found source crate:
echo        %SRC_CRATE%

if exist "%VENDOR_DIR%" (
  echo [INFO] Remove old vendor dir...
  rmdir /s /q "%VENDOR_DIR%"
)

if not exist "%TAURI_DIR%\vendor" mkdir "%TAURI_DIR%\vendor"

echo [INFO] Copy to vendor...
xcopy /E /I /Y "%SRC_CRATE%" "%VENDOR_DIR%" >nul
if errorlevel 1 (
  echo [ERROR] Failed to copy crate.
  exit /b 1
)

if not exist "%VENDOR_DIR%\Cargo.toml" (
  echo [ERROR] Copy finished but Cargo.toml is still missing:
  echo        %VENDOR_DIR%\Cargo.toml
  exit /b 1
)

echo [INFO] Patch build.rs: disable bindgen layout tests...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$p='%VENDOR_DIR%\build.rs'; $s=Get-Content -Path $p -Raw; if ($s -notmatch 'layout_tests\(false\)') { $s=$s -replace 'bindgen::Builder::default\(\)', 'bindgen::Builder::default().layout_tests(false)'; Set-Content -Path $p -Value $s -NoNewline -Encoding UTF8 }; if ((Get-Content -Path $p -Raw) -match 'layout_tests\(false\)') { Write-Host '[INFO] build.rs patched successfully.' } else { Write-Host '[ERROR] patch failed.'; exit 1 }"
if errorlevel 1 exit /b 1

echo [INFO] Done. Vendor crate is ready:
echo        %VENDOR_DIR%
exit /b 0
