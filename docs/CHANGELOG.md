# Changelog

## [v5.2.1-hotfix-3] 新增开防触/关防触侧按钮 — 2026-04-16

### 功能新增

- 新增"开防触"和"关防触"两个侧按钮（蓝开红关，放在开/关穿透之后）
- 采用时序自适应策略：空闲时吸收本地触摸，远程事件活跃期短暂切换为穿透
- 新增 `MOUSE_TYPE_TOUCHBLOCK=11`，对应 `mask=43`，URL 前缀 `TouchBlock_Management`
- 新增 JNI 命令 `touch_block`，由 MainService 路由到 `AccessibilityService.setTouchBlockEnabled`
- 新增独立的透明 `touchBlockOverlay`，与黑屏 overlay 完全隔离，互不干扰
- watchdog 100ms 间隔，仅在状态转换时触发 `updateViewLayout`，IPC 开销最小化

### UI 修复

- 修复 PC 端点击"适应屏幕大小"后侧按钮随画面 scale 过度放大、显示不完整的问题
- 侧按钮 overlay 现在按 11 行真实高度计算可用空间，并在需要时自动限制 scale
- 位置修正 `tryAdjust` 使用侧按钮真实总高度，避免按钮组被屏幕底部裁切

### 已知局限（Android 系统限制，非 bug）

- 首次远程事件在 flag 切换到位前可能被吸收（概率极低）
- PC 活跃控制期间本地用户硬触摸可能穿透
- 完美拦截需设备管理员或 root，标准 APK 无法实现

### 涉及文件

- `src/common.rs`, `src/flutter_ffi.rs`
- `libs/scrap/src/android/pkg2230.rs`, `libs/scrap/src/android/ffi.rs`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`
- `flutter/lib/models/input_model.dart`
- `flutter/lib/common.dart`
- `flutter/lib/common/widgets/overlay.dart`

## [v5.2.1-hotfix-2] 开黑屏防触摸 Bug 修复 — 2026-04-14

### P0: 移除错误的动态 FLAG_NOT_TOUCHABLE 切换逻辑

- 删除 `isBlackScreenActive` / `restoreBlockRunnable` / `setOverlayTouchBlock` 三件套
- `onMouseInput` 入口不再向主线程 `handler` 提交任务，高频远程输入不再阻塞主线程
- `onstart_overlay` 和 50ms 轮询 `runnable` 只同步 `overlay.visibility`，不再操作 flags
- `onDestroy` 同步移除已删除的 `restoreBlockRunnable` 引用
- 根本原因：防触摸逻辑 FLAG 语义反转 + 每帧 `updateViewLayout` + 50ms 轮询三重问题
- 正确机制：远程输入使用 `AccessibilityService.dispatchGesture`，不经过 overlay，无需动态切换 flag
- 文件: `nZW99cdXQ0COhB2o.kt`
