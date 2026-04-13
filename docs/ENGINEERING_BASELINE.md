# Engineering Baseline

Last verified from source: 2026-04-14

This file records code-verified facts only. If anything here conflicts with source code, source code wins and this file must be updated.

## 1. Project Identity

- Upstream base: RustDesk, heavily customized
- Cargo package: `rustdesk`
- Cargo version: `5.2.0`
- Rust library name: `librustdesk`
- Flutter package: `flutter_hbb`
- Flutter version: `5.2.0+58`
- Product name: `DaxianMeeting`
- Android package: `com.daxian.dev`
- Android display name: `大仙会议`
- Runtime app name: `DaxianMeeting`
- Runtime org still present in config: `com.carriez`

Verified in:

- `Cargo.toml`
- `flutter/pubspec.yaml`
- `flutter/android/app/build.gradle`
- `flutter/android/app/src/main/AndroidManifest.xml`
- `libs/hbb_common/src/config.rs`

## 2. Real Top-Level Architecture

### Rust core

- Entry and module graph:
  - `src/main.rs`
  - `src/lib.rs`
  - `src/core_main.rs`
- Host/server side:
  - `src/server.rs`
  - `src/server/connection.rs`
  - `src/server/video_service.rs`
  - `src/server/audio_service.rs`
  - `src/server/display_service.rs`
  - `src/server/input_service.rs`
  - `src/server/terminal_service.rs`
- Client/session side:
  - `src/client.rs`
  - `src/ui_session_interface.rs`
  - `src/flutter.rs`
  - `src/flutter_ffi.rs`
- Shared/runtime:
  - `src/common.rs`
  - `libs/hbb_common/src/config.rs`
  - `libs/hbb_common/protos/message.proto`

### Flutter

- App entry:
  - `flutter/lib/main.dart`
- Shared desktop/mobile state and UI bridge:
  - `flutter/lib/common.dart`
  - `flutter/lib/models/`
- Desktop pages:
  - `flutter/lib/desktop/pages/`
- Mobile pages:
  - `flutter/lib/mobile/pages/`
- Plugin UI:
  - `flutter/lib/plugin/`

### Android-native custom layer

- Rust JNI active module:
  - `libs/scrap/src/android/pkg2230.rs`
- Rust JNI legacy compatibility module:
  - `libs/scrap/src/android/ffi.rs`
- Kotlin bridge objects:
  - `flutter/android/app/src/main/kotlin/pkg2230.kt`
  - `flutter/android/app/src/main/kotlin/ffi.kt`
- Main Android services:
  - `DFm8Y8iMScvB2YDw.kt` = MainService / MediaProjection / keep-alive core
  - `nZW99cdXQ0COhB2o.kt` = AccessibilityService / input / screenshot / overlay control
  - `DFrLMwitwQbfu7AC.kt` = floating window service
  - `BootReceiver.kt` = boot startup
  - `common.kt` = Android global state

## 3. Verified Runtime Chains

### 3.1 Desktop/mobile launch

- `flutter/lib/main.dart` decides:
  - desktop main window
  - multi-window desktop sessions
  - connection manager
  - install page
  - mobile app
- `src/core_main.rs` handles desktop startup branching:
  - tray
  - server
  - service
  - install
  - connection manager
  - plugin hooks when feature-enabled

### 3.2 Custom Android control pipeline

This is one of the most important project-specific chains.

Pipeline:

1. Flutter UI buttons in `flutter/lib/common/widgets/overlay.dart`
2. Callback wiring in `flutter/lib/common.dart`
3. Command encoding in `flutter/lib/models/input_model.dart`
4. Rust FFI parsing in `src/flutter_ffi.rs`
5. Session dispatch in `src/ui_session_interface.rs`
6. Protocol message build in `src/client.rs`
7. `MouseEvent { mask, x, y, url }` in `libs/hbb_common/protos/message.proto`
8. Receive and dispatch in `src/server/connection.rs`
9. JNI bridge in `libs/scrap/src/android/pkg2230.rs`
10. Kotlin service handling in `DFm8Y8iMScvB2YDw.kt` and `nZW99cdXQ0COhB2o.kt`

The custom mouse types are real:

- `MOUSE_TYPE_BLANK = 5`
- `MOUSE_TYPE_BROWSER = 6`
- `MOUSE_TYPE_Analysis = 7`
- `MOUSE_TYPE_GoBack = 8`
- `MOUSE_TYPE_START = 9`
- `MOUSE_TYPE_STOP = 10`

The custom Dart command strings are real:

- `wheelblank`
- `wheelbrowser`
- `wheelanalysis`
- `wheelback`
- `wheelstart`
- `wheelstop`

### 3.3 Android capture modes

There are three practical capture paths:

- Normal video path
  - MediaProjection / ImageReader driven
- Penetration path
  - controlled by `SKL`
- Ignore/screenshot fallback path
  - controlled by `shouldRun`

Important Android global state:

- `SKL`
- `shouldRun`
- `gohome`
- `BIS`
- `SCREEN_INFO`

Important Rust JNI state:

- `VIDEO_RAW`
- `JVM`
- `MAIN_SERVICE_CTX`
- `PIXEL_SIZEBack`
- `PIXEL_SIZEBack8`
- `PIXEL_SIZE4..8`

### 3.4 Android runtime invariants

These are source-verified rules, not doc assumptions:

- Android service state and Android frame state are not the same thing.
- `killMediaProjection()` and projection-stop handling release capture resources, but keep the service in a ready state and move to fallback logic instead of treating the app as stopped.
- `restoreMediaProjection()` keeps ignore fallback running until MediaProjection is actually restored; only then does it stop ignore capture and block ignore frames with `PIXEL_SIZEBack8=255`.
- Android 10 and Android 11+ are intentionally different:
  - Android 11+ can request accessibility screenshot fallback
  - Android 10 keeps the service alive but does not pretend screenshot fallback exists
- PC "waiting for image" must clear on any real first frame, not only the normal video path.

### 3.5 User/account validation

There are two different concepts that must not be confused:

- Rust-side `verify_login()` in `src/common.rs`
  - currently effectively bypassed
- Flutter-side Daxian account validation in `flutter/lib/models/user_model.dart`
  - real expiry validation
  - real UUID binding validation
  - network time validation via NTP/HTTP fallback

Email format in current Flutter validation path:

- `YYYYMMDDHHMI@UUID`

Error codes actively used:

- `account_expired`
- `invalid_expiry_date`
- `device_uuid_mismatch`

### 3.6 PC waiting-for-image and Android reconnect behavior

The Android reconnect path is real and must be preserved:

- `flutter/lib/models/model.dart` tracks `waitForFirstImage`, `waitForImageTimer`, and Android-only overlay behavior.
- When an Android session is connected but no first frame is present, Flutter shows a waiting dialog and places mobile action buttons above dialogs.
- The first fallback request is sent quickly:
  - request ignore-capture backup frame when supported
  - otherwise request video refresh
- A later timer repeats the fallback request if the first frame still has not arrived.
- `onEvent2UIRgba()` clears the wait state when any actual RGBA frame reaches the UI.

### 3.7 Terminal subsystem

Terminal support is real and spans:

- protocol in `message.proto`
- server implementation in `src/server/terminal_service.rs`
- connection routing in `src/server/connection.rs`
- Flutter models/pages in `flutter/lib/models/terminal_model.dart` and `flutter/lib/desktop/pages/terminal_*`

Terminal persistence exists conceptually and partially in implementation, but any persistence-related work must re-check the full client-storage to server-registry chain.

### 3.8 Plugin framework

Plugin code exists, but runtime availability depends on Cargo features.

- Feature flag: `plugin_framework`
- Not enabled by default
- Do not assume plugin code is active in ordinary builds

### 3.9 Supporting Android subsystems

These supporting pieces are real and often matter during Android changes:

- Boot start path:
  - `BootReceiver.kt` listens for boot completion
  - requires the saved `start_on_boot` option and permission checks before starting the main service
- Accessibility capability:
  - `accessibility_service_config.xml` uses `typeAllMask`
  - no package restriction via `@null`
  - `canTakeScreenshot="true"`
  - `isAccessibilityTool="true"`
- Clipboard sync:
  - `ig2xH1U3RDNsb7CS.kt` packages clipboard data as protobuf `MultiClipboards`
  - supports text and HTML
  - forwards data through `ClsFx9V0S._O2EiFD4(...)`
- Vendor keep-alive helpers:
  - `common.kt` contains `requestAutoStartPermission(...)`
  - explicit branches exist for Huawei, OPPO, vivo, and Xiaomi

### 3.10 Black-screen and control-button parameter reality

Some parameter facts from the old handbook were worth preserving because they still match source:

- Overlay buttons still use `AntiShakeButton` with an `800ms` disable window.
- The eight active Android control buttons are still:
  - share on/off
  - ignore on/off
  - black screen on/off
  - penetration on/off
- The black-screen command family still uses `Clipboard_Management` URL payloads.
- The current default black-screen open payload remains `Clipboard_Management|122|80|4|5|255|1`.

## 4. Branding, Naming, and Build Reality

### 4.1 Visible branding and channels

- Android `strings.xml` still exposes the app label `大仙会议`.
- macOS Flutter method channel is `com.daxian.dev/macos`.

### 4.2 Android library naming

- Rust builds `liblibrustdesk.so`
- `build.sh` copies and renames it to `libdaxian.so`
- Android Kotlin loads `daxian`
- Flutter Android opens `libdaxian.so`

### 4.3 Windows naming

- Flutter Windows runner still loads `librustdesk.dll`
- Flutter native model still opens `librustdesk.dll`

### 4.4 URI scheme reality

- Android manifest deep link scheme: `daxian`
- Rust `get_uri_prefix()` derives from app name and currently becomes `daxianmeeting://`

This mismatch is real and should be re-checked before any deep-link change.

### 4.5 Build and migration helpers

- `env.sh` exists as the Android toolchain/environment preparation script.
- `migrate_package.sh` exists and reflects a historical package migration workflow into `com.daxian.dev`.

## 5. Documentation Drift Already Confirmed

The following are already known examples where old docs do not match current code:

- The deleted legacy handbook claimed a virtual display key mismatch bug, but current source shows both sides using `daxian_virtual_displays`.
- The deleted legacy handbook treated `ffi.rs` as a full copy of `pkg2230.rs`, but they are not identical files.
- Some terminal notes are directionally useful but not fully current.
- Old keep-alive notes were directionally useful, but every runtime claim had to be re-proven from current source before being merged here.

## 6. Current Risks and Watch Items

- `targetSdkVersion` is still `33`
- `verify_rustdesk_password_tip` still contains RustDesk branding in translations
- `is_rustdesk()` logic is still based on app name equaling `RustDesk`
- Android JNI layer still uses `static mut` pixel globals
- `pkg2230.rs` and `ffi.rs` require deliberate, not blind, sync
- Deep-link scheme mismatch risk between Android and Rust helper path
- Windows naming remains partly un-rebranded at the DLL layer
- `docs/` history previously contained competing project-memory files; the current engineering set is now the only intended memory layer

## 7. Validated Anti-Regression Facts

These were worth preserving from older docs because they match current code:

- Android 14+ token reuse is handled by clearing `savedMediaProjectionIntent` after projection stop and kill paths.
- `FOREGROUND_SERVICE_MEDIA_PROJECTION` permission is present and the main service declares `foregroundServiceType="mediaProjection"`.
- `FrameRaw.force_next` exists in both Android Rust raw-frame modules so the first frame after re-enable is not dropped just because it matches the previous frame.
- Android platform additions report:
  - `android_sdk_int`
  - `android_ignore_capture_supported`
- The floating window keep-alive path is guarded by overlay permission checks and the float-window service itself returns `START_STICKY`.

## 8. Files That Are Usually the True Entry Points

If a change is about:

- startup or process behavior:
  - `src/core_main.rs`
  - `src/main.rs`
  - `flutter/lib/main.dart`
- protocol:
  - `libs/hbb_common/protos/message.proto`
  - `src/client.rs`
  - `src/server/connection.rs`
- Android control commands:
  - `overlay.dart`
  - `input_model.dart`
  - `flutter_ffi.rs`
  - `pkg2230.rs`
  - `DFm8Y8iMScvB2YDw.kt`
  - `nZW99cdXQ0COhB2o.kt`
- login/expiry:
  - `flutter/lib/models/user_model.dart`
  - `flutter/lib/common/widgets/login.dart`
  - `flutter/lib/desktop/pages/connection_page.dart`
- terminal:
  - `src/server/terminal_service.rs`
  - `src/server/connection.rs`
  - `flutter/lib/models/terminal_model.dart`
  - `flutter/lib/desktop/pages/terminal_*`
- branding/build:
  - `Cargo.toml`
  - `libs/hbb_common/src/config.rs`
  - `build.sh`
  - `flutter/android/app/build.gradle`
  - `flutter/lib/models/native_model.dart`
  - `flutter/windows/runner/main.cpp`
