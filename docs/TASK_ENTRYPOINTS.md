# Task Entrypoints

Last verified: 2026-04-14

This file is organized by change type. Use it to jump directly into the right files before editing.

## 1. Before Any Change

Run:

```powershell
git -c safe.directory=C:/Users/Administrator/Desktop/Code/DaXianDesk status --short
rg -n "<feature keyword>" src libs flutter docs terminal.md
```

Check:

- Is the worktree dirty?
- Is there an existing doc claim that must be re-verified?
- Is the feature cross-layer?
- If the task touches Android runtime behavior, read `docs/ENGINEERING_ANDROID_RUNTIME.md` before editing.

## 2. Android Control Buttons / Custom Commands

Start here:

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

When adding a new custom command, verify all of these:

1. Flutter button exists or is wired
2. Dart `sendMouse()` type and URL are encoded
3. Rust `session_send_mouse()` maps the new type
4. `MouseEvent` carries enough protocol data
5. Server receives and forwards it
6. JNI layer dispatches it
7. Kotlin service actually handles it

## 3. Android Capture / Black Screen / Ignore / Share Recovery

Start here:

- `docs/ENGINEERING_ANDROID_RUNTIME.md`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/DFm8Y8iMScvB2YDw.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/nZW99cdXQ0COhB2o.kt`
- `flutter/android/app/src/main/kotlin/com/daxian/dev/common.kt`
- `libs/scrap/src/android/pkg2230.rs`
- `src/server/connection.rs`
- `flutter/lib/common/widgets/overlay.dart`
- `flutter/lib/models/input_model.dart`

Always re-check:

- `SKL`
- `shouldRun`
- `PIXEL_SIZEBack`
- `PIXEL_SIZEBack8`
- `VIDEO_RAW`
- `killMediaProjection()`
- `restoreMediaProjection()`
- whether the change accidentally turns "video path lost" into "service stopped"
- whether Android 10 is being treated differently from Android 11+

If the change touches reconnection or waiting-for-image behavior, also inspect:

- `src/flutter.rs`
- `flutter/lib/models/model.dart`
- `flutter/lib/desktop/screen/desktop_remote_screen.dart`
- `flutter/lib/desktop/widgets/remote_toolbar.dart`

Rules:

1. Preserve the distinction between service alive and frame source alive
2. Preserve first-frame clearing on any real RGBA frame
3. Preserve Android action buttons above waiting dialogs
4. Do not make Android 10 pretend screenshot fallback exists

## 4. Protocol / Message Shape

Start here:

- `libs/hbb_common/protos/message.proto`
- `libs/hbb_common/build.rs`
- `src/client.rs`
- `src/server/connection.rs`
- `src/flutter_ffi.rs`
- Dart caller side in `flutter/lib/models/`

Checklist:

- Protobuf field added or changed
- Sender updated
- Receiver updated
- Flutter/Rust bridge callers updated
- Platform-specific branches re-checked

## 5. Login / Account / Expiry / Device Binding

Start here:

- `flutter/lib/models/user_model.dart`
- `flutter/lib/common/widgets/login.dart`
- `flutter/lib/desktop/pages/connection_page.dart`
- `src/common.rs`
- `src/ui.rs` if old UI path is involved

Re-check:

- `ChinaNetworkTimeService`
- `validateUser()`
- `user_email`
- `mainGetUuid()`
- Rust `verify_login()` behavior versus Flutter-side product login behavior

## 6. Branding / Naming / Deep Link / App Identity

Start here:

- `Cargo.toml`
- `libs/hbb_common/src/config.rs`
- `flutter/pubspec.yaml`
- `flutter/android/app/build.gradle`
- `flutter/android/app/src/main/AndroidManifest.xml`
- `flutter/lib/utils/platform_channel.dart`
- `src/common.rs`
- `build.sh`
- `flutter/lib/models/native_model.dart`
- `flutter/windows/runner/main.cpp`

Common pitfalls:

- Android package and display name changed, but runtime app name did not
- Android scheme changed, but Rust helper URI prefix did not
- Android SO renamed, but Kotlin/Flutter loader names did not
- Windows DLL renamed, but runner/native loader names did not

## 7. Android JNI Layer

Treat these files carefully:

- `libs/scrap/src/android/pkg2230.rs`
- `libs/scrap/src/android/ffi.rs`
- `flutter/android/app/src/main/kotlin/pkg2230.kt`
- `flutter/android/app/src/main/kotlin/ffi.kt`

Rules:

- `pkg2230.rs` is the active routed module through `libs/scrap/src/android/mod.rs`
- `ffi.rs` is not guaranteed to be an exact copy
- Do not assume blind copy/paste sync is safe
- After JNI changes, inspect both exported Rust symbol style and Kotlin bridge calls
- If frame delivery changes, re-check `force_next`, `VIDEO_RAW`, and `PIXEL_SIZEBack8`

## 8. Terminal Work

Start here:

- `terminal.md`
- `src/server/terminal_service.rs`
- `src/server/connection.rs`
- `libs/hbb_common/protos/message.proto`
- `src/flutter_ffi.rs`
- `flutter/lib/models/terminal_model.dart`
- `flutter/lib/desktop/pages/terminal_connection_manager.dart`
- `flutter/lib/desktop/pages/terminal_tab_page.dart`

If touching persistence, verify:

- terminal `service_id`
- client storage path
- reconnect path
- server registry lifecycle

## 9. Plugin Work

Start here:

- `Cargo.toml`
- `src/plugin/mod.rs`
- `src/plugin/manager.rs`
- `src/plugin/plugins.rs`
- `flutter/lib/plugin/`

Before changing behavior, confirm:

- whether `plugin_framework` is enabled in the target build

## 10. Virtual Display / Privacy Mode / Windows Platform

Start here:

- `src/virtual_display_manager.rs`
- `flutter/lib/consts.dart`
- `src/privacy_mode.rs`
- `src/privacy_mode/win_virtual_display.rs`
- `src/privacy_mode/win_topmost_window.rs`
- `src/privacy_mode/win_mag.rs`
- `src/privacy_mode/win_exclude_from_capture.rs`

Re-check:

- platform addition keys
- current selected privacy implementation
- Windows-only build assumptions

## 11. Build and Packaging

Android:

- `build.sh`
- `env.sh`
- `flutter/build_android.sh`
- `flutter/build_android_deps.sh`
- `flutter/android/app/build.gradle`

Desktop:

- `build.py`
- `build.rs`
- platform runner files

Always re-check:

- output library name
- loadLibrary / `DynamicLibrary.open` names
- manifest / Gradle / bundle identifiers

## 12. After Any Change

Do at least these:

1. Re-run targeted `rg` checks for the touched feature
2. Re-open the edited call chain if the feature is cross-layer
3. Run the smallest practical validation available
4. Update `docs/ENGINEERING_BASELINE.md` if the truth changed
5. Update `docs/ENGINEERING_ANDROID_RUNTIME.md` if Android runtime truth changed
6. Never commit; leave git submission to the user
