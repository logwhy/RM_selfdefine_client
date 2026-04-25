@echo off
setlocal

set "ROOT=D:\RM_self_define\hero-deploy-client"
set "TAURI_DIR=D:\RM_self_define\hero-deploy-client\src-tauri"
set "VCVARS=D:\visual studio2022\community\VC\Auxiliary\Build\vcvars64.bat"
set "LLVM_BIN=C:\Program Files\LLVM\bin"
set "FFMPEG_DIR=D:\ffmpeg\ffmpeg-8.1-full_build-shared"

rem Important: avoid old vcpkg/pkg-config/bindgen env leaking from previous runs.
set "PKG_CONFIG="
set "PKG_CONFIG_PATH="
set "FFMPEG_PKG_CONFIG_PATH="
set "VCPKG_ROOT="
set "VCPKGRS_DYNAMIC="
set "BINDGEN_EXTRA_CLANG_ARGS="
set "CC="
set "CXX="
set "AR="
set "LD="

if not exist "%VCVARS%" (
  echo [ERROR] vcvars64.bat not found: %VCVARS%
  exit /b 1
)

if not exist "%FFMPEG_DIR%\include\libavformat\avformat.h" (
  echo [ERROR] FFmpeg 8.1 include not found:
  echo        %FFMPEG_DIR%\include\libavformat\avformat.h
  exit /b 1
)

if not exist "%FFMPEG_DIR%\lib\avformat.lib" (
  echo [ERROR] FFmpeg 8.1 MSVC import lib not found:
  echo        %FFMPEG_DIR%\lib\avformat.lib
  echo [HINT] You need a shared build that provides .lib import libraries, not only .dll.a files.
  exit /b 1
)

call "%VCVARS%"

set "LIBCLANG_PATH=%LLVM_BIN%"
set "PATH=%FFMPEG_DIR%\bin;%LLVM_BIN%;%PATH%"

cd /d "%TAURI_DIR%" || exit /b 1

echo [INFO] Tool check
where cl
where link
where lib
where clang
where ffmpeg
where avcodec-62.dll
where avformat-62.dll

echo [INFO] FFmpeg version
ffmpeg -version

echo [INFO] Clean old patched 7.1 artifacts
cargo clean -p ffmpeg-sys-next
cargo clean -p ffmpeg-next

echo [INFO] Force ffmpeg crates to 8.1.0
cargo update -p ffmpeg-next --precise 8.1.0
cargo update -p ffmpeg-sys-next --precise 8.1.0

echo [INFO] Check feature propagation
cargo tree -e features --no-default-features --features real-decoder -i ffmpeg-sys-next

echo [INFO] Start Tauri real decoder mode
cd /d "%ROOT%" || exit /b 1
npm run tauri:dev:real
