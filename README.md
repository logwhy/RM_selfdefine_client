# Hero Deploy Client

Hero Deploy Client 是一个面向 RoboMaster 2026 英雄机器人“部署 / 吊射”场景的桌面客户端。项目使用 Vue 3 + TypeScript 构建操作界面，使用 Tauri 2 + Rust 处理低延迟 MQTT、UDP 视频接收、H264/HEVC 解码、键鼠控制和状态诊断。

当前项目重点支持两条视频链路：

- 普通图传：UDP 3334 / HEVC。
- 英雄吊射：MQTT 3333 / `CustomByteBlock` / H264，对应协议消息 `0x0310`。

`CustomByteBlock / 0x0310` 是 UI 上的链路说明；真实 MQTT topic 名称是 `CustomByteBlock`。

## 功能概览

- 普通图传接收：监听本机 `0.0.0.0:3334`，接收带 8 字节分片头的 HEVC UDP 包。
- 英雄吊射接收：连接 MQTT broker，订阅 `CustomByteBlock`，解析 protobuf bytes 字段并重组 H264。
- 实时解码：支持 FFmpeg 真实解码，也支持 stub decoder 调试模式。
- 通信状态：显示 MQTT endpoint、部署状态、裁判系统 topic 计数、CustomByteBlock 速率、SPS/PPS/IDR 状态、解码延迟和丢帧。
- 键鼠控制：FPS 模式下将鼠标移动、滚轮、鼠标按键和键盘位图发布到 MQTT topic `KeyboardMouseControl`。
- HUD / 准星：支持准星参数、HUD 编辑、预设保存和全屏比赛模式。
- 本地测试脚本：提供 H264 over MQTT 和 HEVC over UDP 的发送脚本。

## 技术栈

- 前端：Vue 3, TypeScript, Vite, Pinia, Naive UI。
- 桌面壳：Tauri 2。
- 后端：Rust, Tokio, rumqttc, ffmpeg-next。
- 测试脚本：Python 3, paho-mqtt。
- 默认本地 MQTT broker：Mosquitto。

## 项目结构

```text
hero-deploy-client/
  src/                         Vue 前端
    pages/HeroDeployPage.vue   主操作页面、快捷键、抽屉面板
    components/                视频画布、HUD、调试、通信、输入面板
    composables/               前端业务组合逻辑
    stores/                    Pinia 状态持久化
    services/                  Tauri invoke/event 桥接
    types/                     前端类型定义
  src-tauri/                   Tauri / Rust 后端
    src/mqtt/client.rs         MQTT 连接、订阅、键鼠发布、裁判 topic 解析
    src/commands/mqtt.rs       MQTT 相关 Tauri commands
    src/commands/video.rs      视频模式切换、解码器生命周期、mock 源
    src/video/udp_receiver.rs  UDP 3334 HEVC 接收
    src/video/custom_block_*   CustomByteBlock H264 重组
    src/video/decoder.rs       FFmpeg 或 stub 解码器
    src/video/frame_hub.rs     最新解码帧缓存，供前端轮询渲染
  scripts/
    send_h264_custom_block.py  H264 Annex-B -> MQTT CustomByteBlock
    send_hevc_udp.py           HEVC Annex-B -> UDP 3334
    fixtures/                  本地测试视频 fixture
  run-real-decoder.bat         Windows 真实 FFmpeg 解码启动脚本
```

## 协议和端口

### MQTT

默认协议配置：

- 官方 broker：`192.168.12.1:3333`
- 本地调试 broker：`127.0.0.1:3333`
- Custom client IP：协议中通常为 `192.168.12.2`
- CustomByteBlock topic：`CustomByteBlock`
- CustomByteBlock 频率上限：50Hz
- CustomByteBlock data 上限：300 bytes
- CustomByteBlock QoS：协议最高 1，项目本地视频测试默认使用 QoS 0；`KeyboardMouseControl` 按官方客户端示例使用 QoS 2 / ExactlyOnce。

后端会订阅：

- `CustomByteBlock`
- `KeyboardMouseControl`
- `CustomControl`
- `DeployModeStatusSync`
- 裁判系统相关 topic，例如 `GameStatus`、`RobotDynamicStatus`、`RobotStaticStatus`、`Event` 等

后端会发布：

- `KeyboardMouseControl`

`KeyboardMouseControl` 当前字段编码：

```text
field 1 = mouse_x
field 2 = mouse_y
field 3 = mouse_z
field 4 = left_button_down
field 5 = right_button_down
field 6 = keyboard_value
field 7 = mid_button_down
```

鼠标左键会作为 `left_button_down` 发送。项目 UI 将其作为“发射 / 开火”输入处理。要真正向 MQTT 发布键鼠控制，`I` 诊断面板中的 `dry-run` 必须关闭。

### UDP 普通图传

普通图传默认端口是 `3334`。后端实际绑定：

```text
0.0.0.0:3334
```

这表示监听本机所有 IPv4 网卡，不固定绑定 `127.0.0.1` 或某个 `192.168.x.x` 地址。只要 UDP 包发到这台电脑任意本机 IPv4 地址的 `3334`，程序都能收到。

UDP payload 格式：

```text
frame_id          2 bytes, big-endian
fragment_index    2 bytes, big-endian
frame_total_bytes 4 bytes, big-endian
payload           HEVC bytes
```

## 运行方式

### 安装前端依赖

```powershell
cd D:\RM_self_define\hero-deploy-client
npm install
```

### Stub decoder 调试模式

不需要本机 FFmpeg。收到视频包后会生成测试画面，适合验证 MQTT / UDP / UI 链路。

```powershell
npm run tauri:dev:stub
```

### 真实 FFmpeg 解码模式

真实解码 H264 / HEVC，推荐用于最终调试。

当前仓库提供 Windows 批处理：

```powershell
.\run-real-decoder.bat
```

这个脚本假设以下路径存在：

```text
D:\RM_self_define\hero-deploy-client
D:\visual studio2022\community\VC\Auxiliary\Build\vcvars64.bat
C:\Program Files\LLVM\bin
D:\ffmpeg\ffmpeg-8.1-full_build-shared
```

如果新电脑路径不同，需要修改 `run-real-decoder.bat` 顶部这些变量：

```bat
set "ROOT=..."
set "TAURI_DIR=..."
set "VCVARS=..."
set "LLVM_BIN=..."
set "FFMPEG_DIR=..."
```

`FFMPEG_DIR` 必须是 MSVC 可用的 shared build，至少需要：

```text
%FFMPEG_DIR%\include\libavformat\avformat.h
%FFMPEG_DIR%\lib\avformat.lib
%FFMPEG_DIR%\bin\avcodec-62.dll
%FFMPEG_DIR%\bin\avformat-62.dll
```

### 构建前端

```powershell
npm run build
```

### Rust 检查

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
```

真实解码 feature 检查：

```powershell
npm run cargo:check:real
```

## UI 使用

主界面右侧有快速面板：

- 一键英雄吊射：切到 `hero_lob / CustomByteBlock H264`。
- 普通图传：切到 `normal / UDP 3334 / HEVC`。
- 参数：准星参数。
- 调试：解码和传输诊断。
- 通信：MQTT 连接。
- 模式：手动切换视频源、parser mode、mock 源。
- 全屏：进入比赛全屏界面。

通信面板提供两个快速 endpoint：

- `127.0.0.1:3333`：本地 Mosquitto 调试。
- `192.168.12.1:3333`：RoboMaster 官方链路。

切换 endpoint 后，需要点击“连接 MQTT”。如果已经连接到另一个 endpoint，后端会停止旧连接并重新连接。

### 快捷键

```text
H       一键英雄吊射
N       普通图传
P       参数面板
D       调试面板
C       通信面板
U       HUD 编辑器
I       输入诊断面板
Tab     帮助
Enter   进入/退出 FPS 键鼠控制模式
F11     全屏/退出全屏
Esc     关闭面板、退出 FPS 或退出全屏
方向键  微调准星
Shift + 方向键  大步长微调准星
Ctrl + 方向键   小步长微调准星
1/2/3   切换准星预设
Ctrl + 1/2/3 保存准星预设
R       恢复准星默认值
S       保存准星配置
Delete  HUD 编辑时删除选中元素
Ctrl+Z / Ctrl+Y HUD 编辑撤销/重做
```

### 键鼠控制

按 `Enter` 进入 FPS 控制模式后：

- 鼠标移动：生成 `mouse_x` / `mouse_y`。
- 滚轮：生成 `mouse_z`。
- 鼠标左键：`left_button_down`，按发射处理。
- 鼠标右键：`right_button_down`。
- 鼠标中键：`mid_button_down`。
- 键盘：按 bitset 生成 `keyboard_value`。

键盘 bitset：

```text
W=bit0, S=bit1, A=bit2, D=bit3
Shift=bit4, Ctrl=bit5
Q=bit6, E=bit7, R=bit8, F=bit9, G=bit10
Z=bit11, X=bit12, C=bit13, V=bit14, B=bit15
```

输入诊断面板中的关键开关：

- `dry-run`：默认打开。打开时不会向 MQTT 发布 `KeyboardMouseControl`。
- `禁用发射`：关闭时允许鼠标左键作为发射命令发送。

## 本地 MQTT 调试

### Mosquitto 配置

Windows 上可使用 `D:\Mosquitto\mosquitto.conf`，最小配置示例：

```conf
listener 3333
allow_anonymous true
```

启动 broker：

```powershell
mosquitto -c D:\Mosquitto\mosquitto.conf -v
```

也可以作为 Windows 服务运行，确保 `3333` 正在监听：

```powershell
Get-NetTCPConnection -LocalPort 3333
```

### 测试 CustomByteBlock H264

1. 启动客户端。
2. 打开通信面板，选择 `127.0.0.1`，端口 `3333`，连接 MQTT。
3. 打开模式面板，选择 `CustomByteBlock H264` 的 parser mode。
4. 按 `H` 或点击一键英雄吊射。
5. 运行发送脚本。

raw Annex-B 流模式：

```powershell
python scripts\send_h264_custom_block.py `
  --input scripts\fixtures\input_custombyteblock_100k_annexb.h264 `
  --stream-mode raw-annexb `
  --loop
```

packetized-frame 模式：

```powershell
python scripts\send_h264_custom_block.py `
  --input scripts\fixtures\input_custombyteblock_100k_annexb.h264 `
  --stream-mode packetized-frame `
  --loop
```

注意：脚本的 `--stream-mode` 必须和 UI 中的 `customBlockParserMode` 一致：

```text
--stream-mode raw-annexb       -> raw_annexb_stream
--stream-mode packetized-frame -> packetized_frame
```

脚本依赖：

```powershell
python -m pip install paho-mqtt
```

脚本成功连接时会显示：

```text
mqtt_connected reason=Success
```

如果左侧 `CustomByteBlock` 仍然是 0，优先检查：

- UI 左侧 `MQTT Endpoint` 是否为脚本发布的同一个 broker，例如 `127.0.0.1:3333`。
- Mosquitto 是否正在监听 `3333`。
- 是否误连到了 `192.168.12.1:3333`。
- parser mode 是否匹配。
- 是否有旧客户端进程占用同一个 MQTT 链路。

### 测试 UDP HEVC

1. 启动客户端。
2. 按 `N` 切到普通图传。
3. 模式面板中确认 UDP 端口为 `3334`，点击启动 UDP 接收。
4. 使用 HEVC Annex-B 文件发送。

```powershell
python scripts\send_hevc_udp.py `
  --input path\to\input.hevc `
  --host 127.0.0.1 `
  --port 3334 `
  --loop
```

如果发到真实网卡地址，可将 `--host` 改成目标电脑的 `192.168.x.x` 地址。客户端监听的是 `0.0.0.0:3334`。

## 新电脑配置步骤

以下以 Windows 为主，因为当前项目和 `run-real-decoder.bat` 都按 Windows/MSVC 环境组织。

### 1. 安装基础工具

安装：

- Git
- Node.js LTS
- Rust stable：`rustup`
- Visual Studio 2022 Community 或 Build Tools，必须包含 C++ 桌面开发工具链
- Microsoft Edge WebView2 Runtime
- LLVM，确保存在 `clang.exe` 和 `libclang`
- FFmpeg 8.1 shared build for MSVC，必须包含 headers、`.lib` import libraries、`.dll`
- Mosquitto
- Python 3

确认命令可用：

```powershell
git --version
node --version
npm --version
rustc --version
cargo --version
python --version
```

### 2. 拉取项目并安装依赖

```powershell
git clone <repo-url> D:\RM_self_define\hero-deploy-client
cd D:\RM_self_define\hero-deploy-client
npm install
python -m pip install paho-mqtt
```

### 3. 配置 Mosquitto

编辑 `mosquitto.conf`：

```conf
listener 3333
allow_anonymous true
```

启动或重启 Mosquitto，并确认监听：

```powershell
Get-NetTCPConnection -LocalPort 3333
```

### 4. 配置真实解码环境

修改 `run-real-decoder.bat` 的路径：

```bat
set "ROOT=你的项目路径"
set "TAURI_DIR=你的项目路径\src-tauri"
set "VCVARS=你的 VS vcvars64.bat 路径"
set "LLVM_BIN=你的 LLVM bin 路径"
set "FFMPEG_DIR=你的 FFmpeg 8.1 shared build 路径"
```

检查 FFmpeg 目录必须满足：

```text
include\libavformat\avformat.h
lib\avformat.lib
bin\avcodec-62.dll
bin\avformat-62.dll
```

### 5. 先跑 stub 模式验证 UI

```powershell
npm run tauri:dev:stub
```

确认窗口能打开、快捷键和面板正常。

### 6. 再跑真实解码模式

```powershell
.\run-real-decoder.bat
```

脚本会：

- 调用 VS `vcvars64.bat`。
- 清理旧 `ffmpeg-next` / `ffmpeg-sys-next` 构建产物。
- 将 FFmpeg crate 锁定到 `8.1.0`。
- 设置 `PATH` 和 `LIBCLANG_PATH`。
- 启动 `npm run tauri:dev:real`。

### 7. 验证本地 CustomByteBlock 链路

客户端中选择：

```text
MQTT endpoint: 127.0.0.1:3333
模式: HERO LOB / 0x0310 / H264
parser: raw_annexb_stream 或 packetized_frame
```

然后运行匹配的 Python 脚本。左侧 `CustomByteBlock` 计数应增长，右侧链路监控应显示 rate 和 bitrate。

## 常见问题

### MQTT 显示 ONLINE，但 CustomByteBlock 仍然是 0

优先检查左侧 `MQTT Endpoint`。它必须和发送脚本的 `host:port` 完全一致。

本地测试时两边都应为：

```text
127.0.0.1:3333
```

真实机器人链路时通常为：

```text
192.168.12.1:3333
```

### Python 脚本显示 sent，但 UI 没收到

`sent` 只代表脚本向它连接的 broker 发布了消息。还要确认客户端也连接同一个 broker、topic 是 `CustomByteBlock`、parser mode 匹配。

### packetized-frame 模式没画面

UI parser 必须选 `packetized_frame`。如果 UI 还是 `raw_annexb_stream`，会收到包但无法按正确格式重组。

### raw-annexb 模式等待 SPS/PPS

确认输入文件是 Annex-B H264，并包含 SPS/PPS。Debug 面板查看：

```text
h264SeenSps
h264SeenPps
h264SeenIdr
h264LastNalType
h264NalUnitsParsed
```

### UDP 普通图传收不到

确认：

- UI 当前是 `normal / udp_hevc / hevc`。
- UDP 接收已启动。
- 端口是 `3334` 或你手动设置的端口。
- 防火墙允许 UDP 入站。
- 发送端目标 IP 是本机可达地址，端口匹配。

### F11 不全屏

F11 在项目中由前端全局快捷键处理，会调用 Tauri window fullscreen。若系统或键盘驱动拦截 F11，可使用右侧快速面板的“全屏”按钮。

### 左键不发射

检查输入诊断面板：

- `dry-run` 必须关闭。
- `禁用发射` 必须关闭。
- 必须处在 FPS 控制模式，按 `Enter` 进入。

### 真实解码模式启动失败

常见原因：

- `run-real-decoder.bat` 中路径仍是旧电脑路径。
- FFmpeg 不是 MSVC shared build，缺少 `.lib`。
- `LIBCLANG_PATH` 没指向 LLVM bin。
- VS C++ 工具链未安装。
- `PATH` 中找不到 FFmpeg dll。

## 开发注意事项

- 前端状态会写入 `localStorage`，包括 MQTT endpoint、视频模式、输入控制和 HUD 配置。
- MQTT client id 由通信面板的“选择机器人”选项决定，默认 `1`（红方英雄机器人）。
- rumqttc 内部队列容量当前为 `100`，用于承受 50Hz CustomByteBlock 和其它 topic 的事件压力。
- `CustomByteBlock` 收包计数在任何视频模式都会增长；只有切到 `custombyteblock_h264` 时才会进入 H264 重组和解码。
- 普通 UDP 图传和 CustomByteBlock H264 是两条独立链路，调试时不要混淆 `3334/UDP` 和 `3333/MQTT`。

## 常用命令速查

```powershell
# 安装依赖
npm install
python -m pip install paho-mqtt

# 前端构建
npm run build

# Tauri stub decoder
npm run tauri:dev:stub

# Tauri real decoder
.\run-real-decoder.bat

# Rust 检查
cargo check --manifest-path src-tauri\Cargo.toml

# 本地 H264 CustomByteBlock raw Annex-B
python scripts\send_h264_custom_block.py --input scripts\fixtures\input_custombyteblock_100k_annexb.h264 --stream-mode raw-annexb --loop

# 本地 H264 CustomByteBlock packetized-frame
python scripts\send_h264_custom_block.py --input scripts\fixtures\input_custombyteblock_100k_annexb.h264 --stream-mode packetized-frame --loop

# 本地 HEVC UDP
python scripts\send_hevc_udp.py --input path\to\input.hevc --host 127.0.0.1 --port 3334 --loop
```
