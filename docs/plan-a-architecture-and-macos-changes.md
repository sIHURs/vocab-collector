# Plan A 后的项目架构与 macOS 变化

本文描述 Plan A 完成后的实际代码结构，以及 macOS App 相比重构前的主要变化。

## 1. 当前项目结构

```text
vocab-collector-app/
├── apps/desktop/                 # Tauri 桌面入口与 OS 组合根
├── ui/                           # Svelte UI
├── crates/
│   ├── domain/                   # 词汇、Encounter、复习等领域规则
│   ├── application/              # App 用例与跨平台 Capture workflow
│   ├── capture/                  # Capture 状态机、请求并发与窗口布局
│   ├── storage/                  # SQLite、migration、repository、outbox
│   ├── sync/                     # 同步边界
│   ├── platform-api/             # 跨平台 trait、DTO、capability、typed error
│   └── platform-contract-tests/  # OS adapter 共用的契约测试
├── platform/
│   ├── macos/
│   │   ├── rust/                 # macOS 的 Rust adapter 与 FFI 边界
│   │   └── native/               # Swift Accessibility/OCR/Translation 实现
│   ├── linux/                    # Plan B adapter 骨架
│   └── windows/                  # Plan B adapter 骨架
└── docs/                         # 架构、开发和平台移交文档
```

核心依赖方向：

```text
Svelte UI
    ↓ typed Tauri commands/events
desktop composition root
    ↓
application + capture
    ↓
domain + storage + platform-api
    ↑
macOS / Linux / Windows adapters
```

关键规则：

- Shared Rust crates 不依赖 Swift、Win32、AT-SPI、Tauri 或具体 OS adapter。
- `platform-api` 只定义能力接口，例如 Selection、OCR、Translation、Permission 和 Window。
- `apps/desktop/src-tauri` 根据编译目标选择 adapter，并将其注入 shared workflow。
- SQLite、保存、Undo、复习和业务状态始终属于 shared Rust 层，不进入原生 adapter。
- UI 根据 capability 和 typed error 决定行为，不通过 OS 名称或错误字符串分支。

一次普通 Capture 的数据流为：

```text
快捷键 → desktop 创建 request ID → macOS adapter 读取选中文本
      → shared coordinator 接受 candidate → UI 立即显示 Capture 卡片
      → 异步翻译 → 用户保存 → SQLite transaction → 可选 Undo
```

每个异步结果都必须携带并校验原 request ID。旧 Capture 的翻译、权限、Undo 或关闭操作不能修改新 Capture。

## 2. Plan A 后 macOS App 的变化

| 方面 | Plan A 后的状态 |
| --- | --- |
| 原生代码位置 | Swift package 集中到 `platform/macos/native`，Rust FFI 集中到 `platform/macos/rust`。 |
| Desktop 职责 | Tauri 只负责组合、命令、事件和展示数据转换，不再直接调用 Swift ABI。 |
| Capture 状态 | Selection、OCR、翻译、重试、保存和 stale request 由一个 shared coordinator 管理。 |
| 展示时机 | 选中文本成功后立即显示卡片，不再等待最长 60 秒的翻译结果。 |
| 翻译失败 | 使用 `translation_unavailable` / `translation_failed`，可重试或无翻译保存。 |
| OCR | 必须先获得 Screen Recording 权限并显式确认结果；多显示器坐标统一到 primary-top-left logical space。 |
| 并发安全 | UI continuation、定时关闭和后端 hide 都校验 request ID，避免旧任务覆盖新卡片。 |
| 阻塞调用 | macOS OCR 和 Translation 在 Tokio blocking worker 上运行，不阻塞 async executor。 |
| 能力检测 | Accessibility、OCR、Translation、非激活窗口等能力通过 `PlatformCapabilities` 暴露。 |
| 错误边界 | Swift payload、指针和原生诊断留在 adapter 内；UI 只接收稳定 typed failure code。 |
| 可测试性 | Shared workflow 可注入 fake providers；Swift、FFI、Rust、desktop contract 和 UI 均有回归测试。 |

Plan A 的结果不是“把 macOS 代码复制给其他系统”，而是把可复用 App 逻辑留在 shared Rust 层，只让各 OS adapter 实现原生能力。以后修复 core 时三端可共同更新；修复 macOS Accessibility、OCR 或 Translation 时，改动主要限制在 `platform/macos`。

## 3. 当前完成度

macOS 自动化 gate 已通过：Swift 9 项、Rust 83 项、UI 26 项，以及 Clippy、Svelte、Rust、前端和 Tauri `.app` 构建。

以下内容仍未声明完成：

- macOS 实际权限拒绝/恢复和应用兼容矩阵；
- 多显示器真实交互验证；
- macOS distribution signing 与 notarization；
- Linux/Windows 编译、运行和实体机验证——这些属于 Plan B。

后续开发时，可先根据改动性质定位目录：业务规则改 `crates/`，桌面组合或 IPC 改 `apps/desktop`，界面改 `ui/`，原生能力改对应的 `platform/<os>`。
