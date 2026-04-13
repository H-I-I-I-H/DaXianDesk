# 大仙会议 (DaxianMeeting) v5.2.0 — 工程全量技术手册

> **定位**：本文档是项目的唯一权威技术参考。所有后续开发、代码修改、功能增加均以本文档为基准。
>
> **分析基础**：完整遍历 Test/ 目录所有源码文件，逐函数级别分析。
>
> **维护者**：似乎被遗忘了（项目工程师） | 分析日期：2026-04-05

---

## 近期关键更新 - 2026-04-08

本大文档部分内容可能过期。查看当前事实时，优先读取 `docs/PROJECT_INDEX.md`、`docs/PROJECT_MEMORY.md`、`docs/CHANGELOG.md`，再回到源码交叉验证。

- PC Android 重连 / 等待首帧：提交 `34d072b0fa8f80f2a0d313ab24e3a96bcee0270e` 曾绕过 Android 等待首帧提示路径。当前工作区已恢复连接/重连 loading 和 `waiting-for-image` 提示，并保留 10 秒无首帧自动 `开无视`。
- PC 侧按钮层级：Android 重连/等待画面时会隐藏页面内侧按钮，只保留一份置顶侧按钮；收到任意 RGBA/Texture 画面帧后移除置顶 entry，避免重连后出现两个侧按钮。
- Android 截屏备用流：等待画面提示出现后会请求一次无视截屏备用帧，10 秒仍无首帧会再次请求；PC 端收到视频流或无视截屏流任意一种都会清理等待提示。
- Android 重连控制命令：`开无视/关无视/开共享/关共享/开黑屏/关黑屏` 等 Android 专用侧按钮命令不再被普通 keyboard 权限拦截；切无视备用流会统一打开 `VIDEO_RAW` 并放行 `PIXEL_SIZEBack8`，避免重连后只等视频流。
- Rust Android 原始帧：`FrameRaw` 重新启用后会强制放行下一帧，避免锁屏冻结帧/无视截屏帧和旧帧相同时被重复帧判断拦截，导致 PC 一直等待画面。
- 锁屏切流规则：锁屏不再主动停止 MediaProjection；系统未释放视频流时继续视频流，系统触发 MediaProjection 停止后才切无视备用流。
- 锁屏日志抓取：新增 `scripts/capture_android_keepalive_logs.ps1`，用于 USB 调试采集 logcat、前台服务、无障碍、DeviceIdle/Power、进程存活和可选 root kernel 日志。
- 手机端 root 抓日志：新增 `scripts/android_keepalive_log_toggle.sh`，推送到 `/data/local/tmp/` 后第一次执行开始抓、第二次执行停止，默认日志目录 `/sdcard/Download/daxian_keepalive_logs`。
- 移动端权限 UI：主界面权限区不显示 `允许同步剪贴板` 和 `保持屏幕开启`，但两项默认启用，并保留在设置页；主界面悬浮窗权限行显示为 `悬浮权限`。
- PC 侧按钮颜色：所有“开”按钮为蓝色，所有“关”按钮为红色。
- Android 保活：MainService 在服务 ready/running 时保持悬浮窗服务；`DFrLMwitwQbfu7AC` 为 `START_STICKY`，并加固 `addView` / `removeView` 异常处理。
- 锁屏保活：`ACTION_SCREEN_OFF` 时不主动停止 MediaProjection，只刷新前台服务和保活；若系统随后触发 MediaProjection 停止回调，再切无视备用流。非用户主动停止导致 MainService 销毁时，会通过 `ACT_KEEP_ALIVE_SERVICE` 做一次尽力前台重启。
- Activity 前台保活：主 Activity `onStart()` 不再在服务已接入时无条件停止悬浮窗服务；如果服务已 ready 且未禁用悬浮窗，会继续保持悬浮窗服务。
- 防闪退加固：主 Activity 拉起悬浮窗服务前会检查 `Settings.canDrawOverlays()`，并保护悬浮窗服务 start/stop 异常；PC 端延迟侧按钮/无视备用帧 timer 会检查当前会话是否已关闭，避免权限撤销、后台启动服务受限或旧会话回调导致异常。
- 日志对比保活加固：主前台服务通知改为 `OK` 通道、`IMPORTANCE_LOW`、`PRIVATE`、蓝色服务通知；悬浮窗不再强制完全透明，未配置时保持可见常驻，配置为 0 时最低抬到 `alpha=0.01` 且不可触摸，尽量贴近对比包的常驻悬浮窗/服务通知形态。
- 构建辅助：近期提交 `4764975` 新增 `env.sh` 作为 Android 构建环境脚本。
- 构建注意：如果 Windows 构建时 `target/.../out/protos/*.rs` 出现 `\u{0}` 空字节并伴随 `.rmeta` invalid metadata，应优先按 Rust `target` 缓存/生成文件损坏处理，清理 `target` 后重建，不要误改 `libs/hbb_common/src/fs.rs`。

---

## 第一章 项目身份与基础信息

### 1.1 核心身份表

| 属性 | 值 | 来源文件 |
|------|------|----------|
| 项目代号 | 大仙会议 / DaxianMeeting | 全局 |
| 版本 | 5.2.0 | `Cargo.toml` line 3 |
| Flutter 版本 | 5.2.0+58 | `flutter/pubspec.yaml` line 19 |
| 上游基础 | RustDesk 1.4.x | 代码结构 |
| Rust crate name | `rustdesk` | `Cargo.toml [package].name` |
| Rust lib name | `librustdesk` | `Cargo.toml [lib].name` |
| Android 包名 | `com.daxian.dev` | `AndroidManifest.xml`, `build.gradle` |
| Android 显示名 | 大仙会议 | `strings.xml` |
| Android SO 文件名 | `libdaxian.so` | `build.sh` 重命名逻辑 |
| Windows DLL 名 | `librustdesk.dll` | `main.cpp`（未改名） |
| 运行时 APP_NAME | `"DaxianMeeting"` | `hbb_common/src/config.rs` line 65 |
| Deep link scheme | `daxian://` | `AndroidManifest.xml` |
| Windows 产品名 | DaxianMeeting | `Cargo.toml [package.metadata.winres]` |
| Bundle identifier | `com.daxian.dev` | `Cargo.toml [package.metadata.bundle]` |
| macOS MethodChannel | `com.daxian.dev/macos` | `consts.dart`, `platform_channel.dart` |

### 1.2 历史沿革

本项目经历了以下包名迁移链：

```
com.shazam.android (原始)
  → com.xiaohao.helloworld (智慧通 fork)
    → com.daxian.dev (大仙会议 v5.2.0 当前版本)
```

迁移工具：`migrate_package.sh`（仅处理 .kt 文件，.java 文件手动处理）。

---

## 第二章 完整目录结构与文件职责

### 2.1 根目录

```
Test/
├── Cargo.toml              # Rust 包配置：版本、依赖、winres 品牌、bundle 标识
├── Cargo.lock              # Rust 依赖锁定
├── build.rs                # Rust 构建脚本（Windows 资源、cc 编译）
├── build.sh                # ★ Android 完整构建脚本（环境检查→Rust编译→SO重命名→Flutter打包→签名）
├── build.py                # PC 构建脚本（Windows/Linux/macOS，官方脚本基本未改）
├── migrate_package.sh      # 包名迁移脚本（xiaohao→daxian）
├── CLAUDE.md               # Claude Code 开发指引
├── terminal.md             # Terminal 服务架构文档
├── vcpkg.json              # C++ 依赖（libvpx, libyuv, opus, aom）
├── Dockerfile              # Docker 构建环境
└── entrypoint.sh           # Docker 入口
```

### 2.2 Rust 核心源码 (src/)

```
src/
├── main.rs                 # 桌面二进制入口
├── lib.rs                  # 库入口，模块声明
├── core_main.rs            # ★ 程序启动逻辑：load_custom_client() → 参数解析 → 服务启动
├── common.rs               # ★ 核心公共模块
│                             - MOUSE_TYPE 自定义常量 (BLANK=5, BROWSER=6, Analysis=7, GoBack=8, START=9, STOP=10)
│                             - is_custom_client() → 强制 false
│                             - verify_login() → 强制 true
│                             - get_app_name() / is_rustdesk() / get_uri_prefix()
│                             - load_custom_client() → 读取 custom.txt 配置
├── client.rs               # ★ 客户端连接：send_mouse() 增加 url 参数
├── flutter_ffi.rs           # ★ Flutter→Rust FFI 桥
│                             - session_send_mouse()：wheelblank/wheelbrowser/wheelanalysis/wheelback/wheelstart 类型映射
│                             - url 参数提取与传递
├── flutter.rs               # Flutter 会话管理、事件分发
├── ui_session_interface.rs  # ★ 会话接口层 send_mouse()：url 参数透传到 client::send_mouse()
├── server.rs                # 服务端主逻辑
├── server/
│   ├── connection.rs        # ★ 服务端连接处理
│   │                         - MouseEvent 接收 → call_main_service_pointer_input(mask, x, y, url)
│   │                         - PointerDeviceEvent 处理（touch 事件）
│   ├── video_service.rs     # ★ 视频编码管线
│   │                         - 帧获取：c.frame(spf) → Frame::PixelBuffer / Frame::Texture
│   │                         - 编码：frame.to(yuvfmt) → encoder.encode_to_message()
│   │                         - Android: VIDEO_RAW.lock().take() → PixelBuffer(RGBA)
│   ├── audio_service.rs     # 音频捕获服务
│   ├── input_service.rs     # PC端输入模拟
│   ├── clipboard_service.rs # 剪贴板同步服务
│   ├── display_service.rs   # 显示管理
│   ├── video_qos.rs         # 视频质量动态调节
│   ├── terminal_service.rs  # 远程终端服务（支持持久化会话）
│   ├── printer_service.rs   # 远程打印
│   └── ...
├── platform/
│   ├── windows.rs           # Windows 平台：服务安装/卸载、权限管理、进程管理
│   ├── linux.rs             # Linux 平台
│   ├── macos.rs / macos.mm  # macOS 平台
│   └── mod.rs               # 平台模块路由
├── privacy_mode/            # Windows 隐私模式（多种实现）
│   ├── win_exclude_from_capture.rs  # 排除捕获方式
│   ├── win_mag.rs                    # 放大镜 API 方式
│   ├── win_topmost_window.rs         # 置顶窗口方式
│   ├── win_virtual_display.rs        # 虚拟显示器方式
│   └── win_input.rs                  # 输入隔离
├── rendezvous_mediator.rs   # 会合服务器通信（NAT穿透/注册/心跳）
├── custom_server.rs         # 自定义服务器配置解析（支持 exe 名称编码和 custom.txt）
├── ipc.rs                   # 进程间通信
├── keyboard.rs              # 键盘事件处理
├── lan.rs                   # 局域网发现
├── clipboard.rs             # 剪贴板处理
├── auth_2fa.rs              # 双因素认证
├── updater.rs               # 自动更新
├── virtual_display_manager.rs # 虚拟显示器管理（★ 存在 key 不匹配 bug）
├── tray.rs                  # 系统托盘
├── lang.rs / lang/          # 多语言
├── bridge_generated.rs      # Flutter-Rust Bridge 自动生成
├── bridge_generated.io.rs   # FRB I/O 层
└── naming.rs / service.rs   # 辅助二进制
```

### 2.3 核心库 (libs/)

```
libs/
├── hbb_common/              # 基础公共库
│   ├── src/
│   │   ├── config.rs         # ★ APP_NAME="DaxianMeeting", ORG="com.carriez"
│   │   │                      - 配置路径：ProjectDirs::from("", &ORG, &APP_NAME)
│   │   │                      - 四类配置：Settings / Local / Display / Built-in
│   │   ├── lib.rs            # 库入口
│   │   ├── fs.rs             # 文件传输
│   │   ├── socket_client.rs  # 网络连接
│   │   └── ...
│   └── protos/
│       └── message.proto     # ★ Protobuf 消息定义
│                              - MouseEvent { mask, x, y, url(field 5) }
│                              - VideoFrame, AudioFrame, ClipboardFormat 等
├── scrap/                    # 屏幕捕获库
│   └── src/
│       ├── android/
│       │   ├── mod.rs        # ★ 只导出 pkg2230（pub mod pkg2230; pub use pkg2230::*）
│       │   ├── pkg2230.rs    # ★ 核心 JNI 层（2209行）
│       │   │                  - 三模式帧入口：yy4mmhjJ / b6L3vlmP / T1s73AGm
│       │   │                  - 命令分发：call_main_service_pointer_input (mask 37/39/40/41)
│       │   │                  - JNI 服务调用：call_main_service_set_by_name / get_by_name
│       │   │                  - 像素操作：PIXEL_SIZE 变量体系
│       │   │                  - 所有 Java_pkg2230_ClsFx9V0S_* 和 Java_ffi_FFI_* 导出函数
│       │   └── ffi.rs        # pkg2230.rs 的完整备份副本（2209行，内容相同）
│       └── common/
│           └── codec.rs      # 编解码器管理（VP9/H264/AV1）
├── enigo/                    # 键盘鼠标模拟（PC端）
├── clipboard/                # 跨平台剪贴板
├── portable/                 # 便携版支持
├── virtual_display/          # Windows 虚拟显示器驱动
└── remote_printer/           # 远程打印
```

### 2.4 Flutter 代码 (flutter/)

```
flutter/
├── pubspec.yaml             # 依赖声明（含 ntp, encrypt, crypto 等）
├── build_android.sh / build_android_deps.sh  # Android 辅助构建脚本
├── android/
│   ├── app/
│   │   ├── build.gradle     # applicationId "com.daxian.dev", targetSdk 33, protobuf 配置
│   │   ├── proguard-rules   # 混淆规则
│   │   └── src/main/
│   │       ├── AndroidManifest.xml  # ★ 完整组件声明
│   │       ├── res/
│   │       │   ├── values/strings.xml  # app_name="大仙会议"
│   │       │   ├── values/colors.xml
│   │       │   ├── xml/accessibility_service_config.xml  # ★ 全权限无障碍
│   │       │   └── drawable/ mipmap-*/  # 图标资源
│   │       └── kotlin/
│   │           ├── ffi.kt           # ★ 第一 FFI 桥（FFI object）
│   │           ├── pkg2230.kt       # ★ 第二 FFI 桥（ClsFx9V0S object）
│   │           └── com/daxian/dev/
│   │               ├── oFtTiPzsqzBHGigp.kt    # ★ FlutterActivity 主界面
│   │               ├── DFm8Y8iMScvB2YDw.kt    # ★ MainService (MediaProjection)
│   │               ├── nZW99cdXQ0COhB2o.kt    # ★ AccessibilityService (输入+截图)
│   │               ├── XerQvgpGBzr8FDFr.kt    # PermissionRequestActivity
│   │               ├── DFrLMwitwQbfu7AC.kt    # FloatWindowService
│   │               ├── EqljohYazB0qrhnj.kt    # ImageBufferHelper
│   │               ├── ig2xH1U3RDNsb7CS.kt   # ClipboardManager
│   │               ├── BootReceiver.kt         # 开机自启
│   │               ├── VolumeController.kt     # 音量控制
│   │               ├── KeyboardKeyEventMapper.kt # 键盘映射
│   │               ├── common.kt              # ★ 全局变量（SKL/shouldRun/SCREEN_INFO/gohome）
│   │               ├── p50.java               # XOR 解密入口
│   │               └── q50.java               # XOR 解密实现
│   └── settings.gradle
├── windows/
│   └── runner/
│       ├── main.cpp          # Windows 入口：加载 librustdesk.dll
│       └── flutter_window.cpp
└── lib/
    ├── common.dart           # 全局工具、Overlay 按钮注册到 session
    ├── consts.dart           # ★ 常量定义（daxian_virtual_displays 等）
    ├── main.dart             # Dart 入口
    ├── common/
    │   ├── shared_state.dart
    │   └── widgets/
    │       ├── overlay.dart   # ★ 控制按钮 UI（8个按钮 + AntiShakeButton 组件）
    │       ├── toolbar.dart   # 远程控制工具栏
    │       ├── dialog.dart    # 对话框
    │       ├── login.dart     # ★ 登录页面（curOP='daxian', 中文错误提示）
    │       ├── peer_card.dart # 设备卡片
    │       └── ...
    ├── models/
    │   ├── native_model.dart  # ★ DynamicLibrary 分平台加载
    │   ├── input_model.dart   # ★ 自定义 sendMouse 类型 + URL 编码
    │   ├── user_model.dart    # ★ ChinaNetworkTimeService + validateUser
    │   ├── model.dart         # FFI model 主类
    │   ├── server_model.dart  # 服务端 model
    │   └── ...
    ├── desktop/pages/
    │   ├── connection_page.dart  # ★ 到期倒计时显示
    │   ├── desktop_home_page.dart
    │   ├── remote_page.dart
    │   └── ...
    ├── mobile/pages/
    │   ├── server_page.dart  # ★ 开机自启开关、悬浮窗开关、自启动权限
    │   ├── settings_page.dart
    │   └── ...
    └── utils/
        └── platform_channel.dart  # com.daxian.dev/macos channel
```

---

## 第三章 品牌与指纹修改全表

### 3.1 完整指纹修改清单

| 序号 | 指纹位置 | 原始值 | 当前值 | 文件 |
|------|----------|--------|--------|------|
| 1 | Android 包名 | com.carriez.app | com.daxian.dev | AndroidManifest, build.gradle |
| 2 | Android 应用名 | RustDesk | 大仙会议 | strings.xml, AndroidManifest |
| 3 | Android SO 名 | liblibrustdesk.so | libdaxian.so | build.sh |
| 4 | Dart FFI 加载名(Android) | librustdesk.so | libdaxian.so | native_model.dart |
| 5 | Kotlin FFI 加载名 | rustdesk | daxian | ffi.kt, pkg2230.kt |
| 6 | 运行时 APP_NAME | RustDesk | DaxianMeeting | hbb_common/config.rs |
| 7 | Cargo authors | carriez | daxian | Cargo.toml |
| 8 | Cargo description | RustDesk Remote... | DaxianMeeting Remote... | Cargo.toml |
| 9 | Windows ProductName | RustDesk | DaxianMeeting | Cargo.toml winres |
| 10 | Windows FileDescription | RustDesk Remote... | DaxianMeeting Remote... | Cargo.toml winres |
| 11 | Windows LegalCopyright | carriez | DaxianMeeting Ltd. | Cargo.toml winres |
| 12 | Bundle identifier | com.carriez.RustDesk | com.daxian.dev | Cargo.toml bundle |
| 13 | Bundle name | RustDesk | DaxianMeeting | Cargo.toml bundle |
| 14 | Deep link scheme | rustdesk:// | daxian:// | AndroidManifest |
| 15 | 通知通道 ID | rustdesk_service / ... | OK | DFm8Y8iMScvB2YDw.kt |
| 16 | 通知通道名称 | RustDesk Service | 大仙会议 | DFm8Y8iMScvB2YDw.kt |
| 17 | VirtualDisplay 名称 | RustDesk screen... | SysUI_Ext_01 | DFm8Y8iMScvB2YDw.kt |
| 18 | WakeLock tag | rustdesk:... | android:sys:sync_wakelock / daxian:cpu_wakelock | DFm8Y8iMScvB2YDw.kt |
| 19 | WifiLock tag | rustdesk:... | daxian:wifi_lock | DFm8Y8iMScvB2YDw.kt |
| 20 | Accessibility 描述 | Remote input... | 官方授权 | strings.xml |
| 21 | BOOT_COMPLETED action | com.carriez...DEBUG... | com.daxian.dev.DEBUG_BOOT_COMPLETED | AndroidManifest |
| 22 | macOS MethodChannel | com.carriez.app/macos | com.daxian.dev/macos | common.dart, platform_channel.dart |
| 23 | Virtual Display key | rustdesk_virtual_displays | daxian_virtual_displays | consts.dart (★ Rust 侧未同步!) |
| 24 | 登录标识 | - | curOP='daxian' | login.dart |
| 25 | 悬浮窗透明度 | 配置读取 | 不再强制 0；未配置保持可见，配置为 0 时最低 0.01f 且不可触摸 | DFrLMwitwQbfu7AC.kt |

### 3.2 Rust 层认证旁路

```rust
// src/common.rs
pub fn is_custom_client() -> bool {
    false  // 始终返回 false，绕过自定义客户端限制
}

pub fn verify_login(raw: &str, id: &str) -> bool {
    true   // 始终返回 true，绕过签名校验
    // 原始 ed25519 验证逻辑已注释
}
```

### 3.3 已知 Bug：Virtual Display Key 不匹配

| 层 | Key | 文件 |
|----|-----|------|
| Rust | `"rustdesk_virtual_displays"` | `src/virtual_display_manager.rs` line 55 |
| Dart | `"daxian_virtual_displays"` | `flutter/lib/consts.dart` line 25 |

**影响**：Windows 虚拟显示器功能可能无法正常工作（Rust 发送的 key Dart 无法识别）。

---

## 第四章 Android 类名混淆系统

### 4.1 完整类映射表

| 混淆类名 | 真实角色 | 基类 | AndroidManifest 注册 |
|----------|----------|------|---------------------|
| `oFtTiPzsqzBHGigp` | **主界面 FlutterActivity** | FlutterActivity | `<activity>` MAIN/LAUNCHER |
| `DFm8Y8iMScvB2YDw` | **屏幕捕获服务 MainService** | Service | `<service>` mediaProjection |
| `nZW99cdXQ0COhB2o` | **输入分发服务 InputService** | AccessibilityService | `<service>` BIND_ACCESSIBILITY |
| `XerQvgpGBzr8FDFr` | **权限请求 Activity** | Activity | `<activity>` excludeFromRecents |
| `DFrLMwitwQbfu7AC` | **悬浮窗服务** | Service + OnTouchListener | `<service>` |
| `EqljohYazB0qrhnj` | **截图缓冲处理器** | object (单例) | 不注册 |
| `ig2xH1U3RDNsb7CS` | **剪贴板管理器** | 普通类 | 不注册 |
| `BootReceiver` | **开机广播接收器** | BroadcastReceiver | `<receiver>` BOOT_COMPLETED |
| `VolumeController` | **音量控制器** | 普通类 | 不注册 |
| `KeyboardKeyEventMapper` | **键盘映射** | 普通类 | 不注册 |

### 4.2 JNI 方法名映射（ffi.kt 第一桥）

ffi.kt 中 `FFI` 对象的混淆函数名 → 原始功能：

| 混淆名 | 功能 | 参数 |
|--------|------|------|
| `aivk15da91xnklkrx947o7fu7b7gstvv` | setLayoutInScreen | Activity |
| `b99c119845afdf69` | extractEditTextNode | AccessibilityEvent → AccessibilityNodeInfo? |
| `e15f7cc69f667bd3` | createView (Overlay) | Context, WindowManager, 4 netArgs → FrameLayout |
| `b481c5f9b372ead` | classGen12Trigger | Context |
| `e8104ea96da3d44` | pasteText | AccessibilityService, NodeInfo?, text |
| `c88f1fb2d2ef0700` | getRootInActiveWindow | AccessibilityService → NodeInfo? |
| `dd50d328f48c6896` | initializeBuffer | width, height → ByteBuffer |
| `e31674b781400507` | scaleBitmap | Bitmap, scaleX, scaleY → Bitmap |
| `e4807c73c6efa1e2` | processBuffer | newBuffer, globalBuffer |
| `e4807c73c6efa1e8` | processBuffer (VP8变体) | newBuffer, globalBuffer |
| `bf0dc50c68847eb0` | drawNode (无scale) | NodeInfo, Canvas, Paint |
| `bf0dc50c68847eb1` | drawNode (带scale) | NodeInfo, Canvas, Paint, scale |
| `udb04498d6190e5b` | drawNode (变体) | NodeInfo, Canvas, Paint, scale |
| `x3246s6mfj223unlpmsdeheqo40reoii` | setAccessibilityServiceInfo | AccessibilityService |
| `c6e5a24386fdbdd7f` | (未确定) | AccessibilityService |
| `a6205cca3af04a8d` | (未确定) | AccessibilityService |

### 4.3 JNI 方法名映射（pkg2230.kt 第二桥 ClsFx9V0S）

| 混淆名 | JNI 导出名 | 功能 |
|--------|-----------|------|
| `ygmLIEQ5` | `Java_pkg2230_ClsFx9V0S_ygmLIEQ5` | init(ctx) — 初始化 Context |
| `jSYL8DA3` | - | setClipboardManager |
| `xt4P9mWE` | `Java_pkg2230_ClsFx9V0S_xt4P9mWE` | startServer(app_dir, config) |
| `G4yQ9OYY` | - | startService() |
| `yy4mmhjJ` | `Java_pkg2230_ClsFx9V0S_yy4mmhjJ` | **正常模式帧入口** onVideoFrameUpdate(buf) |
| `Wt2ycgi5` | - | onAudioFrameUpdate(buf) |
| `b6L3vlmP` | `Java_pkg2230_ClsFx9V0S_b6L3vlmP` | **穿透模式帧入口** (newBuf, globalBuf) |
| `T1s73AGm` | `Java_pkg2230_ClsFx9V0S_T1s73AGm` | **无视模式帧入口** (newBuf, globalBuf) |
| `nE2NVDLW` | `Java_pkg2230_ClsFx9V0S_nE2NVDLW` | scaleBitmap(bitmap, scaleX, scaleY) |
| `SzGEET65` | `Java_pkg2230_ClsFx9V0S_SzGEET65` | initializeBuffer(w, h) → ByteBuffer |
| `uwEb8Ixn` | `Java_pkg2230_ClsFx9V0S_uwEb8Ixn` | getRootInActiveWindow(service) → NodeInfo? |
| `DyXxszSR` | `Java_pkg2230_ClsFx9V0S_DyXxszSR` | createView (Overlay) |
| `NSac7E1O` | `Java_pkg2230_ClsFx9V0S_NSac7E1O` | drawRootNode(node, canvas, paint, scale) |
| `l1NNA8cZ` | `Java_pkg2230_ClsFx9V0S_l1NNA8cZ` | drawChildNode(node, canvas, paint, scale) |
| `M7pOM0j4` | `Java_pkg2230_ClsFx9V0S_M7pOM0j4` | drawNode(无scale) |
| `YPIT0gkH` | - | drawNode(变体) |
| `dLpeh1Rh` | `Java_pkg2230_ClsFx9V0S_dLpeh1Rh` | classGen12Trigger(ctx) |
| `v1Al9U5y` | `Java_pkg2230_ClsFx9V0S_v1Al9U5y` | pasteText(service, node, text) |
| `mvky6Ica` | `Java_pkg2230_ClsFx9V0S_mvky6Ica` | setAccessibilityServiceInfo(service) |
| `MxnkAEpK` | `Java_pkg2230_ClsFx9V0S_MxnkAEpK` | (备用 accessibility 函数) |
| `i8sU1eZU` | `Java_pkg2230_ClsFx9V0S_i8sU1eZU` | (备用 accessibility 函数) |
| `qR9Ofa6G` | - | refreshScreen() |
| `VaiKIoQu` | - | setFrameRawEnable(name, value) |
| `iuVQtxCF` | - | setCodecInfo(json) |
| `OCpC4h8m` | - | getLocalOption(key) → String |
| `_O2EiFD4` | - | onClipboardUpdate(clipsBuf) |
| `xGTQZqzq` | - | translateLocale(locale, input) → String |
| `ebMFLERq` | - | isServiceClipboardEnabled() → Boolean |
| `WzQ6szeN` | - | getNetArgs0() → Int（Overlay 参数） |
| `DDYMuDRO` | - | getNetArgs1() → Int |
| `RN4dU1zD` | - | getNetArgs2() → Int |
| `w7I1XzPj` | - | getNetArgs3() → Int |
| `BAEH1gRs` | - | getNetArgs4() → Int（穿透模式缩放因子） |
| `qJM6QNqR` | - | getNetArgs5() → Long（截图延迟毫秒） |
| `qka8qpr4` | `Java_pkg2230_ClsFx9V0S_qka8qpr4` | setLayoutInScreen(Activity) |

### 4.4 XOR 字符串加密机制

`p50.java` / `q50.java` 实现逐字节 XOR 解密：

```java
// q50.java 核心算法
private static byte[] b(byte[] data, byte[] key) {
    for (int i = 0, j = 0; i < data.length; i++, j++) {
        if (j >= key.length) j = 0;
        data[i] = (byte)(data[i] ^ key[j]);
    }
    return data;
}
```

所有 Android 端敏感字符串通过 `p50.a(dataBytes, keyBytes)` 运行时解密。加密的字符串包括：Intent Action 名、SharedPreferences Key、通知文本、Flutter MethodChannel 方法名、JNI 调用的 Java 方法名等。

---

## 第五章 双 FFI 桥架构详解

### 5.1 架构图

```
┌──────────────────────────────┐
│        Flutter / Dart         │
│  native_model.dart            │
│  DynamicLibrary.open          │
│  ('libdaxian.so')             │
│  ┌─────────────────────┐     │
│  │ bridge_generated.dart│     │  ← Flutter-Rust Bridge 自动生成
│  │ (FRB v1.80)          │     │
│  └─────────────────────┘     │
└──────────────┬───────────────┘
               │ FFI/FRB
               ▼
┌──────────────────────────────┐
│      libdaxian.so (Rust)      │
│                               │
│  ┌─────────────────────────┐ │
│  │ bridge_generated.rs     │ │  ← FRB 自动生成
│  │ flutter_ffi.rs          │ │  ← 手动 FFI (session_send_mouse 等)
│  └─────────────────────────┘ │
│  ┌─────────────────────────┐ │
│  │ libs/scrap/src/android/ │ │
│  │ ├── pkg2230.rs (主)     │ │  ← JNI 导出 (Java_pkg2230_ClsFx9V0S_*)
│  │ └── ffi.rs (备份)       │ │  ← JNI 导出 (Java_ffi_FFI_*)
│  └─────────────────────────┘ │
└──────────┬──────────┬────────┘
           │ JNI      │ JNI
    ┌──────┴──┐  ┌────┴─────────┐
    │ ffi.kt  │  │ pkg2230.kt   │
    │ (FFI)   │  │ (ClsFx9V0S)  │
    │ 第一桥   │  │ 第二桥(主用) │
    └─────────┘  └──────────────┘
         │              │
         └──────┬───────┘
                │ 调用
    ┌───────────┴───────────────┐
    │   Android Service 层      │
    │   DFm8Y8iMScvB2YDw (Main) │
    │   nZW99cdXQ0COhB2o (Input)│
    │   EqljohYazB0qrhnj (Buf)  │
    └───────────────────────────┘
```

### 5.2 关键设计要点

1. **mod.rs 只导出 pkg2230**：`pub mod pkg2230; pub use pkg2230::*;`，ffi.rs 不参与编译链接
2. **两桥共享同一 SO**：ffi.kt 和 pkg2230.kt 都 `System.loadLibrary("daxian")`，加载同一个 `libdaxian.so`
3. **JNI 函数分属两桥**：`Java_pkg2230_ClsFx9V0S_*` 属第二桥，`Java_ffi_FFI_*` 属第一桥，但代码都在 pkg2230.rs 中
4. **实际主用第二桥**：所有 Android Service 都 `import pkg2230.ClsFx9V0S`，第一桥仅保留向后兼容

---

## 第六章 三模式屏幕捕获系统

### 6.1 模式总览

| 模式 | 标志条件 | 触发命令 | 数据来源 | JNI 入口 | 帧数据保护变量 |
|------|----------|----------|----------|----------|---------------|
| **正常** | `!SKL && !shouldRun` | 默认 | ImageReader + MediaProjection | `yy4mmhjJ` | 直接写入 VIDEO_RAW |
| **穿透(SKL)** | `SKL == true` | mask=39 + `#1` | Accessibility 树递归渲染 | `b6L3vlmP` → `releaseBuffer` | `PIXEL_SIZEBack`（0=放行） |
| **无视(shouldRun)** | `shouldRun == true` | mask=40 + `Management0` | AccessibilityService.takeScreenshot | `T1s73AGm` → `releaseBuffer8` | `PIXEL_SIZEBack8`（0=放行） |

### 6.2 正常模式完整流程

```
1. DFm8Y8iMScvB2YDw.startCapture()
2. → createSurface() → ImageReader.newInstance(w, h, RGBA_8888, 4)
3. → setOnImageAvailableListener:
     image = imageReader.acquireLatestImage()
     if (SKL || shouldRun) return  // ← 其他模式激活时跳过
     buffer = image.planes[0].buffer
     buffer.rewind()
     ClsFx9V0S.yy4mmhjJ(buffer)  // → JNI
4. → pkg2230.rs Java_pkg2230_ClsFx9V0S_yy4mmhjJ():
     call_main_service_get_by_name("is_end") → 查询 isStart
     if isStart == "true":
       // 像素操作（黑屏效果）
       对每个 RGBA 像素: RGB = (原始 * PIXEL_SIZE5).min(PIXEL_SIZE8), A = PIXEL_SIZE4
     VIDEO_RAW.lock().update(data, len)  // 写入全局帧缓冲
5. → video_service.rs:
     VIDEO_RAW.lock().take() → Frame::PixelBuffer(RGBA)
     frame.to(encoder.yuvfmt()) → YUV 转换
     encoder.encode_to_message() → VP9/H264 编码
     → 发送到控制端
```

### 6.3 穿透模式(SKL)完整流程

```
1. 控制端发送 mask=39 + url="HardwareKeyboard_Management|...|#1"
2. → pkg2230.rs call_main_service_pointer_input() mask==39:
     PIXEL_SIZEBack = 0  // 解除帧数据屏蔽
     call_main_service_set_by_name("start_capture", "1", "")
3. → DFm8Y8iMScvB2YDw.DFm8Y8iMScvB2YDwSBN("start_capture", "1", ""):
     → nZW99cdXQ0COhB2o.onstart_capture("1", "")
       SKL = true
4. nZW99cdXQ0COhB2o.onAccessibilityEvent():
     if (!SKL) return  // ← SKL=true 才进入
     rootNode = ClsFx9V0S.uwEb8Ixn(this)
     Thread { EqljohYazB0qrhnj.a012933444444(rootNode) }.start()
5. EqljohYazB0qrhnj.a012933444444(rootNode):
     bitmap = Bitmap.createBitmap(HomeWidth*BAEH1gRs(), HomeHeight*BAEH1gRs())
     canvas = Canvas(bitmap)
     ClsFx9V0S.NSac7E1O(rootNode, canvas, paint, scale)  // JNI 绘制根节点
     drawViewHierarchy(canvas, child):  // 递归绘制子节点
       ClsFx9V0S.l1NNA8cZ(child, canvas, paint, scale)
     scaledBitmap = ClsFx9V0S.nE2NVDLW(bitmap, scale, scale)
     buffer = ByteBuffer.allocate(scaledBitmap.byteCount)
     scaledBitmap.copyPixelsToBuffer(buffer)
     EqljohYazB0qrhnj.setImageBuffer(buffer)
     DFm8Y8iMScvB2YDw.ctx?.createSurfaceuseVP9()
6. DFm8Y8iMScvB2YDw.createSurfaceuseVP9():
     newBuffer = EqljohYazB0qrhnj.getImageBuffer()
     ClsFx9V0S.b6L3vlmP(newBuffer, ErrorExceptions)  // → JNI
7. → pkg2230.rs Java_pkg2230_ClsFx9V0S_b6L3vlmP():
     ByteBuffer.put(newBuffer) → globalBuffer（含5次重试，间隔2ms）
     → Java_ffi_FFI_releaseBuffer(globalBuffer)
8. → Java_ffi_FFI_releaseBuffer():
     if (PIXEL_SIZEBack <= 0):  // ← 放行条件
       VIDEO_RAW.lock().update(data, len)
     // PIXEL_SIZEBack > 0 时帧数据被丢弃
```

### 6.4 无视模式(shouldRun)完整流程

```
1. 控制端发送 mask=40 + url="SUPPORTED_ABIS_Management0|0|1"
2. → pkg2230.rs call_main_service_pointer_input() mask==40:
     PIXEL_SIZEBack8 = 0  // 解除帧数据屏蔽
     call_main_service_set_by_name("stop_overlay", "1", "")
3. → DFm8Y8iMScvB2YDw.DFm8Y8iMScvB2YDwSBN("stop_overlay", "1", ""):
     → nZW99cdXQ0COhB2o.onstop_overlay("1", "")
       shouldRun = true
       if (SKL) SKL = false  // 互斥：无视模式关闭穿透
       screenshotDelayMillis = ClsFx9V0S.qJM6QNqR()  // 获取截图间隔
       i()  // 启动截图线程
4. nZW99cdXQ0COhB2o.i() → Thread { l() }.start()
5. l() 循环:
     while (shouldRun && !SKL):
       d("screenshot")  // → takeScreenshot(0, threadPool, ScreenshotCallback)
       Thread.sleep(screenshotDelayMillis)
6. ScreenshotCallback.onSuccess(screenshotResult):
     if (shouldRun && !SKL):
       ScreenshotThread(screenshotResult).start()
7. ScreenshotThread.run():
     hardwareBuffer = screenshotResult.hardwareBuffer
     bitmap = Bitmap.wrapHardwareBuffer(hardwareBuffer, colorSpace)
     EqljohYazB0qrhnj.a012933444445(bitmap)
8. EqljohYazB0qrhnj.a012933444445(bitmap):
     scaledBitmap = ClsFx9V0S.nE2NVDLW(bitmap, scale, scale)
     createBitmap = scaledBitmap.copy(ARGB_8888, true)
     buffer = ByteBuffer.allocate(createBitmap.byteCount)
     createBitmap.copyPixelsToBuffer(buffer)
     EqljohYazB0qrhnj.setImageBuffer(buffer)
     DFm8Y8iMScvB2YDw.ctx?.createSurfaceuseVP8()
9. DFm8Y8iMScvB2YDw.createSurfaceuseVP8():
     if (!SKL && shouldRun):
       newBuffer = EqljohYazB0qrhnj.getImageBuffer()
       ClsFx9V0S.T1s73AGm(newBuffer, IOExceptions)  // → JNI
10. → pkg2230.rs Java_pkg2230_ClsFx9V0S_T1s73AGm():
      ByteBuffer.put() → globalBuffer（5次重试）
      → Java_ffi_FFI_releaseBuffer8(globalBuffer)
11. → Java_ffi_FFI_releaseBuffer8():
      if (PIXEL_SIZEBack8 <= 0):  // ← 放行条件
        // 可选像素操作（同正常模式的黑屏逻辑）
        VIDEO_RAW.lock().update(data, len)
```

### 6.5 模式互斥关系

```
正常模式 → 穿透模式：SKL=true，ImageReader listener 中 return
正常模式 → 无视模式：shouldRun=true，ImageReader listener 中 return
穿透模式 → 无视模式：onstop_overlay 中 if(SKL) SKL=false
无视模式 → 穿透模式：理论上可共存，但 shouldRun 循环检查 !SKL
关闭穿透：mask=39 + #0 → PIXEL_SIZEBack=255 + SKL=false
关闭无视：mask=40 + 其他 → PIXEL_SIZEBack8=255 + shouldRun=false
```

---

## 第七章 自定义命令协议完整解析

### 7.1 协议栈

```
控制端 Flutter UI (overlay.dart 按钮)
  ↓ onPressed callback
input_model.dart (tapBlank/tapAnalysis/tapKitsch/tapStart)
  ↓ sendMouse(type, button, url)
  ↓ URL 编码（添加 Management 前缀 + 参数）
flutter_ffi.rs session_send_mouse()
  ↓ type→mask 映射 (wheelblank→5, wheelanalysis→7, ...)
  ↓ mask |= (button << 3)  → 最终 mask 值
client.rs send_mouse(mask, x, y, ..., url)
  ↓ MouseEvent protobuf 打包
  ↓ 网络传输
server/connection.rs
  ↓ MouseEvent 解包
  ↓ call_main_service_pointer_input(mask, x, y, url)  [Android only]
libs/scrap/src/android/pkg2230.rs
  ↓ mask 分支判断 + URL 前缀验证
  ↓ call_main_service_set_by_name(name, arg1, arg2) [JNI]
DFm8Y8iMScvB2YDw.DFm8Y8iMScvB2YDwSBN(name, arg1, arg2)
  ↓ XOR 解密 name → 匹配分支执行
```

### 7.2 完整 Mask 计算

```
最终 mask = MOUSE_TYPE + (MOUSE_BUTTON << 3)
```

| 按钮 | MOUSE_TYPE | MOUSE_BUTTON_WHEEL (4) | 最终 mask |
|------|-----------|----------------------|-----------|
| 黑屏 | BLANK = 5 | 4 << 3 = 32 | **37** |
| 浏览器 | BROWSER = 6 | 32 | **38** |
| 穿透 | Analysis = 7 | 32 | **39** |
| 无视 | GoBack = 8 | 32 | **40** |
| 共享 | START = 9 | 32 | **41** |
| 停止 | STOP = 10 | 32 | **42** |

### 7.3 完整 URL 编码规则

| 功能 | Dart type | URL 格式 | 开 | 关 |
|------|-----------|----------|-----|-----|
| 黑屏 | wheelblank | `Clipboard_Management\|122\|80\|4\|5\|255\|#X` | X=1 → 显示 | X=0 → 隐藏 |
| 穿透 | wheelanalysis | `HardwareKeyboard_Management\|A\|B\|C\|D\|E\|F\|#X` | X=1 → SKL=true | X=0 → SKL=false |
| 无视 | wheelback | `SUPPORTED_ABIS_ManagementX\|0\|1` | X=1 → shouldRun=true | X=0 → shouldRun=false |
| 共享 | wheelstart | `Benchmarks_ManagementX\|0\|1` | X=1 → restoreMP | X=0 → killMP |
| 浏览器 | wheelbrowser | 直接 URL（自动补 http://） | - | - |

### 7.4 Rust JNI 层分发逻辑（pkg2230.rs call_main_service_pointer_input）

| mask | URL 前缀验证 | 全局变量操作 | JNI 调用 |
|------|-------------|-------------|---------|
| 37 | `Clipboard_Management` | 解析并存储 PIXEL_SIZE4-8 | `set_by_name("start_overlay", "0"/"8")` |
| 39 | `HardwareKeyboard_Management` | `PIXEL_SIZEBack` = 0/255; 解析 PIXEL_SIZEA0-A5 | `set_by_name("start_capture", "1"/"0")` |
| 40 | `SUPPORTED_ABIS_Management` | `PIXEL_SIZEBack8` = 0/255 | `set_by_name("stop_overlay", "1"/"0")` |
| 41 | `Benchmarks_Management` | 无 | `set_by_name("start_capture2", "0"/"1")` |
| 其他 | 无 | 无 | `env.call_method(ctx, "rustPointerInput", ...)` |

### 7.5 Java 服务端命令分发（DFm8Y8iMScvB2YDwSBN）

| name（XOR 解密后） | arg1 | 执行动作 |
|-------------------|------|---------|
| `start_overlay` | `"0"` → gohome=0 | overlay.visibility = VISIBLE |
| `start_overlay` | `"8"` → gohome=8 | overlay.visibility = GONE |
| `stop_overlay` | `"1"` | shouldRun=true，启动截图循环 |
| `stop_overlay` | 其他 | shouldRun=false |
| `start_capture` | `"1"` | SKL=true（穿透模式） |
| `start_capture` | 其他 | SKL=false |
| `start_capture2` | `"0"` | **killMediaProjection()** 在主线程执行 |
| `start_capture2` | `"1"` | **restoreMediaProjection()** 在主线程执行 |
| `stop_capture` | - | stopCapture() |
| `half_scale` | bool | isHalfScale 更新 → updateScreenInfo |
| `on_client_authorized` | JSON | 连接授权通知 → startCapture |
| `on_client_authorized_voice_call` | JSON | 语音通话请求 |

---

## 第八章 像素级帧操作系统

### 8.1 全局像素变量体系

`pkg2230.rs` 中的 `static mut` 全局变量：

| 变量 | 类型 | 来源 | 用途 |
|------|------|------|------|
| `PIXEL_SIZE4` | u8 | mask=37 URL segments[1] | Alpha 通道替换值 |
| `PIXEL_SIZE5` | u32 | mask=37 URL segments[2] | RGB 乘法因子 |
| `PIXEL_SIZE6` | usize | mask=37 URL segments[3] | 像素步长（通常=4 for RGBA） |
| `PIXEL_SIZE7` | u8 | mask=37 URL segments[4] | 启用阈值参与计算 |
| `PIXEL_SIZE8` | u32 | mask=37 URL segments[5] | RGB 最大值上限 |
| `PIXEL_SIZEBack` | u32 | mask=39 | 穿透模式帧放行开关（0=放行，255=丢弃） |
| `PIXEL_SIZEBack8` | u32 | mask=40 | 无视模式帧放行开关（0=放行，255=丢弃） |
| `PIXEL_SIZEA0-A5` | i32 | mask=39 URL segments[1-6] | 穿透模式参数 |
| `PIXEL_SIZEHome` | u32 | 未使用 | 保留 |

### 8.2 像素操作算法

在 `yy4mmhjJ` 和 `releaseBuffer8` 中：

```rust
// 条件：call_main_service_get_by_name("is_end") == "true" 且
//       (PIXEL_SIZE7 as u32 + PIXEL_SIZE5) > 30
for i in (0..len).step_by(PIXEL_SIZE6) {
    for j in 0..PIXEL_SIZE6 {
        if j == 3 {
            buffer[i + j] = PIXEL_SIZE4;           // Alpha 通道 = 固定值
        } else {
            let original = buffer[i + j] as u32;
            let new_value = original * PIXEL_SIZE5; // RGB × 乘法因子
            buffer[i + j] = new_value.min(PIXEL_SIZE8) as u8; // 限制最大值
        }
    }
}
```

默认参数 `Clipboard_Management|122|80|4|5|255`：
- PIXEL_SIZE4 = 122 → Alpha ≈ 48%
- PIXEL_SIZE5 = 80 → RGB × 80（极端亮化/暗化效果）
- PIXEL_SIZE6 = 4 → 每4字节一组（RGBA）
- PIXEL_SIZE7 = 5
- PIXEL_SIZE8 = 255 → 上限 255

### 8.3 "is_end" 查询机制

`call_main_service_get_by_name("is_end")` → JNI 调用 → `DFm8Y8iMScvB2YDw.DFm8Y8iMScvB2YDwGYN("is_end")`：

| 查询名（XOR 解密后） | 返回值 |
|---------------------|--------|
| `screen_info` | JSON `{width, height, scale}` |
| `is_end` | `isStart.toString()`（"true"/"false"） |
| `is_bis` | `BIS.toString()`（overlay 是否可见） |

---

## 第九章 用户验证与授权系统

### 9.1 ChinaNetworkTimeService

`flutter/lib/models/user_model.dart` 中实现的防篡改网络时间获取：

**NTP 服务器池（优先级从高到低）**：
1. cn.pool.ntp.org
2. ntp.ntsc.ac.cn（中科院）
3. time.edu.cn（教育网）
4. time.windows.com
5. ntp1.aliyun.com / ntp2.aliyun.com
6. time1.cloud.tencent.com / time2.cloud.tencent.com

**HTTP 时间源备选**：baidu.com, taobao.com, qq.com, jd.com, 163.com（读取响应头 Date 字段）

**时间获取优先级**：缓存（5分钟内有效）→ NTP → HTTP → 本地时间（最后手段）

### 9.2 validateUser 流程

```dart
Future<String?> validateUser(UserPayload user) async {
  // 1. 管理员跳过验证
  if (user.isAdmin) return null;

  // 2. 解析 email 格式: YYYYMMDDHHMI@UUID
  final parts = user.email.split('@');
  String expiryStr = parts[0];                    // 过期时间
  String? machineCode = parts.length > 1 ? parts[1] : null;  // 设备码

  // 3. 获取网络时间（防本地时间篡改）
  DateTime networkTime = await ChinaNetworkTimeService.getTime();

  // 4. 检查过期
  DateTime expiryDate = DateTime(YYYY, MM, DD, HH, MI);
  if (expiryDate.isBefore(networkTime)) return "account_expired";

  // 5. 检查设备绑定
  if (machineCode != null) {
    String currentUuid = await bind.mainGetUuid();
    if (machineCode != currentUuid) return "device_uuid_mismatch";
  }

  return null;  // 验证通过
}
```

### 9.3 登录错误处理

`flutter/lib/common/widgets/login.dart` 中的中文错误提示：

| 错误码 | 中文提示 |
|--------|---------|
| `account_expired` | 账号过期了！ |
| `invalid_expiry_date` | 授权日期错误！ |
| `device_uuid_mismatch` | 识别码不一致！ |
| 默认 | 您输入的账号或密码不匹配！ |

### 9.4 到期倒计时显示

`flutter/lib/desktop/pages/connection_page.dart`：

- 从 `user_email` LocalOption 中提取过期时间
- 每分钟定时刷新（Timer.periodic 1 minute）
- 显示格式：「X天Y小时Z分钟」+「YYYY年MM月DD日 HH:MM」
- 过期后显示 0 天 0 小时 0 分钟

---

## 第十章 Overlay 控制按钮系统

### 10.1 按钮完整清单

`flutter/lib/common/widgets/overlay.dart`：

| 按钮 | 颜色 | 回调链 | 最终效果 |
|------|------|--------|---------|
| **开共享** | 蓝色 | `onScreenStart("开")` → `tapStart` → mask=41 → `Benchmarks_Management1\|0\|1` | restoreMediaProjection() |
| **关共享** | 红色 | `onScreenStart("关")` → mask=41 → `Benchmarks_Management0\|0\|1` | killMediaProjection() |
| **开无视** | 蓝色 | `onScreenKitsch("开")` → `tapKitsch` → mask=40 → `SUPPORTED_ABIS_Management1\|0\|1` | shouldRun=true, 启动截图循环 |
| **关无视** | 红色 | `onScreenKitsch("关")` → mask=40 → `SUPPORTED_ABIS_Management0\|0\|1` | shouldRun=false |
| **开黑屏** | 蓝色 | `onScreenMask("开")` → `tapBlank` → mask=37 → `Clipboard_Management\|...\|#1` | overlay VISIBLE |
| **关黑屏** | 红色 | `onScreenMask("关")` → mask=37 → `...\|#0` | overlay GONE |
| **开穿透** | 蓝色 | `onScreenAnalysis("开")` → `tapAnalysis` → mask=39 → `HardwareKeyboard_Management\|...\|#1` | SKL=true |
| **关穿透** | 红色 | `onScreenAnalysis("关")` → mask=39 → `...\|#0` | SKL=false |

### 10.2 AntiShakeButton 防抖机制

自定义组件，800ms 防抖时间，点击后按钮变灰禁用：

```dart
void _handlePress() {
  setState(() => _isDisabled = true);
  widget.onPressed();
  Future.delayed(Duration(milliseconds: 800), () {
    if (mounted) setState(() => _isDisabled = false);
  });
}
```

### 10.3 按钮注册位置

`flutter/lib/common.dart` line ~1090：

```dart
onScreenMaskPressed: (input) => session.inputModel.onScreenMask(input),
onScreenAnalysisPressed: (input) => session.inputModel.onScreenAnalysis(input),
onScreenKitschPressed: (input) => session.inputModel.onScreenKitsch(input),
onScreenStartPressed: (input) => session.inputModel.onScreenStart(input),
```

---

## 第十一章 全局状态变量表

### 11.1 Kotlin 全局变量 (common.kt)

| 变量 | 类型 | 初始值 | 用途 |
|------|------|--------|------|
| `SKL` | Boolean | false | 穿透模式开关 |
| `shouldRun` | @Volatile Boolean | false | 无视模式开关（截图循环控制） |
| `gohome` | Int | 8 (GONE) | Overlay 可见性 (0=VISIBLE, 8=GONE) |
| `BIS` | Boolean | false | Overlay 实际可见状态 |
| `Wt` | Boolean | false | 辅助标志（无视模式入口标记） |
| `HomeWidth` | Int | 0 | 原始屏幕宽度 |
| `HomeHeight` | Int | 0 | 原始屏幕高度 |
| `HomeDpi` | Int | 0 | 原始屏幕 DPI |
| `SCREEN_INFO` | Info(w,h,scale,dpi) | (0,0,1,200) | 缩放后屏幕信息 |
| `ClassGen12TP` | String | "" | 文本粘贴缓冲 |
| `ClassGen12NP` | Boolean | false | 粘贴待处理标志 |
| `SDT` | Int | 100 | 保留参数 |
| `LOCAL_NAME` | String | Locale.getDefault() | 当前语言 |

### 11.2 Rust 全局变量 (pkg2230.rs)

| 变量 | 类型 | 用途 |
|------|------|------|
| `VIDEO_RAW` | `Mutex<FrameRaw>` | 视频帧缓冲（三模式共用写入点） |
| `PIXEL_SIZE4-8` | u8/u32/usize | 像素操作参数 |
| `PIXEL_SIZEBack` | u32 | 穿透模式帧放行开关 |
| `PIXEL_SIZEBack8` | u32 | 无视模式帧放行开关 |
| `PIXEL_SIZEA0-A5` | i32 | 穿透模式参数 |
| `BUFFER_LOCK` | Mutex<()> | JNI buffer 操作互斥锁 |
| `JVM` | RwLock<Option<JavaVM>> | JVM 实例 |
| `MAIN_SERVICE_CTX` | RwLock<Option<GlobalRef>> | MainService JNI 上下文 |
| `NDK_CONTEXT_INITED` | Mutex<bool> | NDK 初始化标志 |

---

## 第十二章 屏幕缩放策略

### 12.1 缩放算法

`DFm8Y8iMScvB2YDw.updateScreenInfo()`:

```kotlin
scale = calculateIntegerScaleFactor(originalWidth, 350)
// calculateIntegerScaleFactor = originalWidth / targetWidth (整数除法)

w /= scale     // 例如 1080/350 = 3, w = 360
h /= scale
dpi /= scale

SCREEN_INFO = Info(w, h, scale, dpi)
dd50d328f48c6896(w, h)  // 初始化 ErrorExceptions 和 IOExceptions 缓冲区
```

### 12.2 坐标还原

`nZW99cdXQ0COhB2o.onMouseInput()`:

```kotlin
mouseX = x * SCREEN_INFO.scale  // 控制端坐标 × 缩放因子 = 实际屏幕坐标
mouseY = y * SCREEN_INFO.scale
```

---

## 第十三章 构建系统

### 13.1 Android 构建 (build.sh)

完整流程：
1. **环境检查**：工具链 `/opt/rustdesk-toolchain/`、Flutter、Android SDK/NDK 27.2、vcpkg、Java 17
2. **签名检查**：`/opt/rustdesk-toolchain/signing/android/signing.env`
3. **Flutter patch**：`flutter_3.24.4_dropdown_menu_enableFilter.diff`
4. **Rust targets**：`aarch64-linux-android` (+armv7+x86_64 for universal)
5. **Bridge 检查**：`flutter_rust_bridge_codegen v1.80`
6. **依赖安装**：`build_android_deps.sh`（vcpkg 交叉编译）
7. **Rust 编译**：`cargo-ndk` → `ndk_arm64.sh` → `liblibrustdesk.so`
8. **SO 重命名**：`cp liblibrustdesk.so → libdaxian.so` + `libc++_shared.so`
9. **Flutter 打包**：`flutter build apk --release --split-per-abi`
10. **手动签名**：`zipalign` → `apksigner`
11. **输出**：`flutter/build/app/outputs/flutter-apk/app-aarch64-release.apk`

### 13.2 Windows 构建

`build.py --flutter --release`，加载 `librustdesk.dll`（未重命名），通过 `get_rustdesk_app_name()` JNI 获取运行时应用名 "DaxianMeeting"。

### 13.3 SO/DLL 命名关键链

```
Android:
  Cargo.toml [lib] name = "librustdesk"
  → cargo-ndk 编译产出: target/aarch64-linux-android/release/liblibrustdesk.so
  → build.sh cp 重命名: jniLibs/arm64-v8a/libdaxian.so
  → ffi.kt: System.loadLibrary("daxian") → 搜索 libdaxian.so ✓
  → pkg2230.kt: System.loadLibrary("daxian") → 搜索 libdaxian.so ✓
  → native_model.dart: DynamicLibrary.open('libdaxian.so') ✓

Windows:
  → cargo 编译产出: librustdesk.dll
  → main.cpp: LoadLibraryA("librustdesk.dll") ✓
  → native_model.dart: DynamicLibrary.open('librustdesk.dll') ✓
```

---

## 第十四章 其他关键子系统

### 14.1 开机自启

BootReceiver 监听 `BOOT_COMPLETED` → 检查 SharedPreferences `start_on_boot` → 检查权限 → 启动 MainService。

厂商自启动权限适配（`common.kt requestAutoStartPermission`）：华为、OPPO、vivo、小米 各自打开专属的自启动管理 Activity + 电池优化页面。

### 14.2 悬浮窗服务

`DFrLMwitwQbfu7AC.kt`：SVG 图标、屏幕边缘吸附、弹出菜单（显示应用/同步剪贴板/停止服务）、屏幕常亮控制。日志对比后不再强制 `alpha=0.0f`；未配置时保持可见常驻，配置为 0 时最低抬到 `alpha=0.01f`，让系统更像识别到真实常驻悬浮窗；极低透明度下仍按不可触摸处理，避免干扰用户触摸。

### 14.3 无障碍服务配置

`accessibility_service_config.xml` 配置全权限：
- `typeAllMask` — 监听所有事件类型
- `packageNames="@null"` — 监听所有应用
- `canTakeScreenshot="true"` — 截屏权限（无视模式依赖）
- `canPerformGestures="true"` — 手势模拟
- `canRetrieveWindowContent="true"` — 窗口内容获取
- `isAccessibilityTool="true"` — 声明为辅助工具

### 14.4 剪贴板同步

`ig2xH1U3RDNsb7CS.kt`：使用 protobuf `MultiClipboards` 消息格式，支持纯文本和 HTML 格式，通过 `ClsFx9V0S._O2EiFD4()` JNI 发送到 Rust 层。

---

## 第十五章 已知问题与 Bug 清单

| 序号 | 问题 | 严重性 | 位置 |
|------|------|--------|------|
| 1 | Virtual Display key 不匹配：Rust 发 `rustdesk_virtual_displays`，Dart 期望 `daxian_virtual_displays` | **高** | `virtual_display_manager.rs` vs `consts.dart` |
| 2 | `PIXEL_SIZE` 系列变量使用 `static mut`，存在线程安全隐患 | 中 | `pkg2230.rs` |
| 3 | `ffi.rs` 是 `pkg2230.rs` 的冗余备份，修改时需双份同步 | 中 | `libs/scrap/src/android/` |
| 4 | Windows 端 DLL 名仍为 `librustdesk.dll`，未改名 | 低 | `main.cpp`, `native_model.dart` |
| 5 | `verify_rustdesk_password_tip` 翻译 key 残留 RustDesk 名称 | 低 | `dialog.dart` |
| 6 | `is_rustdesk()` 返回 false（APP_NAME != "RustDesk"），依赖此函数的逻辑可能行为变化 | 中 | `common.rs` |
| 7 | `targetSdkVersion 33`（低于当前 Google Play 要求的 34+） | 中 | `build.gradle` |

---

## 第十六章 后续开发操作手册

### 16.1 新增控制命令的完整步骤

1. **`src/common.rs`**：添加 `MOUSE_TYPE_XXX` 常量（如 11）
2. **`src/flutter_ffi.rs`**：在 `session_send_mouse` 的 match 中添加 `"wheelxxx" => MOUSE_TYPE_XXX`
3. **`flutter/lib/models/input_model.dart`**：
   - 添加 `tapXxx()` 方法
   - 添加 `onScreenXxx()` 方法
   - 在 `sendMouse()` 中添加 URL 编码逻辑
4. **`flutter/lib/common/widgets/overlay.dart`**：添加 AntiShakeButton
5. **`flutter/lib/common.dart`**：注册回调 `onScreenXxxPressed: (input) => session.inputModel.onScreenXxx(input)`
6. **`libs/scrap/src/android/pkg2230.rs`**：
   - 在 `call_main_service_pointer_input()` 添加 mask 分支
   - 如需新 JNI 函数调用，添加 `call_main_service_set_by_name()` 调用
7. **`flutter/android/.../DFm8Y8iMScvB2YDw.kt`**：
   - 在 `DFm8Y8iMScvB2YDwSBN()` 添加 name 处理分支
8. **同步 `ffi.rs`**：如果保留 ffi.rs，需同步 pkg2230.rs 的修改

### 16.2 修改 SO/DLL 名称的步骤

Android:
1. `build.sh` → `build_rust_lib_for_target()` 的 cp 行
2. `ffi.kt` → `System.loadLibrary("xxx")`
3. `pkg2230.kt` → `System.loadLibrary("xxx")`
4. `native_model.dart` → `DynamicLibrary.open('libxxx.so')`

Windows:
1. `flutter/windows/runner/main.cpp` → `LoadLibraryA("xxx.dll")`
2. `native_model.dart` → `DynamicLibrary.open('xxx.dll')`

### 16.3 修改 Protobuf 的步骤

1. 修改 `libs/hbb_common/protos/message.proto`
2. Rust 侧：重新编译（cargo build 自动重生成）
3. Android 侧：build.gradle 配置的 protobuf 插件自动从 `libs/hbb_common/protos/` 编译

### 16.4 清理缓存

包名/配置变更后必须执行：
```bash
rm -rf flutter/build/
rm -rf flutter/android/.gradle/
rm -rf flutter/.dart_tool/
cd flutter && flutter clean && flutter pub get
```

---

## 第十七章 文件修改追踪总表

| 文件 | 修改类型 | 摘要 |
|------|---------|------|
| `Cargo.toml` | 品牌+配置 | 版本5.2.0, authors, description, winres, bundle |
| `build.sh` | 新增 | Android 完整构建脚本 |
| `migrate_package.sh` | 新增 | 包名迁移工具 |
| `libs/hbb_common/src/config.rs` | 品牌 | APP_NAME="DaxianMeeting" |
| `libs/hbb_common/protos/message.proto` | 协议 | MouseEvent 增加 url 字段 |
| `libs/scrap/src/android/mod.rs` | 路由 | 只导出 pkg2230 |
| `libs/scrap/src/android/pkg2230.rs` | 核心 | 全部 JNI 层、三模式捕获、命令分发 |
| `libs/scrap/src/android/ffi.rs` | 备份 | pkg2230.rs 副本 |
| `src/common.rs` | 核心 | 自定义 MOUSE_TYPE, is_custom_client=false, verify_login=true |
| `src/client.rs` | 协议 | send_mouse() 增加 url 参数 |
| `src/flutter_ffi.rs` | 协议 | 自定义 wheel* 类型映射 |
| `src/ui_session_interface.rs` | 协议 | send_mouse() url 参数透传 |
| `src/server/connection.rs` | 协议 | call_main_service_pointer_input 传递 url |
| `flutter/pubspec.yaml` | 依赖 | ntp, encrypt, crypto 依赖 |
| `flutter/lib/consts.dart` | 品牌 | daxian_virtual_displays |
| `flutter/lib/common.dart` | UI | overlay 按钮回调注册, macOS channel |
| `flutter/lib/common/widgets/overlay.dart` | UI | 8个控制按钮 + AntiShakeButton |
| `flutter/lib/common/widgets/login.dart` | 逻辑 | curOP='daxian', 中文错误提示 |
| `flutter/lib/models/native_model.dart` | FFI | Android 加载 libdaxian.so |
| `flutter/lib/models/input_model.dart` | 协议 | 自定义 sendMouse + URL 编码 |
| `flutter/lib/models/user_model.dart` | 新增 | ChinaNetworkTimeService + validateUser |
| `flutter/lib/desktop/pages/connection_page.dart` | UI | 到期倒计时显示 |
| `flutter/lib/mobile/pages/server_page.dart` | UI | 自启动权限、悬浮窗开关 |
| `flutter/lib/utils/platform_channel.dart` | 品牌 | com.daxian.dev/macos |
| `flutter/android/AndroidManifest.xml` | 品牌+配置 | 全部组件声明 |
| `flutter/android/app/build.gradle` | 品牌 | applicationId, protobuf |
| `flutter/android/.../strings.xml` | 品牌 | 大仙会议 |
| `flutter/android/.../accessibility_service_config.xml` | 配置 | 全权限 |
| `flutter/android/.../kotlin/ffi.kt` | FFI | 第一桥，混淆函数名 |
| `flutter/android/.../kotlin/pkg2230.kt` | FFI | 第二桥 ClsFx9V0S |
| `flutter/android/.../com/daxian/dev/*.kt` | 核心 | 全部混淆类 |
| `flutter/android/.../com/daxian/dev/p50.java` | 加密 | XOR 入口 |
| `flutter/android/.../com/daxian/dev/q50.java` | 加密 | XOR 实现 |
| `flutter/windows/runner/main.cpp` | 未改 | 仍加载 librustdesk.dll |

---
