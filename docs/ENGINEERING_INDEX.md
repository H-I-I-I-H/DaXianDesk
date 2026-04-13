# Engineering Index

Last verified: 2026-04-14

This index is the first file to read before any future code change. Its purpose is to keep project understanding stable across long conversations and prevent relying on stale documentation.

## Ground Rules

- Source of truth priority:
  1. Source code
  2. `docs/ENGINEERING_BASELINE.md`
  3. `docs/ENGINEERING_ANDROID_RUNTIME.md`
  4. `docs/TASK_ENTRYPOINTS.md`
  5. `terminal.md`
- Do not overwrite user changes in a dirty worktree.
- Do not run `git commit`. Code changes only; the user reviews and commits.
- Old project-memory docs have been retired. Do not recreate alternate memory files without first proving a real gap in the current set.

## Read Order

1. `docs/ENGINEERING_INDEX.md`
2. `docs/ENGINEERING_BASELINE.md`
3. `docs/ENGINEERING_ANDROID_RUNTIME.md` if the task touches Android runtime, reconnection, capture, black screen, ignore mode, or keep-alive
4. `docs/TASK_ENTRYPOINTS.md`
5. Then jump into the specific source files for the task

## What Each Doc Is For

- `docs/ENGINEERING_BASELINE.md`
  - Verified project identity
  - Real architecture
  - Important runtime chains
  - Documentation drift and current risks
- `docs/ENGINEERING_ANDROID_RUNTIME.md`
  - Android service state versus frame state
  - PC waiting-for-image behavior
  - Android 10 versus Android 11+ runtime boundary
  - Anti-regression rules for share, ignore, and keep-alive work
- `docs/TASK_ENTRYPOINTS.md`
  - Feature-oriented file map
  - Where to start for each kind of change
  - Pre-change and post-change checklists

## Existing Docs With Caution

- `terminal.md`
  - Useful for terminal intent and protocol, but some implementation details changed

## Quick Commands

```powershell
git -c safe.directory=C:/Users/Administrator/Desktop/Code/DaXianDesk status --short
rg -n "<keyword>" src libs flutter docs terminal.md
rg --files docs
```

## Current Fast Facts

- This is a RustDesk-derived remote control project with heavy Android customization.
- Rust core is in `src/` and `libs/`.
- Flutter desktop/mobile UI is in `flutter/lib/`.
- Android-native behavior is split across Rust JNI in `libs/scrap/src/android/` and Kotlin services in `flutter/android/app/src/main/kotlin/com/daxian/dev/`.
- The custom Android control path is real and must be understood end-to-end before editing.
- The Android runtime model is multi-plane: service alive, frame source alive, and PC first-frame wait are separate states.
