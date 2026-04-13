# 项目文档入口

> 目标：让后续工程会话用最少上下文恢复项目状态，避免被旧文档带偏。

## 必读顺序

1. `docs/PROJECT_MEMORY.md`
   - 当前事实表、关键链路、易错点。
2. `docs/CHANGELOG.md`
   - 已落地修改，尤其 v5.2.1 和 hotfix。
3. `docs/KNOWN_BUGS.md`
   - 当前 bug 状态和已修复项。
4. `DOCS.md`
   - 大型技术手册，只作为背景资料；使用前必须回源码验证。
5. `terminal.md`
   - 终端系统设计资料；部分实现状态已变化，需与源码交叉验证。

## 可信度规则

源码 > `docs/PROJECT_MEMORY.md` > `docs/CHANGELOG.md` / `docs/KNOWN_BUGS.md` > `DOCS.md` / `CLAUDE.md` / `terminal.md`。

若文档互相冲突：
- 先 `rg` 查源码。
- 再看 `CHANGELOG` 是否说明修复或回退。
- 最后更新 `PROJECT_MEMORY.md`，避免下次重复踩坑。

## 协作铁律

- Codex 可以修改代码和文档。
- Codex 不允许执行 `git commit`。
- Git 提交只能由用户审查后亲自完成。
- 修改前必须查看工作树，避免覆盖用户改动。

## 快速检索模板

```powershell
git -c safe.directory=C:/Users/Administrator/Desktop/Code/Test status --short
rg -n "<关键词>" src libs flutter docs DOCS.md
```

## 高风险改动入口

| 改动类型 | 必查文件 |
|---|---|
| Android 控制按钮 | `overlay.dart`, `input_model.dart`, `flutter_ffi.rs`, `pkg2230.rs`, `DFm8Y8iMScvB2YDw.kt`, `nZW99cdXQ0COhB2o.kt` |
| 协议字段 | `libs/hbb_common/protos/message.proto`, `src/client.rs`, `src/server/connection.rs`, Flutter/Dart 调用点 |
| SO 名称 | `build.sh`, `ffi.kt`, `pkg2230.kt`, `native_model.dart` |
| Windows DLL 名称 | `flutter/windows/runner/main.cpp`, `native_model.dart`, Windows packaging scripts |
| Deep link | `AndroidManifest.xml`, `src/common.rs`, `src/core_main.rs`, `flutter/lib/common.dart`, `flutter/lib/mobile/pages/home_page.dart` |
| 服务认证旁路 | `src/common.rs`, `src/ui.rs`, `src/flutter_ffi.rs` |
| 用户授权/到期 | `user_model.dart`, `login.dart`, `connection_page.dart` |
| 终端 | `terminal_service.rs`, `connection.rs`, `message.proto`, `terminal_model.dart`, `terminal_connection_manager.dart`, `terminal_tab_page.dart` |
| 插件 | `Cargo.toml` feature flags, `src/plugin/`, `flutter/lib/plugin/` |

## 当前已知文档偏差

- `DOCS.md` 仍说 Virtual Display key 未修复；源码已修复。
- `DOCS.md` / 旧 `PROJECT_MEMORY.md` 曾说 `pkg2230.rs` 与 `ffi.rs` 完全相同；实际不相同，且不能盲目双文件复制。
- Android Manifest scheme 是 `daxian`，Rust `get_uri_prefix()` 当前是 `daxianmeeting://`。
- `terminal.md` 的终端持久化说明不是完整当前实现；恢复闭环仍需源码确认。


## 近期关键注意事项

- PC Android 重连/等待首帧：提交 `34d072b0fa8f80f2a0d313ab24e3a96bcee0270e` 曾绕过 Android 等待提示路径。当前工作区已恢复连接/重连 loading 和 `waiting-for-image` 提示，保留 10 秒自动 `开无视`，并把 Android 侧按钮放到提示框上层。
- PC 侧按钮去重：Android 重连/等待画面时，`waitForFirstImage=true` 会隐藏页面内侧按钮，只插入一份可追踪的提示框上层侧按钮。任意 RGBA/Texture 画面帧到达后移除这份 entry，避免重连后出现两个侧按钮。
- Android 重连备用画面：等待画面提示出现后会请求无视截屏备用帧，10 秒后仍无首帧会再次请求；PC 端可以接收视频流或无视截屏流，谁先到就清理等待状态。
- Android 控制命令：侧按钮自定义命令不再被普通 keyboard 权限拦截；`startIgnoreFallback()` 会统一打开 `VIDEO_RAW`、放行 `PIXEL_SIZEBack8` 并请求无视截图循环，避免重连后截屏帧无法进入 PC。
- 冻结帧注意：Android Rust `FrameRaw` 有 `force_next`，重新启用原始帧流后下一帧强制发送，用来解决锁屏冻结帧/无视截屏流重连后被重复帧判断挡住的问题。
- 锁屏切流规则：`ACTION_SCREEN_OFF` 不主动停 MediaProjection；只有系统真的停止投屏后才切无视。
- Android 保活：MainService 在 ready/running 时应保持悬浮窗服务；`DFrLMwitwQbfu7AC` 是 sticky 服务，并加固了 add/remove view。
- 日志脚本：`scripts/capture_android_keepalive_logs.ps1` 可抓锁屏保活/断联日志，有 root 时可加 `-RootKernel`。
- 手机端 root 脚本：`scripts/android_keepalive_log_toggle.sh` 可在设备上 toggle 抓取，第一次开始、第二次停止。
- 锁屏保活：`ACTION_SCREEN_OFF` 时 MainService 不主动停止/释放 MediaProjection，只刷新前台服务和保活；系统真的停止投屏后才切无视。非显式停止导致 MainService 销毁时，会通过 `ACT_KEEP_ALIVE_SERVICE` 尽力前台重启。
- Activity 前台保活：主 Activity `onStart()` 不再在服务已接入时无条件停止悬浮窗服务；服务 ready 且未禁用悬浮窗时，会继续保持悬浮窗服务。
- 防闪退加固：主 Activity 拉起悬浮窗服务前会检查悬浮窗权限，并保护悬浮窗服务 start/stop 异常；PC 端延迟侧按钮/无视备用帧 timer 会检查会话是否已关闭，避免旧会话回调继续操作。
- 日志对比文档：`docs/ANDROID_KEEPALIVE_LOG_ANALYSIS.md` 记录了本项目与 `yxbjv.lmge.gbjrj` 的锁屏保活差异；重点结论是本次日志没显示本项目被系统杀进程，已按对比包方向调整服务通知和悬浮窗透明度。悬浮窗未配置时保持可见常驻，配置为 0 时最低 `alpha=0.01` 且不可触摸。
- UI 状态：主界面权限卡隐藏剪贴板同步和保持屏幕开启，但默认启用；悬浮窗行显示为 `悬浮权限`；侧按钮“开”为蓝色，“关”为红色。
- 构建注意：生成的 `target/.../out/protos/*.rs` 出现空字节并伴随 `.rmeta` invalid metadata 是 Rust 构建缓存损坏；清理构建机 `target`，不要误改 `libs/hbb_common/src/fs.rs`。

## 新增必读

- `docs/ANDROID_STATE_MACHINE.md`
  - Android 服务、视频流、无视截屏流、PC 等待首帧、重连之间的统一状态机。
  - 后续凡是改“锁屏 / 断网 / 关共享 / 开共享 / 等待画面 / 开无视”相关逻辑，先读这份文档。
