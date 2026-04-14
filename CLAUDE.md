# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is Conductor

Conductor is a native desktop GUI (Rust + egui) that provides a unified interface to multiple AI CLI backends (Anthropic Claude, OpenAI, Ollama, Gemini, Codex, etc.). It spawns CLI subprocesses, streams their output, and renders responses with Markdown. The full product specification lives in `spec/` (10 documents).

## Build & Run

```sh
cargo build          # build workspace
cargo run            # launch the app (conductor-app binary)
cargo test           # run all tests
cargo test -p conductor-core  # test just core crate
just screencapture   # screenshot the running Conductor window (macOS)
```

## Workspace Structure

Two crates in `crates/`:

- **conductor-core** — Pure library. No GUI or platform deps. All state types, backend abstraction, stream parsing, config, session management, security, commands.
- **conductor-app** — The egui application. Depends on conductor-core. UI rendering, async runtime bridge, theme, platform adapters.

Core has zero knowledge of egui. The app crate imports core types and renders them.

## Architecture: Unidirectional Data Flow

```
UI (egui frame) ──Action──► tokio dispatcher ──state mutation──► Arc<RwLock<AppState>> ──read──► UI
```

- **Actions** (`events::Action` enum): UI emits these via `mpsc::UnboundedSender`. ~25 variants covering send message, switch backend, new session, settings, etc.
- **SharedState** (`bridge.rs`): `Arc<parking_lot::RwLock<AppState>>` + `egui::Context`. The `mutate()` method acquires write lock, applies change, calls `request_repaint()`.
- **Dispatcher** (`runtime.rs`): Async loop that receives Actions and routes to handlers. Spawns tokio tasks for subprocess streaming, backend discovery, etc.
- The UI thread never blocks. All I/O runs in tokio tasks. Write locks are held only for single mutations (<100μs).

## Backend Abstraction

`backend/mod.rs` defines two traits:
- `BackendDefinition` — How to discover, configure, and invoke a CLI (binary name, version command, auth check, chat command builder, capabilities).
- `StreamParser` — Stateful parser that converts CLI stdout lines into `StreamEvent` variants (TextChunk, ToolStart, ToolResult, ThinkingChunk, UsageData, Error, Done).

7 concrete backends in `definitions.rs`. Adding a new one is ~60 lines implementing `BackendDefinition`. The orchestrator (`orchestrator.rs`) works only through these traits.

## Key State Types (state.rs)

`AppState` is the single source of truth: sessions, backend registry, tray state, voice, channels, scheduler, connectors, notifications, config. `Session` holds messages, streaming state, tool cards, usage totals. `Message` has role, content, status, usage metrics, tool cards.

## UI Layer (conductor-app)

- `app.rs` — `eframe::App` impl. Two modes: empty state (greeting + centered input + chips) and chat mode (messages + bottom input). Conditionally shows sidebar and settings.
- `ui/sidebar.rs` — Conversation list with hover/active states.
- `ui/chat/header.rs` — Minimal model selector dropdown, sidebar toggle.
- `ui/chat/message_list.rs` — Centered content column (640px max), user bubbles right-aligned, assistant messages rendered as Markdown via `egui_commonmark`.
- `ui/chat/input_bar.rs` — Rounded pill input with send/stop button inside.
- `ui/settings/` — Tabbed settings window (About, General, Backends, Debug + placeholder tabs).
- `theme.rs` — Claude-inspired color palette. Pure white surface, warm gray sidebar, terracotta accents.

## Config & Persistence

- Config: `~/.conductor/config.json` (JSON, atomic writes via temp+rename)
- Sessions: `~/.conductor/sessions/{uuid}.json` (one file per session)
- Environment sanitization before subprocess spawn: 3-tier model (blocklist, override-blocked, shell-wrapper) in `security.rs`

## Spec & Plan

- `spec/` — 10 specification documents (product overview, functional requirements, architecture, state model, UI/UX, data flow, algorithms, external interfaces, non-functional requirements, implementation strategy).
- `plan/` — 21 implementation plan documents covering phases 1-11 with task breakdowns, acceptance criteria, and design decisions. Gitignored but available locally.
