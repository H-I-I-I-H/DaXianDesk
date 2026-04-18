# Android 运行时工程文档 / Android Runtime Engineering Notes

最后一次从全仓源码核验：2026-04-18

> 本文件记录的是**当前代码真正体现出来的 Android 运行时模型**。
> 中文用于解释状态和风险；English symbol / path 用于把结论牢牢钉回源码。
> 若与代码冲突，以代码为准，并同步更新本文件。

---

## 0. 最近运行时修复（Recent Runtime Fix）

### 0.1 黑屏 overlay 不再动态切换触摸 flag

2026-04-16 已按源码修复 `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`：

- 删除 `isBlackScreenActive` / `restoreBlockRunnable` / `setOverlayTouchBlock` 三件套。
- `onMouseInput(...)` 不再因黑屏状态向主线程 `handler` 提交 per-mouse-event 任务。
- `onstart_overlay(...)` 只负责把 `gohome` 同步成 `overLay.visibility`，不再调用 `WindowManager.updateViewLayout(...)`。
- 50ms `runnable` 只做 `gohome -> overLay.visibility` 的最终一致性同步，并继续维护 `BIS = overLay.visibility != View.GONE`。
- `onDestroy()` 只移除 50ms 轮询 `runnable`，不存在已删除的 `restoreBlockRunnable`。

防回归规则：

- 不要在黑屏 `overLay` 上恢复任何 `FLAG_NOT_TOUCHABLE` 动态切换。
- 不要在 `onMouseInput(...)` 中按每个鼠标事件直接或重复触发 `WindowManager.updateViewLayout(...)`。
- 远程输入走 `AccessibilityService.dispatchGesture()`，不依赖 overlay 的触摸分发层；黑屏 overlay 应保持为纯显示/隐藏能力。
- `pkg2230.rs` 内的 `PIXEL_SIZE*` 视觉/像素逻辑与本次修复无关，不要为了黑屏输入卡顿问题联动修改。

### 0.2 touchBlockOverlay 是独立防触摸层

2026-04-17 已新增"开防触/关防触"侧按钮链路。它与黑屏 overlay 是两套状态：

- Flutter 侧按钮发送 `wheeltouch`，Rust 映射到 `MOUSE_TYPE_TOUCHBLOCK=11`，最终形成 `mask=43`。
- Android Rust 分发层只接受 `TouchBlock_Management` 前缀，并调用 JNI 命令 `touch_block`。
- `DFm8Y8iMScvB2YDwSBN("touch_block", arg1, ...)` 路由到 `nZW99cdXQ0COhB2o.ctx?.setTouchBlockEnabled(arg1 == "1")`。
- `touchBlockOverlay` 是透明的 `TYPE_ACCESSIBILITY_OVERLAY`，空闲时移除 `FLAG_NOT_TOUCHABLE` 来吸收本地触摸。
- 远程输入活跃时设置 `FLAG_NOT_TOUCHABLE` 短暂穿透，活跃窗口为 500ms；watchdog 每 100ms 检查，仅状态变化时调用 `updateViewLayout(...)`。
- `onMouseInput(...)` / `onTouchInput(...)` 只标记远程活动并在必要时投递一次穿透切换，不能恢复旧版每帧 IPC 风暴。

已知边界：

- 这是标准 APK 下的时序近似防触，不是系统级 100% 物理隔离。
- 第一次远程事件可能因 flag 尚未切换而被吸收；PC 活跃窗口内本地触摸可能穿透。
- 完美本地触摸隔离需要设备管理员、root、OEM API 或系统签名能力。

### 0.3 Android 状态监测推送链路

2026-04-18 已新增 Android 被控端状态 JSON 聚合与 PC 端监测面板：

- Android 查询键：`DFm8Y8iMScvB2YDwGYN("daxian_status")`。
- JSON 字段：`video` / `screenshot` / `share` / `ignore` / `blank` / `penetrate` / `touchblock`。
- 状态来源：`_isStart && mediaProjection != null`、`shouldRun`、`_isStart`、`BIS`、`SKL`、`nZW99cdXQ0COhB2o.isTouchBlockOn`。
- Android server 每秒在 `src/server/connection.rs` 的 `second_timer.tick()` 内发送 `Misc.daxian_status`。
- PC 端 `src/client/io_loop.rs` 接收 `misc::Union::DaxianStatus(json)` 后推送 Flutter 事件 `update_daxian_status`。
- Flutter 端 `DaxianStatusModel` 解析 JSON，`DaxianStatusMonitor` 与 `QualityMonitor` 通过 `RemoteStatusMonitors` 右上角竖排显示。

---

## 1. 核心原则（Core Runtime Principles）

这个项目的 Android 运行时至少有三层状态，不能压成一个“开/关”：

1. Android 服务状态（`service lifecycle state`）
2. Android 帧源状态（`frame source state`）
3. PC 端首帧等待状态（`waiting-for-first-frame state`）

必须记住：

- 服务存活 != `MediaProjection` 存活
- `MediaProjection` 丢失 != 应用停止
- PC 端 waiting 状态必须在收到**任何真实首帧**时清除，而不只是正常视频路径的首帧

关键锚点：

- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/common.kt`
- `flutter/lib/models/model.dart`
- `libs/scrap/src/android/pkg2230.rs`

---

## 2. 组件角色（Component Roles）

### 2.1 `oFtTiPzsqzBHGigp.kt`

角色：主 `FlutterActivity`

职责：

- 作为 Android 入口 Activity
- 权限检查 / channel 入口
- overlay 权限相关辅助

### 2.2 `DFm8Y8iMScvB2YDw.kt`

角色：`MainService`

职责：

- `MediaProjection`
- `ImageReader`
- 正常视频采集
- projection stop / restore / keep-alive
- 前台通知
- overlay keep-alive 刷新

### 2.3 `nZW99cdXQ0COhB2o.kt`

角色：`AccessibilityService`

职责：

- 输入注入
- 节点遍历 / overlay 行为
- `SKL` 路径
- `shouldRun` 路径
- Android 11+ screenshot fallback

### 2.4 `DFrLMwitwQbfu7AC.kt`

角色：`FloatWindowService`

职责：

- 浮窗保活
- 返回 `START_STICKY`

### 2.5 `pkg2230.rs`

角色：主 Android Rust JNI bridge

职责：

- Android raw frame 桥接
- `VIDEO_RAW`
- `PIXEL_SIZEBack` / `PIXEL_SIZEBack8`
- `FrameRaw.force_next`
- 命令分发到 Kotlin 层

### 2.6 `common.kt`

角色：Android 全局状态面板

关键变量：

- `SKL`
- `shouldRun`
- `gohome`
- `BIS`
- `SCREEN_INFO`

---

## 3. Android 服务状态（Android Service States）

可用如下脑内模型：

- `A0 service stopped`
  - `MainService` 未运行
- `A1 service alive, no active MediaProjection stream`
  - 服务活着
  - projection 未产出正常视频
- `A2 service alive, MediaProjection stream active`
  - 正常共享路径在工作
- `A3 service alive, ignore fallback active or being requested`
  - projection 不可用
  - 正在 fallback 或等待 fallback

关键锚点：

- `DFm8Y8iMScvB2YDw.kt`
  - `_isReady`
  - `_isStart`
  - `mediaProjection`
  - `savedMediaProjectionIntent`
  - `killMediaProjection()`
  - `handleProjectionStoppedKeepService()`
  - `restoreMediaProjection()`
  - `startIgnoreFallback()`

源码事实：

- `killMediaProjection()` 释放 projection / virtualDisplay / imageReader 等资源，但并不会把整个服务视为已完全退出。
- `handleProjectionStoppedKeepService()` 的含义是**保持服务存活，切换到 fallback/保活语义**。
- `ACT_KEEP_ALIVE_SERVICE` 是一个真实存在的 service action。
- `onStartCommand()` 在多个分支会返回 `START_STICKY`，目的是尽量维持服务。

---

## 4. Android 帧源状态（Frame Source States）

### 4.1 正常共享路径（normal MediaProjection video path）

锚点：

- `DFm8Y8iMScvB2YDw.kt`
- `ImageReader`
- `libs/scrap/src/android/pkg2230.rs`
- `VIDEO_RAW`

行为：

- `MediaProjection` + `ImageReader` 产出正常共享帧
- 帧被送入 raw 输出路径
- 只要主路径稳定，PC 端通常进入正常远控画面

### 4.2 穿透路径（SKL pass-through path）

锚点：

- `common.kt::SKL`
- `nZW99cdXQ0COhB2o.kt`
- `pkg2230.rs::PIXEL_SIZEBack`

行为：

- 受 `SKL` 控制
- 更偏向 Accessibility / 穿透语义
- 与正常 `MediaProjection` 路径不是同一个状态量

### 4.3 无视回退路径（ignore-capture fallback path）

锚点：

- `common.kt::shouldRun`
- `nZW99cdXQ0COhB2o.kt`
- `pkg2230.rs::PIXEL_SIZEBack8`
- `pkg2230.rs::VIDEO_RAW`

行为：

- projection 不可用或等待恢复时，fallback 可能开启
- Android 11+ 可走 screenshot fallback
- Android 10 明确不承诺 screenshot fallback

---

## 5. PC 端显示状态（PC-Side Display States）

PC 不应与 Android 服务状态混为一谈：

- `P0 disconnected`
- `P1 connected but waiting for first frame`
- `P2 connected and receiving normal video`
- `P3 connected and receiving ignore fallback frame`
- `P4 reconnecting`

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

- waiting 状态是 Flutter/PC 自己的显示状态机。
- 即使 Android 服务活着，PC 仍可能处于 `P1 waiting`。
- 一旦任何真实 RGBA 帧到 UI，`onEvent2UIRgba()` 会清除 waiting。

---

## 6. 关键事件规则（Key Event Rules）

### 6.1 打开分享（Open Share / `restoreMediaProjection`）

当前行为：

1. `restoreMediaProjection()` 启动恢复流程
2. 在恢复成功前，不会先把 ignore fallback 关掉
3. 如果 `savedMediaProjectionIntent` 仍有效，则尝试直接恢复 projection
4. 真正恢复成功后，才会：
   - 停止 ignore capture
   - `PIXEL_SIZEBack8 = 255`
   - 恢复正常共享路径

不得回归：

- 不要在 `MediaProjection` 真恢复前提前禁用 ignore fallback
- 不要把“尝试恢复”误当成“已恢复”

### 6.2 关闭分享 / projection stopped

当前行为：

1. 释放 projection 资源
2. 服务语义保持“活着 / ready”
3. 调用 `startIgnoreFallback()`
4. 刷新前台通知
5. 重新确保 overlay keep-alive

不得回归：

- 不要把 close-share 重新改成 service destroy
- 不要把 projection stop 解释为整个 Android 端彻底停止

### 6.3 熄屏（Screen Off）

当前行为：

- 项目不会因为熄屏就主动停止服务
- 若系统后续真的停止 projection，则从 projection-stopped 路径进入 fallback 语义

不得回归：

- 不要重新引入“熄屏必停服务”的逻辑
- 不要把系统行为与项目主动策略混为一谈

### 6.4 waiting-for-first-frame

当前 Flutter 行为：

1. 显示 waiting dialog
2. 将 Android 操作 overlay 提到对话框上方
3. 立即请求备用帧路径：
   - 支持 ignore capture 时请求 ignore fallback
   - 否则请求 video refresh
4. 若仍无首帧，定时器继续补发 fallback 请求
5. 任何真实 RGBA 帧到达 `onEvent2UIRgba()` 时清理等待状态

不得回归：

- 不要只等待“正常视频首帧”
- 不要让 waiting dialog 遮住 Android 操作按钮

---

## 7. Android 版本边界（Android Version Boundary）

关键上报：

- `android_sdk_int`
- `android_ignore_capture_supported`

填充位置：

- `src/server/connection.rs`

运行时落地点：

- `DFm8Y8iMScvB2YDw.kt::startIgnoreFallback()`

源码事实：

- `sdk_int >= 30`：认为具备 ignore-capture fallback 能力
- `sdk_int < 30`：Android 10 仅保持服务存活，不伪装为支持 screenshot fallback

不得回归：

- 不要让 Android 10 看起来和 Android 11+ 拥有相同 fallback 保证
- 不要在 UI 层丢掉这个平台差异

---

## 8. JNI / 原始帧守卫（JNI and Raw Frame Guards）

关键锚点：

- `libs/scrap/src/android/pkg2230.rs`
- `libs/scrap/src/android/ffi.rs`

### 8.1 `VIDEO_RAW`

含义：

- raw frame 通道
- 在 fallback 模式希望继续送帧时，必须正确启用

### 8.2 `PIXEL_SIZEBack`

含义：

- 与 `SKL` 路径守门相关

### 8.3 `PIXEL_SIZEBack8`

含义：

- ignore frame 守门
- `0`：允许 ignore frame 通过
- `255`：阻断 ignore frame

### 8.4 `FrameRaw.force_next`

含义：

- 恢复后的第一帧即使与前一帧重复，也必须强制送出

不得回归：

- 修改帧路径后，必须同时检查：
  - `VIDEO_RAW`
  - `PIXEL_SIZEBack`
  - `PIXEL_SIZEBack8`
  - `force_next`
- 当前主模块是 `pkg2230.rs`，但 `ffi.rs` 中仍有相似逻辑，不能让两者无意识漂移

---

## 9. keep-alive / overlay / notification 现实（Keep-Alive Reality）

已核事实：

- manifest 存在 `android.permission.FOREGROUND_SERVICE_MEDIA_PROJECTION`
- `MainService` 声明了 `android:foregroundServiceType="mediaProjection"`
- 主服务里存在 `ACT_KEEP_ALIVE_SERVICE`
- overlay keep-alive 受 `Settings.canDrawOverlays(...)` 权限门控
- `FloatWindowService` 返回 `START_STICKY`

结论：

- 当前代码目标是**在 Android 限制下尽可能保持服务可恢复**，不是承诺任何 OEM 场景下绝对不被杀进程
- 因此不要把“尽力保活”写成“必然永生”

---

## 10. Android 自定义命令与运行时的关系（Command-to-Runtime Relationship）

真实链路：

- `overlay.dart` / `input_model.dart`
- `src/flutter_ffi.rs`
- `src/ui_session_interface.rs`
- `src/client.rs`
- `src/server/connection.rs`
- `pkg2230.rs`
- Kotlin services

这意味着任何 Android 运行时改动，都可能跨到：

- UI 按钮可见性
- 命令 type / url 编码
- server 消息分发
- JNI bridge 分支
- Kotlin service 行为

因此运行时任务必须按**跨层任务**处理，而不是只改 Kotlin。

---

## 11. Android 相关但原文档易漏的辅助面（Often-Missed Android Surfaces）

### 11.1 `BootReceiver.kt`

- 负责开机启动路径
- 受 `start_on_boot` 和权限条件约束

### 11.2 `ig2xH1U3RDNsb7CS.kt`

- 剪贴板同步桥
- 会把 text / HTML 封装为 protobuf `MultiClipboards`

### 11.3 `KeyboardKeyEventMapper.kt`

- 键盘事件映射

### 11.4 `VolumeController.kt`

- 音量控制辅助

### 11.5 `XerQvgpGBzr8FDFr.kt`

- 权限 / 透明 Activity

这些文件不一定出现在每次 Android 任务的第一批入口中，但一旦问题涉及权限、输入、系统交互、辅助能力，就必须纳入排查。

---

## 12. 防回归清单（Regression Checklist）

在提交任何 Android 运行时改动前，至少重新确认：

1. 是否把服务状态和 projection / frame 状态重新绑死了？
2. 是否把 PC waiting 状态错误地只绑定到正常视频帧？
3. waiting dialog 是否又遮住了 Android 操作按钮？
4. Android 10 是否被错误宣传为支持 screenshot fallback？
5. projection 丢失后是否还会刷新 notification / overlay keep-alive？
6. `force_next` / `VIDEO_RAW` / `PIXEL_SIZEBack8` 是否仍符合当前恢复语义？
7. 是否忘了 `pkg2230.rs` 与 `ffi.rs` 的同步风险？
8. 是否改坏了 `savedMediaProjectionIntent` 的使用与清理？
9. 是否把 “close-share” 错写回 “stop service”？
10. 是否只改了 Kotlin 而没同步检查 Flutter / Rust / server 侧？

---

## 13. 文档同步写法（How Future Agents Should Sync This Doc）

后续 agent 更新本文件时必须继续采用：

- 中文解释运行时意义
- 英文保留真实 symbol / path / action / permission 名
- 每条关键结论至少给一个代码锚点
- 不得把“推测”“一次测试结论”写成常态真相
