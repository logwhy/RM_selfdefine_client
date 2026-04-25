@echo off
setlocal EnableExtensions EnableDelayedExpansion

rem Put this file in D:\RM_self_define\hero-deploy-client and run it once.
set "PROJECT_DIR=%~dp0"
if "%PROJECT_DIR:~-1%"=="\" set "PROJECT_DIR=%PROJECT_DIR:~0,-1%"
set "SRC_TAURI=%PROJECT_DIR%\src-tauri"
set "CARGO_TOML=%SRC_TAURI%\Cargo.toml"
set "BACKUP=%SRC_TAURI%\Cargo.toml.before_vendor_setup.bak"
set "VENDOR_DIR=%SRC_TAURI%\vendor\ffmpeg-sys-next-7.1.0"

echo [INFO] Setup local patched ffmpeg-sys-next vendor v2...
echo [INFO] Project: %PROJECT_DIR%

if not exist "%CARGO_TOML%" (
  echo [ERROR] Cannot find: %CARGO_TOML%
  exit /b 1
)

rem 1) Temporarily remove [patch.crates-io], otherwise Cargo refuses to fetch before vendor exists.
copy /Y "%CARGO_TOML%" "%BACKUP%" >nul
if errorlevel 1 (
  echo [ERROR] Failed to backup Cargo.toml
  exit /b 1
)

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$p = '%CARGO_TOML%';" ^
  "$lines = Get-Content -LiteralPath $p;" ^
  "$out = New-Object System.Collections.Generic.List[string];" ^
  "$skip = $false;" ^
  "foreach ($l in $lines) {" ^
  "  if ($l -match '^\s*\[patch\.crates-io\]\s*$') { $skip = $true; continue }" ^
  "  if ($skip) {" ^
  "    if ($l -match '^\s*\[') { $skip = $false; $out.Add($l) }" ^
  "    else { continue }" ^
  "  } else { $out.Add($l) }" ^
  "}" ^
  "Set-Content -LiteralPath $p -Value $out -Encoding UTF8"

if errorlevel 1 (
  echo [ERROR] Failed to temporarily remove [patch.crates-io].
  copy /Y "%BACKUP%" "%CARGO_TOML%" >nul
  exit /b 1
)

rem 2) Fetch the crate source from crates.io into Cargo registry.
cd /d "%SRC_TAURI%"
echo [INFO] Fetching ffmpeg-sys-next 7.1.0 from crates.io...
cargo update -p ffmpeg-sys-next --precise 7.1.0
cargo fetch
set "FETCH_EXIT=%ERRORLEVEL%"

rem Restore Cargo.toml regardless of fetch success.
copy /Y "%BACKUP%" "%CARGO_TOML%" >nul

if not "%FETCH_EXIT%"=="0" (
  echo [ERROR] cargo fetch/update failed.
  echo [HINT] Try manually:
  echo        cd /d %SRC_TAURI%
  echo        cargo update -p ffmpeg-sys-next --precise 7.1.0
  exit /b 1
)

rem 3) Locate the downloaded crate source.
if defined CARGO_HOME (
  set "CARGO_SRC_ROOT=%CARGO_HOME%\registry\src"
) else (
  set "CARGO_SRC_ROOT=%USERPROFILE%\.cargo\registry\src"
)

echo [INFO] Searching crate source under: %CARGO_SRC_ROOT%
set "CRATE_SRC="
for /f "usebackq delims=" %%D in (`powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$root = '%CARGO_SRC_ROOT%';" ^
  "if (Test-Path -LiteralPath $root) {" ^
  "  Get-ChildItem -LiteralPath $root -Directory -Recurse -Filter 'ffmpeg-sys-next-7.1.0' -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty FullName" ^
  "}"`) do (
  set "CRATE_SRC=%%D"
)

if not defined CRATE_SRC (
  echo [ERROR] Cannot find ffmpeg-sys-next-7.1.0 in Cargo registry after fetch.
  echo [DEBUG] Cargo registry root checked: %CARGO_SRC_ROOT%
  exit /b 1
)

echo [INFO] Found crate source:
echo        !CRATE_SRC!

rem 4) Copy to local vendor directory.
if exist "%VENDOR_DIR%" rmdir /S /Q "%VENDOR_DIR%"
mkdir "%SRC_TAURI%\vendor" 2>nul
xcopy /E /I /Y "!CRATE_SRC!" "%VENDOR_DIR%" >nul
if errorlevel 1 (
  echo [ERROR] Failed to copy crate source to vendor.
  exit /b 1
)

rem 5) Patch build.rs: disable bindgen layout tests.
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$p = '%VENDOR_DIR%\build.rs';" ^
  "$s = Get-Content -LiteralPath $p -Raw;" ^
  "if ($s -notmatch 'layout_tests\(false\)') {" ^
  "  $s = $s -replace 'bindgen::Builder::default\(\)', 'bindgen::Builder::default().layout_tests(false)';" ^
  "  Set-Content -LiteralPath $p -Value $s -Encoding UTF8;" ^
  "}" ^
  "if ((Get-Content -LiteralPath $p -Raw) -notmatch 'layout_tests\(false\)') { throw 'patch failed' }"

if errorlevel 1 (
  echo [ERROR] Failed to patch build.rs.
  exit /b 1
)

echo [OK] Local patched ffmpeg-sys-next is ready:
echo      %VENDOR_DIR%
echo [OK] Verified patch:
findstr /C:"layout_tests(false)" "%VENDOR_DIR%\build.rs"
echo.
echo [NEXT] Run:
echo        %PROJECT_DIR%\run-real-decoder.bat
exit /b 0
