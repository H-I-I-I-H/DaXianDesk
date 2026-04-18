# 工程基线 / Engineering Baseline

最后一次从全仓源码核验：2026-04-18

> 本文件只记录**已经通过当前源码核验**的事实。
> 这里的中文用于解释，English symbol / path 用于保证 Codex / Claude Code 检索稳定。
> 若与源码冲突，以源码为准，并同步更新本文件。

---

## 0. 当前已确认的维护纪律（Current Maintenance Discipline）

- 后续修改只能改代码/文档，不替用户执行 `git commit`。
- 用户明确要求不由 Codex 执行编译命令；验证优先使用源码检索、静态 diff、定向 grep。
- 重要代码变更后必须同步当前 docs 文档体系，不恢复已废弃的 `DOCS.md` / `docs/CHANGELOG.md` / 旧项目记忆文档。
- 文档永远是辅助记忆；若文档与源码冲突，以当前源码和当前 diff 为准。

### 0.1 2026-04-16 黑屏 overlay 输入性能基线

已修复 Android AccessibilityService 内黑屏 overlay 的错误动态防触摸逻辑，源码锚点：

- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`

当前基线：

- `onMouseInput(...)` 不能再提交黑屏触摸恢复/穿透相关 handler 任务。
- `onstart_overlay(...)` 和 50ms `runnable` 不能再通过 `WindowManager.updateViewLayout(...)` 动态切换 overlay touch flags。
- `BIS` 查询语义保留：`BIS = overLay.visibility != View.GONE`。
- 黑屏功能的运行时边界是 overlay 可见性切换；远程输入性能问题不要扩散修改到 Rust 协议、`PIXEL_SIZE*`、帧传输或侧按钮协议。

### 0.2 2026-04-17 防触摸功能基线

已新增独立透明 `touchBlockOverlay`，源码锚点：

- `src/common.rs`
- `src/flutter_ffi.rs`
- `libs/scrap/src/android/pkg2230.rs`
- `libs/scrap/src/android/ffi.rs`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`
- `flutter/lib/models/input_model.dart`
- `flutter/lib/common.dart`
- `flutter/lib/common/widgets/overlay.dart`

当前基线：

- 侧按钮文本为"开防触"/"关防触"，放在"开穿透"/"关穿透"之后。
- 协议为 `wheeltouch -> MOUSE_TYPE_TOUCHBLOCK=11 -> mask=43 -> TouchBlock_Management -> touch_block`。
- 防触摸只操作独立 `touchBlockOverlay`，不得复用黑屏 `overLay`。
- 空闲默认吸收本地触摸，远程活跃时 500ms 穿透；watchdog 100ms tick，只有状态变化才更新 WindowManager。
- 此功能是标准 Android APK 能力范围内的近似防触，不承诺拦截恶意本地干扰或系统级手势。

### 0.3 2026-04-18 安卓状态监测基线

当前基线：

- 协议字段：`Misc.daxian_status = 39`。
- Android 查询入口：`call_main_service_get_by_name("daxian_status")`。
- PC Flutter 事件：`update_daxian_status`，payload key 为 `status`。
- UI model：`DaxianStatusModel`，默认显示，配置键为 `show_daxian_status_monitor`。
- Toolbar 会话开关：`show-daxian-status-monitor`。
- UI Widget：`RemoteStatusMonitors` 组合 `QualityMonitor` 与 `DaxianStatusMonitor`，只有两者同时显示时插入 6px 间距。
- Sciter UI 只保留 `update_daxian_status` 空实现，当前面板只在 Flutter UI 显示。

---

## 1. 项目身份（Project Identity）

### 1.1 包与产品信息

已在以下文件核验：

- `Cargo.toml`
- `flutter/pubspec.yaml`
- `flutter/android/app/build.gradle`
- `flutter/android/app/src/main/AndroidManifest.xml`
- `libs/hbb_common/src/config.rs`

当前事实：

- Rust crate：`rustdesk`
- Rust crate version：`5.2.0`
- Rust library：`librustdesk`
- Flutter package：`flutter_hbb`
- Flutter version：`5.2.0+58`
- 产品名（runtime app name）：`DaxianMeeting`
- Android package：`com.daxian.dev`
- Android visible label：`大仙会议`
- 配置组织名（runtime org）：`com.carriez`

### 1.2 品牌与命名现实（Branding Reality）

项目已经做了深度品牌替换，但**没有彻底收口**：

- Android 动态库实际加载：`libdaxian.so`
- Android Kotlin `System.loadLibrary("daxian")`
- Flutter Android 侧 `DynamicLibrary.open('libdaxian.so')`
- Windows 侧仍加载：`librustdesk.dll`
- Rust `APP_NAME` 为 `DaxianMeeting`
- Android manifest deep link scheme：`daxian`
- Rust `get_uri_prefix()` 由 `APP_NAME` 推导，当前与 manifest scheme 并不完全一致

结论：

- 这是一个**品牌迁移已经进行但仍有历史残留**的仓库。
- 修改品牌、URI scheme、SO / DLL 名称时，必须全链路重查。

---

## 2. 真实顶层架构（True Top-Level Architecture）

### 2.1 Rust 核心层（Rust Core）

入口与总装配：

- `src/main.rs`
- `src/lib.rs`
- `src/core_main.rs`

核心模块：

- client / session / bridge
  - `src/client.rs`
  - `src/ui_session_interface.rs`
  - `src/flutter.rs`
  - `src/flutter_ffi.rs`
  - `src/ui_interface.rs`
  - `src/ui_cm_interface.rs`
- server
  - `src/server.rs`
  - `src/server/connection.rs`
  - `src/server/video_service.rs`
  - `src/server/audio_service.rs`
  - `src/server/display_service.rs`
  - `src/server/input_service.rs`
  - `src/server/terminal_service.rs`
  - `src/server/printer_service.rs`
  - `src/server/portable_service.rs`
- runtime / shared
  - `src/common.rs`
  - `src/ipc.rs`
  - `src/clipboard.rs`
  - `src/clipboard_file.rs`
  - `src/platform/`
  - `libs/hbb_common/`

### 2.2 Flutter UI 层（Flutter UI Layer）

应用入口与主分流：

- `flutter/lib/main.dart`
- `flutter/lib/common.dart`

主要分层：

- 桌面：`flutter/lib/desktop/`
- 移动：`flutter/lib/mobile/`
- 数据模型：`flutter/lib/models/`
- 原生桥接：`flutter/lib/native/`
- 工具层：`flutter/lib/utils/`
- web 适配：`flutter/lib/web/`
- plugin UI：`flutter/lib/plugin/`

说明：

- 当前桌面端不是单窗口直连模型，而是包含 `desktop_multi_window` 多窗口分流。
- 终端、文件管理、远程控制、view camera、port forward 都有对应窗口/页面路径。

### 2.3 旧桌面 UI / 兼容路径（Legacy Sciter UI Path）

以下路径仍存在且应被视为真实维护面：

- `src/ui.rs`
- `src/ui/`
  - `remote.rs`
  - `cm.rs`
  - 多个 `.html` / `.tis` / `.css`

结论：

- 不能把仓库理解成“只有 Flutter UI”。
- 桌面启动流程、安装流程、旧参数兼容、部分桌面行为仍需关注 `src/ui/`。

### 2.4 Android 原生层（Android Native Layer）

Rust JNI：

- 主路由：`libs/scrap/src/android/pkg2230.rs`
- 兼容副路由：`libs/scrap/src/android/ffi.rs`
- 路由声明：`libs/scrap/src/android/mod.rs`（当前只 `pub mod pkg2230;`）

Kotlin / Java：

- `flutter/android/app/src/main/kotlin/com/daxian/dev/oFtTiPzsqzBHGigp.kt`
  - 主 `FlutterActivity`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/XerQvgpGBzr8FDFr.kt`
  - 权限 / 透明 Activity
- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`
  - `MainService`, `MediaProjection`, keep-alive
- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`
  - `AccessibilityService`, 输入 / 截图 / overlay / fallback
- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFrLMwitwQbfu7AC.kt`
  - `FloatWindowService`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/BootReceiver.kt`
  - 开机启动接收器
- `flutter/android/app/src/main/kotlin/com/daxian/dev/ig2xH1U3RDNsb7CS.kt`
  - 剪贴板桥
- `flutter/android/app/src/main/kotlin/com/daxian/dev/EqljohYazB0qrhnj.kt`
  - 图像辅助 / 节点可视化
- `flutter/android/app/src/main/kotlin/com/daxian/dev/common.kt`
  - Android 全局状态
- `flutter/android/app/src/main/kotlin/com/daxian/dev/KeyboardKeyEventMapper.kt`
  - 键盘映射
- `flutter/android/app/src/main/kotlin/com/daxian/dev/VolumeController.kt`
  - 音量控制
- `flutter/android/app/src/main/kotlin/pkg2230.kt`
  - 主 JNI bridge
- `flutter/android/app/src/main/kotlin/ffi.kt`
  - 兼容 JNI bridge
- `p50.java`, `q50.java`
  - XOR / 混淆辅助

### 2.5 共享协议层（Shared Protocol Layer）

- `libs/hbb_common/protos/message.proto`
- `libs/hbb_common/src/config.rs`
- `libs/hbb_common/src/lib.rs`
- `libs/hbb_common/build.rs`

这里定义并支撑：

- protobuf 消息
- config / id / app name / option
- 网络 / socket / stream 共通层

### 2.6 账号 / HTTP / 同步子系统（HBBS HTTP Subsystem）

真实存在并被多处调用：

- `src/hbbs_http/account.rs`
- `src/hbbs_http/downloader.rs`
- `src/hbbs_http/http_client.rs`
- `src/hbbs_http/record_upload.rs`
- `src/hbbs_http/sync.rs`

它不是边角料，而是一条独立维护面：

- OIDC device auth / account auth
- 下载任务与进度轮询
- 录像上传
- 与 server 连接相关的 sync / signal / pro 状态

### 2.7 Windows 隐私模式 / 虚拟显示器

- `src/privacy_mode.rs`
- `src/privacy_mode/win_virtual_display.rs`
- `src/privacy_mode/win_topmost_window.rs`
- `src/privacy_mode/win_mag.rs`
- `src/privacy_mode/win_exclude_from_capture.rs`
- `src/privacy_mode/win_input.rs`
- `src/virtual_display_manager.rs`

结论：

- 这是 Windows 平台的重要功能面，不是附属实验代码。
- `daxian_virtual_displays` 已成为实际 key / 常量的一部分。

### 2.8 Plugin 框架

- Rust：
  - `src/plugin/mod.rs`
  - `src/plugin/manager.rs`
  - `src/plugin/plugins.rs`
  - `src/plugin/native_handlers/`
- Flutter：
  - `flutter/lib/plugin/`

Feature gate：

- Cargo feature：`plugin_framework`
- 只有在 `feature = "flutter"` 且 `feature = "plugin_framework"` 且非 mobile 平台时才编译 Rust plugin 模块

结论：

- 插件代码存在，但**默认不能假设在普通构建里活跃**。

---

## 3. 已核验的主链路（Verified Main Flows）

### 3.1 启动与界面分流（Startup and UI Split）

#### Rust 侧

- `src/main.rs` 在不同 target / feature 下选择：
  - mobile / flutter / cli / sciter path
- `src/core_main.rs` 处理：
  - 安装 / tray / server / connection manager / 参数分流 / elevate / quick support

#### Flutter 侧

- `flutter/lib/main.dart` 决定：
  - main desktop window
  - remote / file transfer / terminal / port-forward multi-window
  - connection manager
  - install page
  - mobile app

结论：

- 启动链路是 **Rust main + Flutter main 双侧分流**，不能只看一边。

### 3.2 自定义 Android 控制命令链（Custom Android Control Command Chain）

主链路锚点：

1. UI 按钮：`flutter/lib/common/widgets/overlay.dart`
2. UI callback / overlay 控制：`flutter/lib/common.dart`
3. Dart 命令编码：`flutter/lib/models/input_model.dart`
4. Rust FFI：`src/flutter_ffi.rs`
5. 会话层：`src/ui_session_interface.rs`
6. 客户端消息构造：`src/client.rs`
7. 协议：`libs/hbb_common/protos/message.proto`
8. 服务端接收与分发：`src/server/connection.rs`
9. JNI bridge：`libs/scrap/src/android/pkg2230.rs`
10. Kotlin service 执行：`DFm8Y8iMScvB2YDw.kt` / `nZW99cdXQ0COhB2o.kt`

已核到的命令字符串：

- `wheelblank`
- `wheelbrowser`
- `wheelanalysis`
- `wheelback`
- `wheelstart`
- `wheelstop`

已核到的 type 映射：

- `MOUSE_TYPE_BLANK = 5`
- `MOUSE_TYPE_BROWSER = 6`
- `MOUSE_TYPE_Analysis = 7`
- `MOUSE_TYPE_GoBack = 8`
- `MOUSE_TYPE_START = 9`
- `MOUSE_TYPE_STOP = 10`

说明：

- 这条链路是**真实的跨层协议链**，不是 UI 本地逻辑。

### 3.3 Android 三条采集路径（Three Android Capture Paths）

当前源码中存在三条实际路径：

1. 正常共享路径（`normal MediaProjection video path`）
   - `MediaProjection` + `ImageReader`
   - `DFm8Y8iMScvB2YDw.kt`
   - raw frame 最终进入 `VIDEO_RAW`
2. 穿透路径（`SKL pass-through path`）
   - 受 `SKL` 控制
   - 依赖 `AccessibilityService`
3. 无视回退路径（`ignore-capture fallback path`）
   - 受 `shouldRun` 控制
   - Android 11+ 才有真正意义上的 screenshot fallback

关键全局状态：

- Kotlin `common.kt`
  - `SKL`
  - `shouldRun`
  - `gohome`
  - `BIS`
  - `SCREEN_INFO`
- Rust JNI
  - `VIDEO_RAW`
  - `PIXEL_SIZEBack`
  - `PIXEL_SIZEBack8`
  - `force_next`

### 3.4 waiting-for-first-frame 与 Android 重连（PC Waiting State and Reconnect）

关键锚点：

- `flutter/lib/models/model.dart`
  - `waitForFirstImage`
  - `waitForImageTimer`
  - `showConnectedWaitingForImage()`
  - `onEvent2UIRgba()`
- `flutter/lib/common.dart`
  - `showMobileActionsOverlayAboveDialogs()`
  - `removeMobileActionsOverlayEntry()`

源码事实：

- Android 会话连接成功但首帧未到时，PC 会进入 waiting 状态。
- 第一次 fallback 请求会快速发出：
  - 支持 ignore capture 时走 ignore fallback
  - 否则请求 video refresh
- 收到**任何真实 RGBA 帧**时，`onEvent2UIRgba()` 会清除等待状态。
- Android 控制按钮会被提升到 waiting dialog 上层，防止被遮挡。

### 3.5 登录 / 到期 / UUID 绑定（Product Login vs Rust Login）

源码事实：

- `src/common.rs::verify_login()` 当前基本等价于绕过
- 真正的产品账号校验在 Flutter：
  - `flutter/lib/models/user_model.dart`
  - `ChinaNetworkTimeService`
  - `validateUser()`
  - `bind.mainGetUuid()`

真实校验内容：

- 到期时间
- 日期格式
- UUID 绑定
- NTP / HTTP 回退的网络时间

结论：

- 不要把 Rust `verify_login()` 当成当前产品登录的真实准入逻辑。

### 3.6 OIDC / 下载 / 上传 / 同步（HBBS HTTP Flow）

#### 账号 / OIDC

- `src/hbbs_http/account.rs`
- FFI 暴露：
  - `src/flutter_ffi.rs::main_account_auth`
  - `src/flutter_ffi.rs::main_account_auth_cancel`
- UI 接口：
  - `src/ui_interface.rs::account_auth`
  - `src/ui_interface.rs::account_auth_cancel`

#### 下载器

- `src/hbbs_http/downloader.rs`
- FFI 轮询下载状态：
  - `src/flutter_ffi.rs`
- 插件管理器也使用 HTTP 下载：
  - `src/plugin/manager.rs`

#### 录像上传

- `src/hbbs_http/record_upload.rs`
- 由 `src/server/video_service.rs` 触发

#### 同步 / pro 信号

- `src/hbbs_http/sync.rs`
- `src/rendezvous_mediator.rs::start()` 会启动 sync
- `src/server/connection.rs` 会接收来自 `sync::signal_receiver()` 的信号
- `src/ipc.rs` 与 `src/server/connection.rs` 里都有 `is_pro()` 分支

结论：

- `hbbs_http` 是当前工程文档里必须单独记住的一层，不能再缺省。

### 3.7 终端子系统（Terminal Subsystem）

主链路：

- 协议：`libs/hbb_common/protos/message.proto`
- server：`src/server/terminal_service.rs`
- connection route：`src/server/connection.rs`
- Flutter model：`flutter/lib/models/terminal_model.dart`
- Flutter pages：`flutter/lib/desktop/pages/terminal_*`

已核事实：

- 当前 `generate_service_id()` 返回 `ts_<uuid>`
- `terminal.md` 中基于 `tmp_` / `persist_` 的旧叙述已发生漂移
- terminal 持久化概念仍有实现基础，但不能沿用旧文档的完整行为假设

### 3.8 Windows 隐私模式 / 虚拟显示器（Privacy Mode / Virtual Display）

真实入口：

- `src/server/connection.rs`
  - `supported_privacy_mode_impl`
  - `toggle_privacy_mode()`
  - `turn_on_privacy()`
  - `turn_off_privacy()`
- `src/privacy_mode.rs`
- `src/privacy_mode/win_virtual_display.rs`
- `src/virtual_display_manager.rs`
- `flutter/lib/consts.dart`
  - `daxian_virtual_displays`
  - `supported_privacy_mode_impl`

结论：

- 这块应被视为**平台功能主链**，不是附属工具。

---

## 4. Android 运行时核心事实（Android Runtime Facts Summary）

以下内容已经在代码中核验，详细解释见 `docs/ENGINEERING_ANDROID_RUNTIME.md`：

- 服务状态 != 帧源状态 != PC waiting 状态
- `killMediaProjection()` / projection stop 不会等同于整个服务终止
- `restoreMediaProjection()` 在恢复成功前不会先撤掉 ignore fallback
- Android 10 不会假装支持 screenshot fallback
- `FrameRaw.force_next` 用于避免恢复后的第一帧因重复而被吞掉
- `PIXEL_SIZEBack8 = 255` 表示阻断 ignore frame；`0` 表示允许通过
- `VIDEO_RAW` 必须在 fallback 期保持正确启用

---

## 5. 构建与产物现实（Build and Artifact Reality）

### 5.1 Android

关键文件：

- `build.sh`
- `env.sh`
- `flutter/build_android.sh`
- `flutter/build_android_deps.sh`
- `flutter/android/app/build.gradle`

已核事实：

- `build.sh` 从 `target/<triple>/release/liblibrustdesk.so` 复制到：
  - `flutter/android/app/src/main/jniLibs/<abi>/libdaxian.so`
- Kotlin 端 `System.loadLibrary("daxian")`
- Dart Android 端 `DynamicLibrary.open('libdaxian.so')`

### 5.2 Windows

关键文件：

- `flutter/windows/runner/main.cpp`
- `flutter/lib/models/native_model.dart`

已核事实：

- Windows 仍加载 `librustdesk.dll`
- 因此品牌变更仍存在 Windows 侧残留

### 5.3 其他构建脚本 / 包装层

仓库还包含：

- `build.py`
- `build.rs`
- `Dockerfile`
- `appimage/`
- `flatpak/`
- `fastlane/`
- `res/`

结论：

- 任何“改名字 / 改包名 / 改构建产物 / 改入口”类任务，都不能只改一处。

---

## 6. 已确认的文档漂移（Confirmed Doc Drift）

### 6.1 `terminal.md`

- 旧文档描述 `service_id` 为 `tmp_` / `persist_`
- 当前 `src/server/terminal_service.rs` 使用 `ts_<uuid>`
- 旧文档中的持久化叙述不能直接当现行事实

### 6.2 `CLAUDE.md`

- deep link 写成单一 `daxian://`
- 当前源码实际是 manifest scheme 与 Rust `get_uri_prefix()` 并存且不完全一致
- 因此 `CLAUDE.md` 只能当导航，不是最终真相层

### 6.3 对 Android JNI 的旧误解

- `ffi.rs` 不能再被描述成 `pkg2230.rs` 的“完全副本”
- 当前主导出模块是 `pkg2230.rs`

---

## 7. 当前风险与注意事项（Current Risks）

- `APP_NAME` 与 Android manifest scheme 存在 URI prefix 不一致风险
- `ORG` 仍为 `com.carriez`
- Windows DLL 命名仍残留 `rustdesk`
- `pkg2230.rs` 中存在 `static mut` 像素与 JNI 全局状态
- Android runtime 逻辑高度耦合，修改时极易破坏 fallback / waiting / keep-alive
- `src/ui/` 旧路径与 Flutter 新路径并存，桌面入口任务若只看 Flutter 容易漏逻辑
- `hbbs_http` 若不纳入心智模型，账号 / 下载 / pro / 上传相关任务会出现记忆断层

---

## 8. 后续文档维护标准（How to Keep This Baseline Healthy）

修改后如果以下任一项变化，必须同步本文件：

- 产品身份 / 包名 / app name / artifact name
- 顶层模块图
- 关键运行时链路
- 账号 / 同步 / 下载 / 插件 / 终端 / 隐私模式的真实入口
- 已确认的文档漂移结论

更新时继续使用：

- 中文解释
- English symbol / path 原文
- 明确的代码锚点
- 避免与现有文档结构竞争的新记忆文件
