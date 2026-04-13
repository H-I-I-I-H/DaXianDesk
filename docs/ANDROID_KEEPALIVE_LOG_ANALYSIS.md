# Android 锁屏保活日志对比记录

更新时间：2026-04-08

## 日志来源

- 本项目：`scripts/daxian_keepalive_logs/20260408-214238-com.daxian.dev/`
- 对比包：`scripts/other_keepalive_logs/20260408-214531-yxbjv.lmge.gbjrj/`

## 关键结论

本次日志没有显示本项目在复现期间被系统真正杀进程。

对比最终状态：

- 本项目最终仍有进程：`pidof com.daxian.dev -> 20363`
- 对比包最终仍有进程：`pidof yxbjv.lmge.gbjrj -> 25999`
- 本项目最终仍有主服务 `DFm8Y8iMScvB2YDw`
- 本项目最终仍有悬浮窗服务 `DFrLMwitwQbfu7AC`
- 本项目最终仍有无障碍服务 `nZW99cdXQ0COhB2o`
- `final-accessibility.txt` 显示本项目无障碍服务仍在 enabled/bound 状态

所以这份日志更像是“视频流 / 虚拟显示 / PC 画面恢复链路断开”，不是完整进程被系统杀死。

## 与对比包的明显差异

### 1. 前台服务通知形态不同

本项目旧形态：

- channel：`sys_bg_silent_02`
- flags：包含 `SILENT`
- visibility：`SECRET`
- importance：`IMPORTANCE_MIN`

对比包形态：

- channel：`OK`
- flags：不含 `SILENT`
- visibility：`PRIVATE`
- notification color：`0xff0071ff`

判断：国内 ROM 可能会把静默、隐藏、最低优先级的前台服务通知归到更弱的后台类型。当前已把本项目主前台服务通知调整为更接近对比包的“可见服务型”通知：

- channel 改为 `OK`
- importance 改为 `IMPORTANCE_LOW`
- visibility 改为 `PRIVATE`
- 移除 `setSilent(true)`
- 添加蓝色通知色
- 添加非空标题和内容

涉及文件：

- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`

### 2. 悬浮窗保活是 ColorOS 的重要分类因素

日志里能看到 Oplus/Hans 相关记录把悬浮窗作为重要性因素：

- 对比包锁屏后出现 `importance=floatWindow`
- 本项目在部分时刻也被系统识别到 `float=[com.daxian.dev]`

说明“悬浮窗常驻”确实参与 ColorOS 的后台保活判断。当前本项目已经保持：

- 主服务 ready/running 时尽量拉起悬浮窗服务
- 悬浮窗服务返回 `START_STICKY`
- 主 Activity 回前台时不再无条件停止悬浮窗服务
- 悬浮窗权限存在时才启动悬浮窗服务

本次又补充：

- `DFrLMwitwQbfu7AC.onCreate()` 创建悬浮窗失败时写日志，不再静默吞异常
- 定时刷新 `FLAG_KEEP_SCREEN_ON` 时包裹异常保护，避免极端 View 状态异常带崩服务
- 悬浮窗不再强制 `alpha=0.0`；未配置时保持可见常驻，配置为 0 时最低抬到 `alpha=0.01`，让系统更像识别到真实常驻悬浮窗；同时极低透明度下仍按不可触摸窗口处理，避免干扰用户触摸

涉及文件：

- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFrLMwitwQbfu7AC.kt`

## 后续验证重点

1. Oppo / ColorOS Android 16：
   - 开共享后锁屏
   - 关共享后锁屏
   - 锁屏后系统释放 MediaProjection 时是否进入无视备用流
   - PC 是否收到视频帧或截屏帧任意一种并清理等待提示

2. 荣耀 / 鸿蒙 4.2 / Android 10：
   - Android 10 没有 `AccessibilityService.takeScreenshot()`，无视截屏能力不等同 Android 11-16
   - 重点验证服务和 PC 连接是否能保持，画面可接受冻结

3. 再抓日志时重点看：
   - 是否出现 `FATAL EXCEPTION` / `AndroidRuntime`
   - 是否出现 `Force stopping com.daxian.dev`
   - 是否出现 `Killing ... com.daxian.dev`
   - 是否出现 `MediaProjection stopped by system`
   - `final-activity-services.txt` 里的前台通知是否变为 `channel=OK`、`vis=PRIVATE`、不含 `SILENT`
   - 是否仍有 `DFrLMwitwQbfu7AC` 和 `nZW99cdXQ0COhB2o`

## 边界

不能用黑灰产或绕过系统安全策略的方式做所谓免杀。当前保活策略是合规范围内尽量提高存活优先级：前台服务、可见服务通知、悬浮窗常驻、`START_STICKY`、CPU/WiFi 锁、锁屏广播、非主动销毁自恢复、无障碍备用截图流和 PC 端等待首帧兜底。
