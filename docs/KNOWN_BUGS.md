# 已知 Bug 列表

| 序号 | 问题 | 严重性 | 位置 | 状态 |
|------|------|--------|------|------|
| 1 | Virtual Display key 不匹配：Rust 发 "rustdesk_virtual_displays"，Dart 期望 "daxian_virtual_displays" | 高 | virtual_display_manager.rs vs consts.dart | **已修复 v5.2.1** |
| 2 | PIXEL_SIZE 系列使用 static mut，线程安全隐患 | 中 | pkg2230.rs | 待评估 |
| 3 | ffi.rs 是 pkg2230.rs 冗余备份，修改需双份同步 | 中 | libs/scrap/src/android/ | 待决策 |
| 4 | targetSdkVersion=33，低于 Google Play 要求的 34+ | 中 | build.gradle | 待修复 |
| 5 | Windows DLL 名仍为 librustdesk.dll 未改名 | 低 | main.cpp, native_model.dart | 待决策 |
| 6 | verify_rustdesk_password_tip 翻译 key 残留 RustDesk | 低 | dialog.dart | 待修复 |

## 新增发现并修复的 Bug（v5.2.1）

| 序号 | 问题 | 严重性 | 位置 | 状态 |
|------|------|--------|------|------|
| 7 | 黑屏 Overlay 无法阻止触摸（缺少 FLAG_NOT_TOUCHABLE 动态切换） | 高 | nZW99cdXQ0COhB2o.kt | **已修复 v5.2.1-hotfix** (动态切换+远程穿透) |
| 8 | 穿透→关穿透导致无视模式意外恢复（shouldRun 未清除） | 高 | nZW99cdXQ0COhB2o.kt | **回退修复** (原始逻辑正确，v5.2.1的shouldRun=false已移除) |
| 9 | InputService Handler 泄漏（onDestroy 未停止 runnable） | 中 | nZW99cdXQ0COhB2o.kt | **已修复 v5.2.1** |
| 10 | 缺少 FOREGROUND_SERVICE_MEDIA_PROJECTION 权限（Android 14+ 崩溃） | 高 | AndroidManifest.xml + DFm8Y8iMScvB2YDw.kt | **已修复 v5.2.1** |
| 11 | Android 14+ MediaProjection Token 不可复用（恢复共享延迟） | 中 | DFm8Y8iMScvB2YDw.kt | **已修复 v5.2.1** |
| 12 | 关共享后画面冻结（MediaProjection销毁后无备用帧流） | P0 | DFm8Y8iMScvB2YDw.kt | **已修复 v5.2.1-hotfix** (关共享自动激活无视) |
| 13 | 开共享需App在前台（后台Service无法启动授权Activity） | P0 | DFm8Y8iMScvB2YDw.kt | **已修复 v5.2.1-hotfix** (无视保持+自动startCapture) |
