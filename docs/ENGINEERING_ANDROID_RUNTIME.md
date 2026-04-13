# Engineering Android Runtime

Last verified from source: 2026-04-14

This document records the Android runtime model that actually exists in source today. It replaces older state-machine, keep-alive, and changelog notes that mixed useful reasoning with stale facts.

## 1. Core Principle

This project has multiple runtime planes that must not be collapsed into one:

- Android service state
- Android frame-source state
- PC waiting-for-first-frame state

The most important rule is:

- service alive does not mean MediaProjection is alive
- MediaProjection lost does not mean the Android service is stopped
- PC waiting for image must clear on any real first frame, not only the normal video path

## 2. Android Service States

Use these as mental models for future changes:

- `A0 service stopped`
  - Main Android service is not running
- `A1 service alive, no active MediaProjection stream`
  - service is ready
  - no normal video stream is active
- `A2 service alive, MediaProjection stream active`
  - normal capture path is producing frames
- `A3 service alive, ignore-fallback path active or being requested`
  - normal projection is unavailable
  - ignore-capture recovery path is the intended fallback on Android 11+

Relevant source anchors:

- `DFm8Y8iMScvB2YDw.kt`
  - `_isReady`
  - `_isStart`
  - `mediaProjection`
  - `killMediaProjection()`
  - `handleProjectionStoppedKeepService()`
  - `restoreMediaProjection()`
  - `startIgnoreFallback()`
- `common.kt`
  - `SKL`
  - `shouldRun`
- `pkg2230.rs`
  - `VIDEO_RAW`
  - `PIXEL_SIZEBack`
  - `PIXEL_SIZEBack8`

## 3. PC Display States

The PC side has its own state machine and it is intentionally not identical to Android service state:

- `P0 disconnected`
- `P1 connected but waiting for first frame`
- `P2 connected and receiving normal video`
- `P3 connected and receiving ignore/screenshot fallback frames`
- `P4 reconnecting`

Relevant source anchors:

- `flutter/lib/models/model.dart`
  - `waitForFirstImage`
  - `waitForImageTimer`
  - `_androidAutoReconnectTimer`
  - `showConnectedWaitingForImage()`
  - `onEvent2UIRgba()`
- `flutter/lib/common.dart`
  - `showMobileActionsOverlayAboveDialogs()`
  - `removeMobileActionsOverlayEntry()`

## 4. Event Rules

### 4.1 Open Share

Current source behavior:

1. `restoreMediaProjection()` begins recovery.
2. Ignore fallback is intentionally not shut down first.
3. If a saved projection intent still works, MediaProjection is restored directly.
4. Only after success:
   - ignore capture is stopped
   - `PIXEL_SIZEBack8` is set to `255`
   - normal sharing resumes

What must not regress:

- Do not disable ignore fallback before MediaProjection is truly available.

### 4.2 Close Share or Projection Stopped

Current source behavior:

1. Projection resources are released.
2. `_isReady` stays true.
3. `startIgnoreFallback()` is called.
4. Foreground notification is refreshed.
5. Floating-window keep-alive is re-ensured.

What this means:

- close-share is treated as capture-path loss, not app shutdown
- projection stop is treated as capture-path loss, not app shutdown

What must not regress:

- Do not convert close-share into service teardown.

### 4.3 Screen Off

Current source behavior:

- the project does not proactively stop MediaProjection just because the screen turns off
- if the system later stops MediaProjection, fallback handling begins from the projection-stopped path

What must not regress:

- Do not reintroduce logic that treats screen-off itself as a mandatory service stop.

### 4.4 Waiting for First Frame

Current Flutter behavior for Android sessions:

1. show waiting dialog
2. place Android action overlay above dialogs
3. quickly request a backup frame path:
   - ignore-capture request when supported
   - otherwise video refresh
4. run a later fallback timer if the first frame still has not arrived
5. clear wait state in `onEvent2UIRgba()` when any real RGBA frame reaches the UI

What must not regress:

- Do not make the PC wait only for normal video.
- Do not let waiting dialogs hide Android control actions.

## 5. Android Version Boundary

Current source advertises Android version capability to Flutter:

- `android_sdk_int`
- `android_ignore_capture_supported`

Current rule:

- `sdk_int >= 30`
  - ignore-capture fallback is considered supported
- `sdk_int < 30`
  - Android 10 branch keeps the service alive without screenshot fallback

This is enforced in two places:

- `src/server/connection.rs` populates platform additions
- `DFm8Y8iMScvB2YDw.kt` short-circuits `startIgnoreFallback()` on Android 10

What must not regress:

- Do not pretend Android 10 has the same fallback guarantees as Android 11+.

## 6. JNI and Raw-Frame Guards

These guards are easy to break accidentally:

- `FrameRaw.force_next`
  - first frame after re-enable is forced through even if identical to the previous frame
- `VIDEO_RAW`
  - raw output path must remain enabled when fallback mode expects it
- `PIXEL_SIZEBack8`
  - `0` allows ignore frames through
  - `255` blocks ignore frames

What must not regress:

- After capture-path changes, re-check both `pkg2230.rs` and `ffi.rs`.
- Active routing still goes through `pkg2230.rs`, but `ffi.rs` contains similar frame logic that should not drift blindly.

## 7. Keep-Alive Facts Confirmed From Source

- Main service declares `foregroundServiceType="mediaProjection"`.
- `FOREGROUND_SERVICE_MEDIA_PROJECTION` permission is present in the manifest.
- Foreground notification is built as a visible service notification:
  - low importance
  - private visibility
  - non-silent setup
  - blue color accent
- Floating-window keep-alive is permission-gated through `Settings.canDrawOverlays(...)`.
- Float-window service returns `START_STICKY`.
- Main service has an internal keep-alive action:
  - `ACT_KEEP_ALIVE_SERVICE`

Important limit:

- current code aims to keep the app alive within normal Android constraints
- it does not imply guaranteed immunity from OEM process killing

## 8. Anti-Regression Checklist

Before shipping any Android runtime change, re-check all of these:

1. Did we accidentally tie service state to MediaProjection state again?
2. Did we accidentally make PC wait only for normal video frames?
3. Did we accidentally hide Android control buttons behind a waiting dialog?
4. Did we accidentally make Android 10 look like it supports ignore-capture fallback?
5. Did we accidentally stop refreshing foreground or floating-window keep-alive after projection loss?
6. Did we re-check `force_next`, `VIDEO_RAW`, and `PIXEL_SIZEBack8` after touching frame flow?
