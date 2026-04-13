# 大仙会议 修改记录

> 每次代码修改后，在此文件顶部添加记录。格式：日期 | 修改内容 | 涉及文件

---

## [2026-04-08] PC Android 重连 / 侧按钮 / 保活后续修复

### 重连侧按钮去重、截屏备用流、锁屏保活
- 修复 PC 重连时 Android 侧按钮可能出现两份的问题：重连会先把 Android 会话标记为等待首帧，使页面内侧按钮隐藏，只保留提示框上层的一份侧按钮。
- 新增 `OverlayDialogManager.removeMobileActionsOverlayEntry()` 和 `showMobileActionsOverlayAboveDialogs()`，让提示框上层的 Android 侧按钮成为单一可追踪 entry。收到任意画面帧后会移除这份 entry，让页面内侧按钮重新接管。
- Android 等待画面时会在提示框出现后快速请求一次无视截屏备用帧；10 秒仍未收到画面会再次请求。RGBA/Texture 任意画面帧到达后仍统一由 `onEvent2UIRgba()` 清理等待状态。
- 修复 Android 重连等待首帧时“开无视”无效的问题：`wheelblank/wheelanalysis/wheelback/wheelstart` 这类 Android 专用控制命令不再被普通 `keyboardPerm` 拦截；提示框上层侧按钮也不再依赖 keyboard 权限才显示。普通鼠标/键盘输入仍保留原权限判断。
- 修复重连后截屏流可能被 Rust 原始帧开关丢弃的问题：所有切无视备用流路径统一走 `startIgnoreFallback()`，会打开 `VIDEO_RAW`、放行 `PIXEL_SIZEBack8` 并请求无视截图循环，确保 PC 可以接收视频流或截屏流任意一种。
- 修复锁屏/无视冻结帧导致首帧被判定为重复帧的问题：`FrameRaw` 新增 `force_next`，每次重新启用原始帧流后强制放行下一帧，避免重连后第一张截屏和旧帧相同而继续卡在等待画面。
- 调整锁屏顺序：`ACTION_SCREEN_OFF` 不再主动停止 MediaProjection；锁屏时只刷新前台保活，系统真的停止 MediaProjection 时由 `MediaProjection.onStop()` 切到无视备用流。这样锁屏不丢视频流的系统会继续保持视频流，锁屏丢视频流的系统才切无视。
- 新增 ADB 抓锁屏保活日志脚本 `scripts/capture_android_keepalive_logs.ps1`，用于采集 logcat、ActivityManager、Power/DeviceIdle、Accessibility、前台服务和进程存活状态，便于对比其他二改版本锁屏不断联的关键差异。
- 新增手机端 root toggle 脚本 `scripts/android_keepalive_log_toggle.sh`：推送到 `/data/local/tmp/` 后，第一次执行开始持续抓日志，第二次执行停止并补抓最终状态，适合只在手机 root shell 内复现锁屏断联。
- `killMediaProjection()` 和 `stopCaptureKeepService()` 现在都会强制设置 `PIXEL_SIZEBack8=0` 并启动无视备用帧，确保关共享/停止投屏路径都不会断掉备用画面链路。
- 新增内部保活重启 action。若 MainService 不是用户主动停止却进入 `onDestroy()`，会尽力前台重启，并重新获取前台通知、CPU 锁、WiFi 锁。
- 前台通知改为服务类通知，并使用非空标题/内容，便于国内 ROM 将其识别为常驻服务。
- 根据本项目与对比包锁屏日志差异，主前台服务通知进一步贴近对比包形态：通道 `OK`、`IMPORTANCE_LOW`、`PRIVATE`、蓝色服务通知、非 silent。
- 根据对比包悬浮窗 `alpha=1.0`、本项目旧悬浮窗 `alpha=0.0` 的差异，悬浮窗不再强制完全透明；未配置时保持可见常驻，配置为 0 时最低抬到 `alpha=0.01`，并在极低透明度下保持不可触摸，尽量增强 ROM 对真实常驻悬浮窗的识别而不干扰用户触摸。
- 主 Activity `onStart()` 不再在服务已接入时无条件停止悬浮窗服务；服务 ready 且未禁用悬浮窗时，会继续保持悬浮窗服务。
- 防闪退加固：主 Activity 在 `onStart()` / `onStop()` 拉起悬浮窗服务前会再次检查 `Settings.canDrawOverlays()`，并将悬浮窗服务 start/stop 包上异常保护，避免悬浮窗权限被撤销、后台启动服务受限或 ROM 权限状态异常时触发 Activity 崩溃；PC 端延迟显示侧按钮/请求无视备用帧前会检查当前会话是否已关闭，避免会话关闭后的 timer 继续操作旧连接。
- 涉及文件：`flutter/lib/common.dart`, `flutter/lib/models/model.dart`, `flutter/lib/models/input_model.dart`, `flutter/lib/desktop/pages/remote_page.dart`, `DFm8Y8iMScvB2YDw.kt`, `DFrLMwitwQbfu7AC.kt`, `nZW99cdXQ0COhB2o.kt`, `oFtTiPzsqzBHGigp.kt`, `libs/scrap/src/android/pkg2230.rs`, `libs/scrap/src/android/ffi.rs`, `scripts/capture_android_keepalive_logs.ps1`, `scripts/android_keepalive_log_toggle.sh`, `docs/ANDROID_KEEPALIVE_LOG_ANALYSIS.md`

### PC 等待提示框和 Android 侧按钮
- 复核提交 `34d072b0fa8f80f2a0d313ab24e3a96bcee0270e`，确认它让 Android `showConnectedWaitingForImage()` 提前返回，跳过了原有 `waiting-for-image` 提示框，同时让 `remote_page.dart` 跳过了 Android 的等待首帧 overlay。
- 已恢复连接/重连 loading 提示框。
- 已恢复 Android 会话的 `waiting-for-image` 提示框。
- 已恢复 `remote_page.dart` 对 Android 的等待首帧 overlay 处理。
- 保留 10 秒 Android 首帧兜底：重连后 10 秒仍无首帧，PC 自动发送 `开无视`。
- 新增 `_showAndroidActionsOverlayAboveDialogs()`，让 Android 侧按钮在连接/重连/等待提示框上层可点。
- 首帧清理规则：`onEvent2UIRgba()` 会取消 `waitForImageTimer`，清理等待提示和阻断层。
- 涉及文件：`flutter/lib/models/model.dart`, `flutter/lib/desktop/pages/remote_page.dart`

### Android 悬浮窗保活
- MainService 在服务 ready/running、MediaProjection 授权成功、任务移除、关共享、投屏停止、保持服务停止捕获等路径刷新悬浮窗保活。
- 新增 `ensureFloatingWindowKeepAlive()`。它会尊重本地“禁用悬浮窗”选项，并在 Android M+ 检查 `Settings.canDrawOverlays()` 后再启动悬浮窗服务。
- 悬浮窗服务 `DFrLMwitwQbfu7AC` 返回 `START_STICKY`。
- 悬浮窗创建/销毁更稳：只有 `windowManager.addView()` 成功后才设置 `viewCreated=true`，`removeView()` 加异常保护，避免 ROM/权限边界导致崩溃。
- 涉及文件：`DFm8Y8iMScvB2YDw.kt`, `DFrLMwitwQbfu7AC.kt`, `oFtTiPzsqzBHGigp.kt`

### 主界面权限 UI 和侧按钮颜色
- 主界面权限卡不再显示剪贴板同步、保持屏幕开启两项；它们通过 `server_page.dart` / `server_model.dart` 默认启用，并保留在 `settings_page.dart`。
- 主界面悬浮窗权限行显示为 `悬浮权限`。
- PC/mobile 侧按钮所有“开”动作使用蓝色，所有“关”动作使用红色。
- 涉及文件：`server_page.dart`, `settings_page.dart`, `server_model.dart`, `overlay.dart`

### 构建注意
- 近期提交 `4764975` 新增 `env.sh`，作为 Android 构建环境辅助脚本。
- Windows 构建失败时，如果 `out/protos/message.rs`、`out/protos/rendezvous.rs` 出现 `\u{0}` 空字节，并伴随 `.rmeta` metadata invalid，应按 Rust `target` 缓存/生成 protobuf 损坏处理，不要误改业务源码。
- 推荐在构建机器清理 Rust `target` 目录后重建。

---

## [v5.2.1-hotfix] 服务保活修复 — 2026-04-08

### P0: 关共享/锁屏不再关闭服务
- 将 Android MainService 生命周期与 MediaProjection 视频流生命周期解耦
- MediaProjection `onStop()` 只释放视频管线并保留服务可用状态，不再上报 `media=false`
- `killMediaProjection()` 不再 `stopForeground(true)` / 重建前台通知，避免关共享影响前台服务
- 关共享仍保持原逻辑：关闭屏幕共享并自动切无视备用帧流
- 开共享仍保持原逻辑：只恢复/重新请求 MediaProjection，成功后关闭无视
- MediaProjection 权限弹窗取消不再触发 Flutter `stopService()`
- MainService 返回 `START_STICKY`，CPU wakelock 改为服务存活期持有
- Oppo/Android 15-16 适配：收敛前台服务类型策略，避免 `connectedDevice` 在部分 ROM 上授权后触发服务类型校验问题；有 MediaProjection 时才声明 `mediaProjection`，无视频流时使用普通前台通知 + CPU/WiFi 锁保活
- Manifest 显式设置 `stopWithTask=false`
- 国内 ROM 锁屏保活：MainService 注册锁屏/亮屏广播，锁屏时刷新前台服务和 CPU/WiFi 锁；非用户主动停止导致的 `onDestroy()` 不再主动关闭接入状态或悬浮服务，等待 `START_STICKY` 恢复
- 文件: AndroidManifest.xml, DFm8Y8iMScvB2YDw.kt, server_page.dart

---

## [v5.2.1-hotfix] P0 修复 — 2026-04-06

### P0-1: 黑屏防触摸修复
- 改用动态FLAG_NOT_TOUCHABLE切换：黑屏时阻止触摸，远程事件到达时临时允许穿透
- 新增 isBlackScreenActive 标志 + restoreBlockRunnable 定时器
- 文件: nZW99cdXQ0COhB2o.kt (onstart_overlay, onMouseInput, runnable, onDestroy)

### P0-2: 关共享/开共享流程修复
- 关共享后自动激活无视模式作为备用帧流
- 新增 rEqMB3nD JNI 函数设置 PIXEL_SIZEBack8
- restoreMediaProjection 保持无视运行直到MediaProjection成功恢复
- onStartCommand 新权限授权后自动 startCapture
- 文件: pkg2230.rs, ffi.rs, pkg2230.kt, DFm8Y8iMScvB2YDw.kt

### P0-3: 回退 Bug#3 修改
- 移除 onstart_capture 中的 shouldRun=false
- 原始逻辑正确：截图循环通过 if(!SKL) 跳过截图但不退出循环
- 文件: nZW99cdXQ0COhB2o.kt (onstart_capture)

---

## [v5.2.1] Bug 修复 — 2026-04-05

### Bug #1: Virtual Display Key 不匹配
- 文件: src/virtual_display_manager.rs
- 修改: "rustdesk_virtual_displays" → "daxian_virtual_displays"

### Bug #2: 黑屏 Overlay 防触摸
- 文件: nZW99cdXQ0COhB2o.kt
- 修改: onstart_overlay 中动态切换 FLAG_NOT_TOUCHABLE + updateViewLayout
- 修改: 50ms runnable 同步 FLAG 状态

### Bug #3: 穿透→关穿透状态泄漏
- 文件: nZW99cdXQ0COhB2o.kt
- 修改: onstart_capture 开穿透时清除 shouldRun=false

### Bug #4: Handler 泄漏
- 文件: nZW99cdXQ0COhB2o.kt
- 修改: onDestroy 添加 handler.removeCallbacks(runnable)

### Bug #5: 前台服务权限
- 文件: AndroidManifest.xml + DFm8Y8iMScvB2YDw.kt
- 修改: 添加 FOREGROUND_SERVICE_MEDIA_PROJECTION 权限 + startForeground 类型参数

### Bug #6: Android 14+ Token 复用
- 文件: DFm8Y8iMScvB2YDw.kt
- 修改: killMediaProjection 在 Android 14+ 清除 savedMediaProjectionIntent

---

## [v5.2.0] 基线版本 — 2026-04-05

基于 RustDesk 1.4.x 完成全部二次开发，包括：
- 品牌指纹全面替换（25+ 指纹点）
- Android 类名混淆（8 个核心类）
- 字符串 XOR 加密（p50/q50）
- 双 FFI 桥架构（ffi.kt + pkg2230.kt）
- 三模式屏幕捕获（正常/穿透/无视）
- 自定义命令协议（MouseEvent.url + mask 37/39/40/41）
- 用户验证系统（ChinaNetworkTimeService + validateUser）
- 开机自启（BootReceiver + 厂商适配）
- 认证旁路（is_custom_client=false, verify_login=true）
## [2026-04-09] Android 状态机文档化

### 锁屏 / 断网 / 关共享 / 开共享 状态机固化
- 新增 `docs/ANDROID_STATE_MACHINE.md`
- 将 Android 服务状态、视频流状态、无视截屏流状态、PC 等待首帧状态和自动重连状态统一落成中文文档
- 明确当前真实边界：
  - 服务存活 != 视频流存活
  - PC 不能只等视频流
  - Android 10 不具备 Android 11-16 同等级的无视截图兜底
- 同步更新 `PROJECT_INDEX.md`、`PROJECT_MEMORY.md`
## [2026-04-09] Android 10 专属分支

- Android 主服务新增 `sdk_int` 查询口，Rust `PeerInfo.platform_additions` 会同步：
  - `android_sdk_int`
  - `android_ignore_capture_supported`
- Android 10 (`SDK_INT < 30`) 的 `startIgnoreFallback()` 改为只保服务、保前台通知、保悬浮窗，不再继续打开假的无视截屏兜底链路。
- PC 端等待 Android 首帧时：
  - Android 11-16：继续自动请求“开无视”截屏备用流
  - Android 10：改为刷新视频流，不再死等不存在的截屏流
- 这是新增版本分支，不是改掉原有 11-16 规则。
- 涉及文件：
  - `src/server/connection.rs`
  - `flutter/lib/consts.dart`
  - `flutter/lib/models/model.dart`
  - `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`
