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

Three crates in `crates/`:

- **conductor-core** — Pure library. No GUI or platform deps. All state types, backend abstraction, stream parsing, config, session management, security, commands.
- **egui-swift** — SwiftUI-style UI component framework built on egui. No app logic. Provides styled, composable widgets (buttons, toggles, cards, forms, navigation, etc.) with dark/light mode support.
- **conductor-app** — The egui application. Depends on conductor-core and egui-swift. UI rendering, async runtime bridge, platform adapters.

Core has zero knowledge of egui. egui-swift has zero knowledge of Conductor. The app crate imports both.

## Architecture: Unidirectional Data Flow

```
UI (egui frame) ──Action──► tokio dispatcher ──state mutation──► Arc<RwLock<AppState>> ──read──► UI
```

- **Actions** (`events::Action` enum): UI emits these via `mpsc::UnboundedSender`. ~25 variants covering send message, switch backend, new session, settings, etc.
- **SharedState** (`bridge.rs`): `Arc<parking_lot::RwLock<AppState>>` + `egui::Context`. The `mutate()` method acquires write lock, applies change, calls `request_repaint()`.
- **Dispatcher** (`runtime.rs`): Async loop that receives Actions and routes to handlers. Spawns tokio tasks for subprocess streaming, backend discovery, etc.
- The UI thread never blocks. All I/O runs in tokio tasks. Write locks are held only for single mutations (<100μs).

## egui-swift — SwiftUI-Style UI Framework

**Always `use egui_swift::prelude::*;`** — one import gives you everything.

egui-swift mirrors SwiftUI's API naming so you can think in SwiftUI and write Rust. It provides SwiftUI-compatible type aliases alongside native names.

### SwiftUI → egui-swift Quick Reference

| SwiftUI | egui-swift | Example |
|---------|-----------|---------|
| `Text("x").font(.title)` | `Text::new("x").font(Font::Title).show(ui)` | Also available as `Label::new()` |
| `Text("x").foregroundColor(.secondary)` | `Text::new("x").secondary().show(ui)` | `.muted()`, `.accent()`, `.destructive()` also work |
| `Button("Save") { }` | `Button::new("Save").show(ui).clicked()` | Styles: `.style(ButtonStyle::BorderedProminent)` / `Bordered` / `Borderless` / `Destructive` |
| `Toggle("Dark mode", $val)` | `Toggle::new(&mut val).label("Dark mode").show(ui)` | |
| `Picker("Level", $sel) { }` | `Picker::new("Level", &mut sel, &opts).show(ui)` | Options: `&[(T, &str)]` |
| `TextField("Name", $text)` | `TextField::new(&mut text).placeholder("Name").show(ui)` | `.multiline(rows)`, `.monospace(true)` |
| `Stepper(value: $v, in: 1...100)` | `Stepper::new(&mut v, 1.0..=100.0).show(ui)` | `.step(5.0)`, `.label("Count")` |
| `Section("Header") { }` | `Section::new().header("Header").show(ui, \|ui\| { })` | Also `FormSection::new()` |
| `GroupBox { }` | `GroupBox::new().show(ui, \|ui\| { })` | Also `Card::new()`. `.border_color()`, `.shadow(true)` |
| `struct Foo: View { var body }` | `impl View for Foo { fn body(&mut self, ui) { } }` | `.show(ui)` to render |
| `NavigationStack { NavigationLink }` | `NavigationStack::new(&mut nav).show(ui, \|ui, nav\| { nav.push("id") })` | `NavPath` holds stack |
| `Form { Section { } }` | `Form::new().show(ui, \|ui\| { Section::new().show(ui, \|ui\| {}) })` | `.max_width(500.0)` |
| `DisclosureGroup("Title") { }` | `DisclosureGroup::new("Title", &mut open).show(ui, \|ui\| { })` | Animated chevron |
| `ProgressView()` | `ProgressView::spinner().show(ui)` | Also `ProgressView::bar(0.5).show(ui)` |
| `ProgressView(value: 0.5)` | `ProgressView::bar(0.5).show(ui)` | Also `ProgressIndicator` |
| `ContentUnavailableView("No results", systemImage: "magnifyingglass")` | `ContentUnavailableView::new("No results").icon("🔍").show(ui)` | Also `EmptyState` |
| `Image(systemName: "gear")` | `Image::system_name("gear").show(ui)` | Maps ~150 SF Symbol names to Unicode |
| `NavigationSplitView { sidebar } detail: { }` | `NavigationSplitView::new("id").show(ctx, \|sb, dt\| { })` | `.sidebar_width(160.0)` |
| `Divider()` | `Divider::new().show(ui)` | `.inset(16.0)` for left indent |
| `.sheet(isPresented: $open) { }` | `Sheet::new("id", &mut open, "Title").show(ctx, \|ui\| { })` | Animated backdrop |
| `List { ForEach... }` | `List::new().inset_grouped().show(ui, \|list\| { list.row(sel, \|ui\| {}) })` | `.divider_inset(16.0)` |
| `TabView { }` | `TabView::new(&mut sel).tab("id", "Label", "sf_name", \|ui\| {}).show(ui)` | Bottom tab bar |
| `.alert("Title", isPresented: $show) { }` | `Alert::new("Title", &mut show).destructive_action("Del").cancel().show(ctx)` | Returns `AlertAction` |
| `DataTable / Table` | `DataTable::new(&cols).show(ui, \|ui\| { })` | `.striped(true)` |
| `VStack(spacing: 8) { }` | `VStack::new().spacing(8.0).show(ui, \|ui\| { })` | `.padding()`, `.background()`, `.corner_radius()`, `.border()` |
| `HStack { }` | `HStack::new().show(ui, \|ui\| { })` | Same modifiers as VStack |
| `Spacer()` | `Spacer::trailing(ui, \|ui\| { trailing })` | Use inside HStack; `Spacer::fixed(16.0)` for gaps |
| `LabeledContent("Key", value: "Val")` | `LabeledContent::new("Key", "Val").show(ui)` | `.show_with(ui, \|ui\| { })` for custom controls |
| `ScrollView { }` | `ScrollView::vertical().show(ui, \|ui\| { })` | `.horizontal()`, `.both()`, `.stick_to_bottom(true)` |
| `Color.gray.opacity(0.1)` | `p.text_muted.opacity(0.1)` | `ColorExt` trait in prelude |
| `Color.primary` | `ui.palette().text_primary` | Or `Label::new("x").secondary()` |
| `Color.accentColor` | `ui.palette().accent` | Or `Label::new("x").accent()` |

### Font Presets (match SwiftUI exactly)

| SwiftUI | egui-swift | Size |
|---------|-----------|------|
| `.largeTitle` | `Font::LargeTitle` | 34pt |
| `.title` | `Font::Title` | 22pt, bold |
| `.headline` | `Font::Headline` | 17pt, bold |
| `.body` | `Font::Body` | 14.5pt |
| `.callout` | `Font::Callout` | 13pt |
| `.subheadline` | `Font::Subheadline` | 12pt |
| `.caption` | `Font::Caption` | 11pt |
| `.footnote` | `Font::Footnote` | 10pt |

### Common Patterns

```rust
use egui_swift::prelude::*;

// Page heading
Label::heading("Settings").show(ui);

// Palette access (extension trait)
let p = ui.palette();      // on &egui::Ui
let p = ctx.palette();     // on &egui::Context

// Centered content column
ui.centered_content(Layout::MAX_CONTENT_WIDTH, |ui| { ... });

// SF Symbol icons
Image::system_name("gear").show(ui);
Image::system_name("bell.fill").tint(p.accent).show(ui);

// Settings page layout
NavigationSplitView::new("settings")
    .sidebar_width(160.0)
    .show(ctx, |sidebar, detail| {
        sidebar.show(|ui| { /* nav items */ });
        detail.show(|ui| { /* content */ });
    });
```

### SF Symbols Available

The `Image::system_name()` mapper covers ~150 common SF Symbol names including:
- Navigation: `chevron.right`, `chevron.down`, `arrow.left`, `arrow.right`
- Actions: `plus`, `minus`, `xmark`, `checkmark`, `checkmark.circle.fill`
- Objects: `gear`, `magnifyingglass`, `bell`, `trash`, `folder`, `calendar`, `clock`, `star`, `heart`, `bookmark`, `tag`, `link`, `envelope`
- System: `lock`, `key`, `eye`, `globe`, `wifi`, `desktopcomputer`, `terminal`
- Shapes: `circle.fill`, `square.fill`, `triangle.fill`, `exclamationmark.triangle`
- Development: `ant`, `hammer`, `wrench`
- Nature: `sun.max`, `moon`, `cloud`, `bolt`, `flame`
- Misc: `sparkles`, `lightbulb`, `book`, `puzzlepiece`, `pencil`

Both `.fill` variants (e.g. `"bell"` and `"bell.fill"`) map to the same glyph.

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
- `ui/settings/` — Full settings pane using NavigationSplitView with 11 tabs, all built with egui-swift components.
- `theme.rs` — Thin wrapper that calls `egui_swift::theme::apply_macos_style()`.

## Config & Persistence

- Config: `~/.conductor/config.json` (JSON, atomic writes via temp+rename)
- Sessions: `~/.conductor/sessions/{uuid}.json` (one file per session)
- Environment sanitization before subprocess spawn: 3-tier model (blocklist, override-blocked, shell-wrapper) in `security.rs`

## Spec & Plan

- `spec/` — 10 specification documents (product overview, functional requirements, architecture, state model, UI/UX, data flow, algorithms, external interfaces, non-functional requirements, implementation strategy).
- `plan/` — 21 implementation plan documents covering phases 1-11 with task breakdowns, acceptance criteria, and design decisions. Gitignored but available locally.
