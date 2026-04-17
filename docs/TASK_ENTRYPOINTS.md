# 任务入口点 / Task Entrypoints

最后一次从全仓源码核验：2026-04-17

> 本文件按“改动类型”给出第一批应该打开的文件。
> 目标是让 Codex / Claude Code 从**最短、最对的调用链入口**开始。
> 所有路径保留英文原文，中文只负责解释任务边界。

---

## 0. 当前任务纪律与最新热修入口（Current Task Guard）

后续任务必须遵守：

- 不替用户执行 `git commit`。
- 不执行编译/构建命令，除非用户之后明确改变要求。
- 重要修改后同步当前 `docs/` 工程文档。
- 若文档与源码冲突，先相信源码，再修正文档。

黑屏 overlay / 远程输入卡顿相关任务，第一入口固定为：

- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`
- `docs/ENGINEERING_ANDROID_RUNTIME.md`

防止误改范围：

- 不要优先改 `pkg2230.rs` 的 `PIXEL_SIZE*` 逻辑；它属于黑屏视觉/像素路径，不是本次动态 touch flag 卡顿根因。
- 不要改 Rust FFI、命令协议、侧按钮 mask、`overlay.dart` 或 `input_model.dart`，除非新问题的调用链明确指向这些文件。
- 检查是否误恢复 `isBlackScreenActive` / `restoreBlockRunnable` / `setOverlayTouchBlock` / `FLAG_NOT_TOUCHABLE` 动态切换。

开防触 / 关防触相关任务，第一入口固定为：

- `flutter/lib/common/widgets/overlay.dart`
- `flutter/lib/models/input_model.dart`
- `src/flutter_ffi.rs`
- `libs/scrap/src/android/pkg2230.rs`
- `libs/scrap/src/android/ffi.rs`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`

防触摸排查关键词：

- `wheeltouch`
- `MOUSE_TYPE_TOUCHBLOCK`
- `TouchBlock_Management`
- `touch_block`
- `touchBlockOverlay`
- `setTouchBlockEnabled`

---

## 1. 任何改动之前（Before Any Change）

先做：

```bash
git -c safe.directory="$PWD" status --short
rg -n "<feature keyword>" src libs flutter docs CLAUDE.md terminal.md
```

然后判断：

- worktree 是否脏？
- 改动是否跨层（Flutter + Rust + server + Android）？
- 是否涉及已知文档漂移（见 `docs/DOCUMENT_AUDIT.md`）？
- 是否需要先看 `docs/ENGINEERING_ANDROID_RUNTIME.md`？

---

## 2. 启动 / 进程行为 / 参数分流（Startup / Process / Args）

先看：

- `src/main.rs`
- `src/core_main.rs`
- `src/lib.rs`
- `flutter/lib/main.dart`

再按平台补看：

- `src/ui.rs`
- `src/ui/`
- `flutter/windows/runner/main.cpp`
- `flutter/linux/`
- `flutter/macos/`

检查点：

- 这是 Rust 启动分流还是 Flutter 启动分流？
- 是桌面主窗口、远程多窗口、CM、install 还是移动端？
- 是否仍有旧 `Sciter UI` 路径被触发？
- 参数是否影响 tray / server / install / quick support / elevate？

---

## 3. Android 控制按钮 / 自定义命令（Android Control Commands）

先看：

- `flutter/lib/common/widgets/overlay.dart`
- `flutter/lib/common.dart`
- `flutter/lib/models/input_model.dart`
- `src/flutter_ffi.rs`
- `src/ui_session_interface.rs`
- `src/client.rs`
- `libs/hbb_common/protos/message.proto`
- `src/server/connection.rs`
- `libs/scrap/src/android/pkg2230.rs`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`

新增命令时必须逐项确认：

1. UI 上是否有按钮 / 回调
2. `sendMouse()` 是否编码了正确 type / url
3. `src/flutter_ffi.rs` 是否映射到正确 `MOUSE_TYPE_*`
4. `message.proto` 是否承载了所需字段
5. `src/server/connection.rs` 是否接收并转发
6. `pkg2230.rs` 是否分发到 Kotlin
7. Kotlin service 是否真的执行了逻辑

---

## 4. Android 采集 / 分享 / 无视 / 穿透 / 黑屏（Android Capture / Share / Ignore / SKL / Blank）

先看：

- `docs/ENGINEERING_ANDROID_RUNTIME.md`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/common.kt`
- `libs/scrap/src/android/pkg2230.rs`
- `libs/scrap/src/android/ffi.rs`
- `src/server/connection.rs`
- `flutter/lib/models/model.dart`
- `flutter/lib/common/widgets/overlay.dart`
- `flutter/lib/models/input_model.dart`

必须重新核对：

- `SKL`
- `shouldRun`
- `VIDEO_RAW`
- `PIXEL_SIZEBack`
- `PIXEL_SIZEBack8`
- `force_next`
- `killMediaProjection()`
- `handleProjectionStoppedKeepService()`
- `restoreMediaProjection()`
- `startIgnoreFallback()`

若涉及 waiting / reconnect，再补看：

- `flutter/lib/models/model.dart`
- `flutter/lib/common.dart`
- `src/flutter.rs`
- `flutter/lib/desktop/screen/desktop_remote_screen.dart`
- `flutter/lib/desktop/widgets/remote_toolbar.dart`

---

## 5. waiting-for-image / Android 首帧 / 重连（Waiting / First Frame / Reconnect）

先看：

- `flutter/lib/models/model.dart`
- `flutter/lib/common.dart`
- `src/server/connection.rs`
- `src/flutter.rs`

要确认：

- `waitForFirstImage`
- `waitForImageTimer`
- `showConnectedWaitingForImage()`
- `onEvent2UIRgba()`
- Android 平台 additions：
  - `android_sdk_int`
  - `android_ignore_capture_supported`

不要遗漏：

- waiting dialog 与 Android overlay 的层级关系
- fallback request 的触发时机
- “任何真实首帧都能清理 waiting”的不变量

---

## 6. 协议 / protobuf / 消息形状（Protocol / Message Shape）

先看：

- `libs/hbb_common/protos/message.proto`
- `libs/hbb_common/build.rs`
- `src/client.rs`
- `src/server/connection.rs`
- `src/flutter_ffi.rs`
- Flutter 调用端：`flutter/lib/models/`

检查清单：

- protobuf 字段是否新增 / 修改 / 重排
- 发送端是否更新
- 接收端是否更新
- Flutter ↔ Rust bridge 是否更新
- Android / desktop / mobile 平台分支是否都核过

---

## 7. 登录 / 账号 / 到期 / UUID 绑定（Login / Account / Expiry / UUID Binding）

先看：

- `flutter/lib/models/user_model.dart`
- `flutter/lib/common/widgets/login.dart`
- `flutter/lib/desktop/pages/connection_page.dart`
- `src/common.rs`
- `src/ui_interface.rs`
- `src/flutter_ffi.rs`

若涉及账号/OIDC，再补看：

- `src/hbbs_http/account.rs`
- `src/hbbs_http/http_client.rs`

必须分清两个概念：

1. Rust `verify_login()`（当前近似绕过）
2. Flutter 产品登录校验（到期 / UUID / 网络时间）

不要把两者混成一个“登录逻辑”。

---

## 8. OIDC / 下载 / 同步 / 上传（HBBS HTTP Tasks）

先看：

- `src/hbbs_http/account.rs`
- `src/hbbs_http/downloader.rs`
- `src/hbbs_http/http_client.rs`
- `src/hbbs_http/record_upload.rs`
- `src/hbbs_http/sync.rs`
- `src/flutter_ffi.rs`
- `src/ui_interface.rs`
- `src/rendezvous_mediator.rs`
- `src/server/connection.rs`

任务类型与入口：

### 8.1 OIDC / account auth

- `account.rs`
- `main_account_auth`
- `account_auth_cancel`

### 8.2 下载器 / 进度轮询

- `downloader.rs`
- `get_download_data()`
- `download_file()`

### 8.3 录像上传

- `record_upload.rs`
- `src/server/video_service.rs`

### 8.4 sync / pro 状态

- `sync.rs`
- `signal_receiver()`
- `is_pro()`

---

## 9. 终端（Terminal）

先看：

- `src/server/terminal_service.rs`
- `src/server/connection.rs`
- `libs/hbb_common/protos/message.proto`
- `src/flutter_ffi.rs`
- `flutter/lib/models/terminal_model.dart`
- `flutter/lib/desktop/pages/terminal_connection_manager.dart`
- `flutter/lib/desktop/pages/terminal_tab_page.dart`

再看参考：

- `terminal.md`（仅作历史背景，不能直接当真相层）

必须核对：

- `generate_service_id()` 当前格式
- terminal open / data / resize / close 路径
- connection 断开后 terminal 行为
- 是否真的持久化了你想依赖的那部分状态

---

## 10. Plugin（插件）

先看：

- `Cargo.toml`
- `src/plugin/mod.rs`
- `src/plugin/manager.rs`
- `src/plugin/plugins.rs`
- `src/plugin/native_handlers/`
- `flutter/lib/plugin/`

先确认：

- 目标构建是否启用了 `plugin_framework`
- 是 plugin runtime 行为，还是 plugin 下载 / 安装 / UI

---

## 11. 隐私模式 / 虚拟显示器 / Windows 平台（Privacy Mode / Virtual Display / Windows）

先看：

- `src/privacy_mode.rs`
- `src/privacy_mode/win_virtual_display.rs`
- `src/privacy_mode/win_topmost_window.rs`
- `src/privacy_mode/win_mag.rs`
- `src/privacy_mode/win_exclude_from_capture.rs`
- `src/privacy_mode/win_input.rs`
- `src/virtual_display_manager.rs`
- `src/server/connection.rs`
- `flutter/lib/consts.dart`

重点核对：

- `supported_privacy_mode_impl`
- `daxian_virtual_displays`
- 连接侧 turn on / turn off 路径
- Windows-only 假设是否成立

---

## 12. 桌面旧 UI / Sciter 路径（Legacy Desktop UI / Sciter）

先看：

- `src/ui.rs`
- `src/ui/remote.rs`
- `src/ui/cm.rs`
- `src/core_main.rs`

适用场景：

- 桌面启动问题
- 非 Flutter 的旧路径兼容问题
- 某些桌面功能在旧 UI 下的保留行为

---

## 13. Flutter 桌面 / 移动 / web 分层（Flutter Layer Tasks）

先看：

- `flutter/lib/main.dart`
- `flutter/lib/common.dart`
- `flutter/lib/models/`
- `flutter/lib/desktop/`
- `flutter/lib/mobile/`
- `flutter/lib/web/`
- `flutter/lib/utils/platform_channel.dart`
- `flutter/lib/models/native_model.dart`

适用场景：

- 页面 / 状态 / overlay / dialog
- desktop multi-window
- 平台 channel
- Dart 动态库加载

---

## 14. 品牌 / 命名 / URI scheme / 构建产物（Branding / Naming / Deep Link / Artifacts）

先看：

- `Cargo.toml`
- `libs/hbb_common/src/config.rs`
- `flutter/pubspec.yaml`
- `flutter/android/app/build.gradle`
- `flutter/android/app/src/main/AndroidManifest.xml`
- `src/common.rs`
- `build.sh`
- `flutter/lib/models/native_model.dart`
- `flutter/windows/runner/main.cpp`
- `flutter/android/app/src/main/kotlin/pkg2230.kt`
- `flutter/android/app/src/main/kotlin/ffi.kt`

常见坑：

- manifest scheme 改了，但 Rust `get_uri_prefix()` 没改
- Android SO 改名了，但 Kotlin / Dart loader 没改
- Windows DLL 仍保留旧名
- 包名与可见品牌改了，但 `APP_NAME` / `ORG` / helper path 没统一

---

## 15. 构建 / 打包 / 环境脚本（Build / Packaging / Env Scripts）

先看：

- `build.sh`
- `env.sh`
- `build.py`
- `build.rs`
- `migrate_package.sh`
- `flutter/build_android.sh`
- `flutter/build_android_deps.sh`
- `flutter/android/app/build.gradle`
- `appimage/`
- `flatpak/`
- `fastlane/`
- `res/`

适用场景：

- 产物命名
- 平台打包
- Android NDK / cargo-ndk
- 包名迁移
- 资源与 installer

---

## 16. Android 辅助能力（Android Auxiliary Surfaces）

先看：

- `BootReceiver.kt`
- `ig2xH1U3RDNsb7CS.kt`
- `KeyboardKeyEventMapper.kt`
- `VolumeController.kt`
- `XerQvgpGBzr8FDFr.kt`
- `oFtTiPzsqzBHGigp.kt`

适用场景：

- 开机启动
- 剪贴板
- 音量键 / 键盘事件
- 权限 / overlay / 特殊 Activity

---

## 17. 任意改动之后（After Any Change）

最少做这些：

1. 用 `rg` 重新扫一遍改动关键词
2. 若功能跨层，重开完整调用链
3. 做最小但有效的验证
4. 若事实变化，更新：
   - `docs/ENGINEERING_BASELINE.md`
   - 若涉及 Android runtime，再更新 `docs/ENGINEERING_ANDROID_RUNTIME.md`
   - 若入口变化，再更新 `docs/TASK_ENTRYPOINTS.md`
   - 若文档可信度结论变化，再更新 `docs/DOCUMENT_AUDIT.md`
5. 继续沿用：
   - 中文解释
   - English path / symbol anchor
   - 同一概念的 canonical term 不漂移
