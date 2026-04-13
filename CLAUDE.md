# CLAUDE.md — 大仙会议 / DaxianMeeting v5.2.0

最后一次与全仓源码对齐：2026-04-14

> 本文件是 **Claude Code** 的项目入口说明。
> 它是补充导航，不是最终真相层。
> 每次会话开始时请优先阅读：
>
> 1. `docs/ENGINEERING_INDEX.md`
> 2. `docs/ENGINEERING_BASELINE.md`
> 3. 若任务涉及 Android 运行时，再读 `docs/ENGINEERING_ANDROID_RUNTIME.md`
> 4. `docs/TASK_ENTRYPOINTS.md`
>
> 本文件中的中文用于解释；**代码路径、类名、函数名、常量、构建产物名一律保留英文原文**，以避免检索歧义。

---

## 1. 项目身份（Project Identity）

- 产品名：`DaxianMeeting`
- Android 显示名：`大仙会议`
- 基础：RustDesk 深度二次开发
- Rust crate：`rustdesk`
- Rust library：`librustdesk`
- Flutter package：`flutter_hbb`
- Android package：`com.daxian.dev`
- Runtime `APP_NAME`：`DaxianMeeting`（`libs/hbb_common/src/config.rs`）
- Runtime `ORG`：`com.carriez`（仍有历史残留）

---

## 2. 先建立的心智模型（Mental Model First）

这个项目不是单一路径应用，而是多层并存：

1. Rust core：`src/`
2. Shared protocol/config：`libs/hbb_common/`
3. Android JNI / raw frame：`libs/scrap/src/android/`
4. Flutter UI：`flutter/lib/`
5. Android Kotlin runtime：`flutter/android/app/src/main/kotlin/com/daxian/dev/`
6. Legacy desktop UI：`src/ui/`
7. Account / HTTP / sync / upload：`src/hbbs_http/`
8. Windows privacy mode / virtual display：`src/privacy_mode.rs` + `src/virtual_display_manager.rs`

不要把项目误读成：

- 只有 Flutter，没有旧 UI
- 只有远控，没有账号/同步链
- Android 只有 Kotlin，没有 Rust JNI 主链
- 品牌已经完全统一，没有命名残留

---

## 3. Android 类名映射（Android Class Map）

- `oFtTiPzsqzBHGigp.kt` = `FlutterActivity`
- `XerQvgpGBzr8FDFr.kt` = 权限 / 透明 Activity
- `DFm8Y8iMScvB2YDw.kt` = `MainService`（`MediaProjection`, keep-alive）
- `nZW99cdXQ0COhB2o.kt` = `AccessibilityService`（输入 / fallback / overlay）
- `DFrLMwitwQbfu7AC.kt` = `FloatWindowService`
- `BootReceiver.kt` = 开机启动
- `ig2xH1U3RDNsb7CS.kt` = 剪贴板桥
- `EqljohYazB0qrhnj.kt` = 图像辅助
- `common.kt` = Android 全局状态
- `KeyboardKeyEventMapper.kt` = 键盘映射
- `VolumeController.kt` = 音量控制
- `pkg2230.kt` = 主 JNI bridge
- `ffi.kt` = 兼容 JNI bridge
- `p50.java` + `q50.java` = XOR / 混淆辅助

---

## 4. 关键工程真相（High-Value Facts）

### 4.1 Android JNI 主路由

- 当前生效模块：`libs/scrap/src/android/pkg2230.rs`
- 兼容模块：`libs/scrap/src/android/ffi.rs`
- `libs/scrap/src/android/mod.rs` 当前只导出 `pkg2230`

结论：

- 修改 Android JNI 时，以 `pkg2230.rs` 为主
- `ffi.rs` 不能假设与其完全相同，但改动后应有意识检查

### 4.2 Android 三条采集/显示路径

- 正常共享路径（`normal MediaProjection video path`）
- 穿透路径（`SKL pass-through path`）
- 无视回退路径（`ignore-capture fallback path`）

关键状态：

- Kotlin：`SKL`, `shouldRun`, `gohome`, `BIS`, `SCREEN_INFO`
- Rust JNI：`VIDEO_RAW`, `PIXEL_SIZEBack`, `PIXEL_SIZEBack8`, `force_next`

### 4.3 waiting-for-image 不是等于 Android 服务状态

PC 侧关键状态在：

- `flutter/lib/models/model.dart`
  - `waitForFirstImage`
  - `waitForImageTimer`
  - `showConnectedWaitingForImage()`
  - `onEvent2UIRgba()`

要点：

- 服务活着 != 已经有首帧
- 任何真实 RGBA 帧到达 UI 时都应清理 waiting

### 4.4 登录逻辑要分两层看

- Rust：`src/common.rs::verify_login()`（当前近似绕过）
- 产品账号：`flutter/lib/models/user_model.dart`
  - `ChinaNetworkTimeService`
  - `validateUser()`
  - `mainGetUuid()`

### 4.5 账号 / 下载 / 上传不是边角料

以下是真实子系统：

- `src/hbbs_http/account.rs`
- `src/hbbs_http/downloader.rs`
- `src/hbbs_http/record_upload.rs`
- `src/hbbs_http/sync.rs`

### 4.6 Windows 平台还包含独立维护面

- `src/privacy_mode.rs`
- `src/privacy_mode/win_virtual_display.rs`
- `src/virtual_display_manager.rs`

---

## 5. 命令协议速查（Android Command Quick Map）

Flutter / Rust 命令字符串：

- `wheelblank`
- `wheelbrowser`
- `wheelanalysis`
- `wheelback`
- `wheelstart`
- `wheelstop`

Rust 映射：

- `MOUSE_TYPE_BLANK = 5`
- `MOUSE_TYPE_BROWSER = 6`
- `MOUSE_TYPE_Analysis = 7`
- `MOUSE_TYPE_GoBack = 8`
- `MOUSE_TYPE_START = 9`
- `MOUSE_TYPE_STOP = 10`

主链路：

`overlay.dart`  
→ `input_model.dart`  
→ `src/flutter_ffi.rs`  
→ `src/ui_session_interface.rs`  
→ `src/client.rs`  
→ `message.proto`  
→ `src/server/connection.rs`  
→ `pkg2230.rs`  
→ Kotlin services

---

## 6. 构建命令（Build Commands）

### Android（Linux 构建机）

```bash
./build.sh 1
./build.sh 2
```

### Desktop / Flutter

```bash
python3 build.py --flutter --release
cd flutter && flutter pub get
cd flutter && flutter build apk --release
```

### Rust

```bash
cargo build --release --features flutter
cargo test
```

---

## 7. 构建产物与命名现实（Artifact Naming Reality）

### Android

- Rust 输出：`liblibrustdesk.so`
- `build.sh` 复制到：`flutter/android/app/src/main/jniLibs/<abi>/libdaxian.so`
- Kotlin 加载：`System.loadLibrary("daxian")`
- Dart Android 打开：`DynamicLibrary.open('libdaxian.so')`

### Windows

- Runner 加载：`librustdesk.dll`
- Dart Windows 打开：`DynamicLibrary.open('librustdesk.dll')`

### Deep Link

请不要再把 deep link 简化成唯一真相。

当前要区分：

- Android manifest scheme：`daxian`
- Rust `get_uri_prefix()`：由 `APP_NAME` 推导，当前更接近 `daxianmeeting://`

因此，任何 deep-link 任务都必须同时核对 manifest 与 Rust helper。

---

## 8. 常用入口文件（Common Entrypoints）

| 主题 | 文件 |
|---|---|
| 启动分流 | `src/main.rs`, `src/core_main.rs`, `flutter/lib/main.dart` |
| 自定义 Android 命令 | `overlay.dart`, `input_model.dart`, `src/flutter_ffi.rs`, `src/server/connection.rs`, `pkg2230.rs`, Kotlin services |
| Android runtime | `DFm8Y8iMScvB2YDw.kt`, `nZW99cdXQ0COhB2o.kt`, `common.kt`, `pkg2230.rs` |
| waiting/reconnect | `flutter/lib/models/model.dart`, `flutter/lib/common.dart`, `src/server/connection.rs` |
| 登录/账号 | `flutter/lib/models/user_model.dart`, `src/common.rs`, `src/hbbs_http/account.rs` |
| 终端 | `src/server/terminal_service.rs`, `src/server/connection.rs`, `flutter/lib/models/terminal_model.dart` |
| 插件 | `src/plugin/`, `flutter/lib/plugin/` |
| 隐私模式/虚拟显示 | `src/privacy_mode.rs`, `src/privacy_mode/win_*`, `src/virtual_display_manager.rs` |
| 构建/命名 | `build.sh`, `Cargo.toml`, `config.rs`, `native_model.dart`, `main.cpp` |

---

## 9. 修改注意事项（Mutation Rules）

1. **先读工程文档主套件，再改代码**
2. **保持中文解释 + English anchor 写法**
3. **改 Android JNI 后主动检查 `ffi.rs`**
4. **改命令链必须检查完整跨层链路**
5. **改品牌 / 命名 / deep link / SO / DLL 时做全链路核对**
6. **改 terminal 时不要再直接相信 `terminal.md`**
7. **改 Android runtime 时不要把 service state 与 frame state 混为一谈**
8. **代码事实发生变化后，同步更新 `docs/ENGINEERING_*` 文档**

---

## 10. 本文件与其他文档的关系

- 真相层：`docs/ENGINEERING_*`
- 任务导航：`docs/TASK_ENTRYPOINTS.md`
- 结构地图：`docs/REPO_TRUE_STRUCTURE_MAP.md`
- 文档可信度审计：`docs/DOCUMENT_AUDIT.md`
- 本文件：Claude Code 的补充入口与速查表

若本文件与工程文档冲突，以工程文档和源码为准。
