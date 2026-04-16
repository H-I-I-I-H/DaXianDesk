# Changelog

## [v5.2.1-hotfix-2] 开黑屏防触摸 Bug 修复 — 2026-04-14

### P0: 移除错误的动态 FLAG_NOT_TOUCHABLE 切换逻辑

- 删除 `isBlackScreenActive` / `restoreBlockRunnable` / `setOverlayTouchBlock` 三件套
- `onMouseInput` 入口不再向主线程 `handler` 提交任务，高频远程输入不再阻塞主线程
- `onstart_overlay` 和 50ms 轮询 `runnable` 只同步 `overlay.visibility`，不再操作 flags
- `onDestroy` 同步移除已删除的 `restoreBlockRunnable` 引用
- 根本原因：防触摸逻辑 FLAG 语义反转 + 每帧 `updateViewLayout` + 50ms 轮询三重问题
- 正确机制：远程输入使用 `AccessibilityService.dispatchGesture`，不经过 overlay，无需动态切换 flag
- 文件: `nZW99cdXQ0COhB2o.kt`
