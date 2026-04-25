@echo off
setlocal enabledelayedexpansion

echo [INFO] Preparing real decoder environment (MSVC + vcpkg FFmpeg)...

set "PROJECT_ROOT=%~dp0"
set "PROJECT_ROOT=%PROJECT_ROOT:~0,-1%"
set "VCPKG_ROOT=D:\vcpkg"
set "LLVM_BIN=C:\Program Files\LLVM\bin"
set "VCVARS=D:\visual studio2022\community\VC\Auxiliary\Build\vcvars64.bat"
set "VCPKG_INSTALLED=%VCPKG_ROOT%\installed\x64-windows"

if not exist "%VCVARS%" (
  echo [ERROR] vcvars64.bat not found: %VCVARS%
  exit /b 1
)

if not exist "%LLVM_BIN%\clang.exe" (
  echo [ERROR] clang.exe not found: %LLVM_BIN%\clang.exe
  echo [HINT] Install LLVM for Windows and keep LIBCLANG_PATH pointing to LLVM\bin.
  exit /b 1
)

if not exist "%VCPKG_INSTALLED%\include\libavcodec\avcodec.h" (
  echo [ERROR] FFmpeg headers not found in vcpkg: %VCPKG_INSTALLED%\include
  echo [HINT] Run: vcpkg install ffmpeg:x64-windows
  exit /b 1
)

if not exist "%VCPKG_INSTALLED%\lib\avcodec.lib" (
  echo [ERROR] FFmpeg MSVC import libs not found: %VCPKG_INSTALLED%\lib\avcodec.lib
  echo [HINT] Do not use MSYS2/MinGW FFmpeg for MSVC target. Use vcpkg x64-windows.
  exit /b 1
)

if not exist "%VCPKG_INSTALLED%\tools\pkgconf\pkgconf.exe" (
  echo [ERROR] pkgconf.exe not found: %VCPKG_INSTALLED%\tools\pkgconf\pkgconf.exe
  exit /b 1
)

REM Load MSVC first so cl/link/lib are the MSVC versions.
call "%VCVARS%"

REM Keep this environment pure MSVC. Do NOT prepend MSYS2 or Git Bash here.
set "PATH=%VCPKG_INSTALLED%\bin;%VCPKG_INSTALLED%\tools\pkgconf;%LLVM_BIN%;%PATH%"

set "LIBCLANG_PATH=%LLVM_BIN%"
set "VCPKGRS_TRIPLET=x64-windows"
set "VCPKG_DEFAULT_TRIPLET=x64-windows"
set "PKG_CONFIG=%VCPKG_INSTALLED%\tools\pkgconf\pkgconf.exe"
set "PKG_CONFIG_PATH=%VCPKG_INSTALLED%\lib\pkgconfig"
set "PKG_CONFIG_ALLOW_SYSTEM_LIBS=1"
set "PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1"
set "FFMPEG_DIR=%VCPKG_INSTALLED%"
set "FFMPEG_INCLUDE_DIR=%VCPKG_INSTALLED%\include"
set "FFMPEG_LIB_DIR=%VCPKG_INSTALLED%\lib"
set "BINDGEN_EXTRA_CLANG_ARGS=--target=x86_64-pc-windows-msvc -I%FFMPEG_INCLUDE_DIR%"

REM Clear potentially polluted flags from conda/msys/gcc toolchains.
set "CC="
set "CXX="
set "AR="
set "LD="
set "CFLAGS="
set "CPPFLAGS="
set "CXXFLAGS="
set "LDFLAGS="
set "CL="
set "CMAKE_C_FLAGS="
set "CMAKE_CXX_FLAGS="
set "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER="

echo [INFO] Tool check:
where cl
where link
where lib
where clang
where pkgconf
where ffmpeg

echo [INFO] FFmpeg pkg-config versions:
"%PKG_CONFIG%" --modversion libavcodec
"%PKG_CONFIG%" --modversion libavformat
"%PKG_CONFIG%" --modversion libavutil

cd /d "%PROJECT_ROOT%\src-tauri"

echo [INFO] Cleaning ffmpeg crates only...
cargo clean -p ffmpeg-sys-next
cargo clean -p ffmpeg-next

cd /d "%PROJECT_ROOT%"
echo [INFO] Starting Tauri real decoder mode...
call npm run tauri:dev:real
exit /b %errorlevel%
