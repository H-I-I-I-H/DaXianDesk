# 工程总索引 / Engineering Index

最后一次基于全仓源码核验：2026-04-14

> 这是 **Codex / Claude Code / 人工开发者** 在进入本仓库后的第一份文档。
> 目标不是替代源码，而是提供**稳定、可检索、不会被中文措辞歧义污染**的工程记忆层。
> 本文件中的中文叙述用于解释；**文件名、类名、函数名、常量名、协议字段名、命令名一律保留英文原文**。

---

## 0. 使用约定（必须遵守）

### 0.1 真相优先级（Source of Truth Order）

1. **当前源码**
2. `docs/ENGINEERING_BASELINE.md`
3. `docs/ENGINEERING_ANDROID_RUNTIME.md`
4. `docs/TASK_ENTRYPOINTS.md`
5. `docs/REPO_TRUE_STRUCTURE_MAP.md`
6. `docs/DOCUMENT_AUDIT.md`
7. `CLAUDE.md`
8. `terminal.md`

说明：

- `terminal.md` 和 `CLAUDE.md` 仍有参考价值，但都存在**局部漂移**，不能高于源码与工程文档。
- 当工程文档与源码冲突时，**以源码为准**，并同步更新工程文档。
- 本仓库当前**没有 `AGENTS.md`**。如未来新增 `AGENTS.md`，应只做导航入口，并显式指向本文件，不得另起一套相互竞争的项目记忆。

### 0.2 文档写法约定（Canonical Writing Contract）

为了让 Codex / Claude Code 稳定读取，后续所有工程文档必须遵守以下格式：

- 中文用于解释业务含义与风险。
- 英文用于保持代码锚点稳定；不得翻译以下内容：
  - 文件路径
  - 类名 / 函数名 / 结构体名 / enum / 常量
  - protobuf message / field 名称
  - Android action / permission / service name
  - Cargo feature / Gradle property / build artifact name
- 任何关键概念首次出现时，写成：
  - `中文名 (Canonical English Term)`
  - 例：首帧等待（`waiting-for-first-frame`）
- 任何运行时结论，都至少给出**一个代码锚点**，格式为：
  - `文件路径` + `符号名/关键字符串`
- 不要用“它 / 那里 / 这条逻辑”这种无锚点指代来替代真实符号。
- 不要把“猜测、经验、上游 RustDesk 印象”写成既定事实。
- 不要把多个近义词混用。必须优先使用本文档定义的 canonical term。

### 0.3 文档同步规则（Doc Sync Rule）

每次代码修改后，执行下面的最小同步动作：

1. 重新检索改动链路涉及的所有层：
   - Flutter / Rust FFI / server / platform / Android Kotlin / build script
2. 判断是否改变了：
   - 入口文件
   - 状态机 / 运行时不变量
   - 协议消息形状
   - 构建产物命名
   - 文档审计结论
3. 若改变，至少同步更新：
   - `docs/ENGINEERING_BASELINE.md`
   - 若涉及 Android 运行时，再更新 `docs/ENGINEERING_ANDROID_RUNTIME.md`
   - 若涉及入口或维护方式，再更新 `docs/TASK_ENTRYPOINTS.md`
   - 若文档真实性发生变化，再更新 `docs/DOCUMENT_AUDIT.md`
4. 更新时继续使用**中文解释 + English anchor + precise file path** 的写法。
5. 不要创建新的“记忆型文档”与本套文档竞争，除非现有文档结构已经证明无法承载。

---

## 1. 阅读顺序（Reading Order）

### 1.1 所有任务通用

1. `docs/ENGINEERING_INDEX.md`
2. `docs/ENGINEERING_BASELINE.md`
3. `docs/TASK_ENTRYPOINTS.md`
4. 对应源码入口文件

### 1.2 Android 运行时 / 重连 / 黑屏 / 无视 / 保活任务

1. `docs/ENGINEERING_INDEX.md`
2. `docs/ENGINEERING_BASELINE.md`
3. `docs/ENGINEERING_ANDROID_RUNTIME.md`
4. `docs/TASK_ENTRYPOINTS.md`
5. Android 源码链路

### 1.3 结构摸底 / 全仓定位

1. `docs/ENGINEERING_INDEX.md`
2. `docs/REPO_TRUE_STRUCTURE_MAP.md`
3. `docs/ENGINEERING_BASELINE.md`

### 1.4 文档是否可信 / 是否漂移

1. `docs/DOCUMENT_AUDIT.md`
2. 回到对应源码核验

---

## 2. 文档用途（What Each Doc Is For）

### `docs/ENGINEERING_BASELINE.md`

记录**已经通过源码核验**的项目身份、真实架构、关键链路、品牌/构建现实、已知风险与漂移。

### `docs/ENGINEERING_ANDROID_RUNTIME.md`

记录 Android 端真实运行时模型，包括：

- 服务状态 vs 帧源状态 vs PC 等首帧状态
- `MediaProjection`、ignore fallback、overlay keep-alive
- Android 10 与 Android 11+ 的能力边界
- 不得回归的运行时不变量

### `docs/TASK_ENTRYPOINTS.md`

按“改动类型”组织入口文件和检查清单，目的是让 agent 修改代码时从**正确的第一批文件**开始，而不是盲搜。

### `docs/REPO_TRUE_STRUCTURE_MAP.md`

记录全仓真实结构图，帮助快速判断：

- 哪些目录是主路径
- 哪些目录是兼容路径
- 哪些目录是平台专有实现
- 哪些目录和文档主链强相关

### `docs/DOCUMENT_AUDIT.md`

审计各类文档与当前代码的关系：

- 哪些可信
- 哪些部分可信
- 哪些已经过时
- 哪些只能当历史背景

### `CLAUDE.md`

现有的 Claude Code 仓库说明。它是有用的入口，但应服从 `docs/ENGINEERING_INDEX.md` 与基线文档，不应与本套工程文档冲突。

---

## 3. 快速事实（Quick Facts）

- 本项目是**基于 RustDesk 深度定制**的远程控制产品。
- Rust 主 crate：`rustdesk`
- Rust library 名称：`librustdesk`
- 产品运行时名称：`DaxianMeeting`
- Android package：`com.daxian.dev`
- Android 可见标签：`大仙会议`
- Flutter package：`flutter_hbb`
- Android 自定义链路并非轻量修补，而是包含：
  - Flutter UI 命令入口
  - Rust FFI 映射
  - protobuf / server 路由
  - Android JNI bridge
  - Kotlin `MainService` / `AccessibilityService`
- 当前生效的 Android Rust JNI 模块是 `libs/scrap/src/android/pkg2230.rs`；`ffi.rs` 仍存在但不是 `mod.rs` 导出的主路由。
- 项目不仅有 Flutter UI，也保留了 `src/ui/` 的旧 Sciter 路径。
- 项目不仅有远控核心，还存在 `src/hbbs_http/` 这条账号 / OIDC / 下载 / 同步 / 录像上传链路。
- Windows 隐私模式（`privacy_mode`）与虚拟显示器（`virtual_display_manager`）是独立维护面，不能忽略。

---

## 4. 推荐的第一批检索命令（First Search Batch）

```bash
git -c safe.directory="$PWD" status --short
rg --files docs src libs flutter CLAUDE.md terminal.md
rg -n "<keyword>" src libs flutter docs CLAUDE.md terminal.md
```

根据任务主题继续追：

```bash
# Android 自定义命令 / 黑屏 / 无视 / 分享
rg -n "wheelblank|wheelbrowser|wheelanalysis|wheelback|wheelstart|wheelstop|MOUSE_TYPE_|PIXEL_SIZEBack|PIXEL_SIZEBack8|VIDEO_RAW|SKL|shouldRun" src libs flutter

# waiting-for-image / Android 重连
rg -n "waitForFirstImage|waitForImageTimer|onEvent2UIRgba|showConnectedWaitingForImage|android_ignore_capture_supported" src flutter

# 账号 / OIDC / 下载 / 上传
rg -n "account_auth|OidcSession|download_file|get_download_data|record_upload|sync::start|is_pro" src flutter

# 隐私模式 / 虚拟显示
rg -n "privacy_mode|daxian_virtual_displays|supported_privacy_mode_impl|win_virtual_display" src flutter
```

---

## 5. 关键术语（Canonical Terms）

以下术语后续必须固定写法：

- 首帧等待（`waiting-for-first-frame`）
- waiting for image 对话框（`waiting-for-image dialog`）
- 正常视频路径（`normal MediaProjection video path`）
- 无视回退路径（`ignore-capture fallback path`）
- 穿透路径（`SKL pass-through path`）
- 分享恢复（`restoreMediaProjection`）
- 关闭分享 / projection 丢失（`projection stopped / share off`）
- Android 主服务（`MainService`, `DFm8Y8iMScvB2YDw.kt`）
- Android 输入服务（`AccessibilityService`, `nZW99cdXQ0COhB2o.kt`）
- 浮窗服务（`FloatWindowService`, `DFrLMwitwQbfu7AC.kt`）
- 旧桌面 UI（`Sciter UI`, `src/ui/`）
- Flutter 桌面多窗口（`desktop multi-window`）
- 持久终端服务（`persistent terminal service`）
- 账号 OIDC 授权（`OIDC device auth flow`）
- Windows 隐私模式（`Windows privacy mode`）
- Windows 虚拟显示器（`Windows virtual display`）

---

## 6. 不要做的事（Do Not）

- 不要把中文概念自由改写成多个不同说法。
- 不要新增新的“记忆型 markdown”与本套文档竞争。
- 不要假设 `terminal.md` 或 `CLAUDE.md` 一定是最新。
- 不要把 `ffi.rs` 当成 `pkg2230.rs` 的精确镜像。
- 不要只看 Flutter 而忽略 `src/ui/`、`src/hbbs_http/`、`src/privacy_mode.rs`、`build.sh`。
- 不要在未核代码前，把“上游 RustDesk 的行为”直接套用到本项目。

---

## 7. 当前核验到的外部文档风险

- `terminal.md` 中 terminal `service_id` 仍描述为 `tmp_` / `persist_`，但当前源码主实现使用的是 `ts_<uuid>`。
- `CLAUDE.md` 把 deep link 简化成 `daxian://`，而实际代码存在：
  - Android manifest scheme：`daxian`
  - Rust `get_uri_prefix()`：从 `APP_NAME` 推导，当前更接近 `daxianmeeting://`
- 这些差异在 `docs/DOCUMENT_AUDIT.md` 中有更完整说明。

---

## 8. 维护结论

这套工程文档的目标是：**让 agent 在中文说明下仍然能稳定落到英文代码锚点上**。

因此，后续同步文档时要坚持：

- 解释用中文
- 代码锚点用英文原文
- 每个结论都能回到真实文件和真实符号
- 不额外扩散新的记忆文档
