# egui-swift

A SwiftUI-style UI framework built on top of [egui](https://github.com/emilk/egui). Write egui apps that look and feel like native macOS / SwiftUI with familiar naming, automatic dark/light theming, and SwiftUI-compatible type aliases.

## Build

```sh
cargo build -p egui-swift
cargo test -p egui-swift
```

Single dependency: `egui = "0.31"`. No platform-specific code.

## Quick Start

```rust
use egui_swift::prelude::*;

fn my_view(ui: &mut egui::Ui) {
    let p = ui.palette();
    Text::new("Settings").font(Font::Title).show(ui);
    Section::new().header("Appearance").show(ui, |ui| {
        Toggle::new(&mut dark_mode).label("Dark mode").show(ui);
        Picker::new("Theme", &mut theme, &options).show(ui);
    });
    Image::system_name("gear").tint(p.accent).show(ui);
}
```

**Always use `use egui_swift::prelude::*;`** — one import gives you every type, trait, and alias.

## Architecture

### Design Principles

1. **SwiftUI vocabulary** — Types are named after their SwiftUI equivalents. `Text`, `Section`, `GroupBox`, `ProgressView`, `ContentUnavailableView`, `NavigationSplitView` all work. Native names (`Label`, `FormSection`, `Card`, `ProgressIndicator`, `EmptyState`) also work.
2. **Builder pattern everywhere** — `Component::new(required).option(val).show(ui) -> Response`. Every `show()` returns `egui::Response` so you can always chain `.clicked()`, `.changed()`, etc.
3. **Auto-theming** — Components read `ui.visuals().dark_mode` internally and pick colors from the palette. No manual palette passing needed inside components.
4. **Zero app logic** — This crate knows nothing about any specific application. It only depends on `egui`.

### Module Organization

```
src/
  prelude.rs            — Single re-export of everything
  swiftui_compat.rs     — Type aliases: Text, Section, GroupBox, ProgressView, ContentUnavailableView

  colors.rs             — Palette struct with dark()/light() constructors, palette(ui) accessor
  ext.rs                — UiExt (.palette(), .centered_content()), CtxExt (.palette()), ColorExt (.opacity())
  typography.rs          — Font enum: LargeTitle/Title/Headline/Body/Callout/Subheadline/Caption/Footnote
  theme.rs              — Layout constants + apply_macos_style(ctx)
  helpers.rs            — paint_shadow, lerp_color, animate_bool, truncate_text
  icons.rs              — Unicode glyph constants (CHEVRON_RIGHT, GEAR, SPARKLE, etc.)
  image.rs              — Image::system_name() SF Symbols mapper (~125 symbols)

  label.rs              — Text/Label with .font(), .secondary(), .muted(), .accent(), .bold(), .italic()
  button.rs             — Button with ButtonStyle: BorderedProminent/Bordered/Borderless/Destructive
  toggle.rs             — iOS-style animated toggle switch
  divider.rs            — 0.5px themed separator with optional inset
  status_dot.rs         — Colored circle indicator with optional label

  card.rs               — Rounded container (also aliased as GroupBox)
  form_section.rs       — Grouped inset section with header/footer (also aliased as Section)
  form_row.rs           — Label + right-aligned control row

  picker.rs             — Styled ComboBox dropdown
  radio_group.rs        — Animated radio button group
  text_field.rs         — Styled text input with focus ring
  stepper.rs            — +/- numeric control in a capsule

  button_row.rs         — Right-aligned horizontal button group
  data_table.rs         — Striped grid with header row
  disclosure_group.rs   — Animated collapsible section
  empty_state.rs        — Centered icon + title + subtitle + action (also ContentUnavailableView)

  stacks.rs             — VStack / HStack with .padding(), .background(), .corner_radius(), .border()
  spacer.rs             — Spacer::trailing(ui, |ui|), Spacer::fixed(f32)
  labeled_content.rs    — LabeledContent key-value rows
  scroll_view.rs        — ScrollView::vertical() / .horizontal() / .both()

  list.rs               — Styled scrollable list with dividers and selection (Plain / InsetGrouped)
  tab_view.rs           — Bottom tab bar with SF Symbol icons + labels
  navigation_split_view.rs — Sidebar + detail split layout
  toolbar.rs            — Top bar with leading/title/trailing regions
  alert.rs              — Lightweight confirmation dialog with animated backdrop
  sheet.rs              — Modal panel with animated backdrop
  progress_indicator.rs — Spinner and progress bar (also ProgressView)
  context_menu.rs       — Styled right-click menu

  badge.rs              — Count pill
  chat_input.rs         — Multiline chat input with send/stop
  conversation_item.rs  — Sidebar conversation list item
  nav_row.rs            — Icon + label sidebar navigation row
  search_field.rs       — Rounded search input with magnifying glass
  section_header.rs     — Uppercase small section label
  segmented_control.rs  — Multi-segment pill selector
  sidebar.rs            — SidebarPanel layout helper
  suggestion_chip.rs    — Bordered pill chips
  user_profile.rs       — Avatar + name + settings gear
```

### Color System

`Palette` has ~30 semantic colors. Use `ui.palette()` or `ctx.palette()` (via `UiExt`/`CtxExt` traits from prelude).

| Category | Fields |
|----------|--------|
| **Surfaces** | `surface`, `surface_raised`, `sidebar_bg`, `input_bg`, `card_bg` |
| **Text** | `text_primary`, `text_secondary`, `text_muted`, `text_placeholder`, `text_on_accent` |
| **Accent** | `accent`, `accent_bg`, `accent_subtle` |
| **Status** | `status_green`, `status_yellow`, `status_red`, `destructive` |
| **Borders** | `border`, `border_subtle`, `divider` |
| **States** | `hover_bg`, `active_bg`, `active_indicator`, `toggle_on`, `toggle_off` |
| **Overlay** | `shadow`, `overlay_bg` |
| **Chat** | `user_bubble_bg`, `error_bg`, `tool_card_bg` |

Components use semantic Label modifiers instead of raw colors where possible:
- `.secondary()` — uses `text_secondary`
- `.muted()` — uses `text_muted`
- `.accent()` — uses `accent`
- `.destructive()` — uses `destructive`/`status_red`

### Font Presets

The `Font` enum maps to SwiftUI's font styles:

| Variant | Size | Bold by default |
|---------|------|-----------------|
| `Font::LargeTitle` | 34pt | no |
| `Font::Title` | 22pt | yes |
| `Font::Headline` | 17pt | yes |
| `Font::Body` | 14.5pt | no |
| `Font::Callout` | 13pt | no |
| `Font::Subheadline` | 12pt | no |
| `Font::Caption` | 11pt | no |
| `Font::Footnote` | 10pt | no |

### Layout Constants (`Layout::`)

| Constant | Value | Use |
|----------|-------|-----|
| `MAX_CONTENT_WIDTH` | 640px | Chat message column, input bar |
| `SIDEBAR_WIDTH` | 240px | Main sidebar panel |
| `TOOLBAR_HEIGHT` | 44px | Top bar |
| `FORM_ROW_HEIGHT` | 36px | Label + control rows |
| `CARD_RADIUS` | 10px | Card/GroupBox corners |
| `CONTROL_RADIUS` | 8px | Input/button corners |
| `PILL_RADIUS` | 16px | Capsule-shaped elements |
| `BODY_FONT_SIZE` | 14.5px | Default body text |

## SwiftUI → egui-swift Reference

| SwiftUI | egui-swift |
|---------|-----------|
| `Text("x").font(.title)` | `Text::new("x").font(Font::Title).show(ui)` |
| `Text("x").foregroundColor(.secondary)` | `Text::new("x").secondary().show(ui)` |
| `Button("Save") { action }` | `if Button::new("Save").show(ui).clicked() { action }` |
| `Button("Save").buttonStyle(.borderedProminent)` | `Button::new("Save").style(ButtonStyle::BorderedProminent).show(ui)` |
| `Toggle("Label", $val)` | `Toggle::new(&mut val).label("Label").show(ui)` |
| `Picker("Label", $sel) { ForEach... }` | `Picker::new("Label", &mut sel, &opts).show(ui)` |
| `TextField("Placeholder", $text)` | `TextField::new(&mut text).placeholder("Placeholder").show(ui)` |
| `Stepper(value: $v, in: 1...100)` | `Stepper::new(&mut v, 1.0..=100.0).show(ui)` |
| `Section("Header") { content }` | `Section::new().header("Header").show(ui, \|ui\| { content })` |
| `GroupBox { content }` | `GroupBox::new().show(ui, \|ui\| { content })` |
| `DisclosureGroup("Title") { content }` | `DisclosureGroup::new("Title", &mut open).show(ui, \|ui\| { content })` |
| `ProgressView()` | `ProgressView::spinner().show(ui)` |
| `ProgressView(value: 0.5)` | `ProgressView::bar(0.5).show(ui)` |
| `ContentUnavailableView("Title", systemImage: "x")` | `ContentUnavailableView::new("Title").icon("emoji").show(ui)` |
| `Image(systemName: "gear")` | `Image::system_name("gear").show(ui)` |
| `NavigationSplitView { sidebar } detail: { }` | `NavigationSplitView::new("id").show(ctx, \|sb, dt\| { ... })` |
| `Divider()` | `Divider::new().show(ui)` |
| `List { ForEach... }` | `List::new().inset_grouped().show(ui, \|list\| { list.row(sel, \|ui\| { }) })` |
| `TabView { }.tabItem { }` | `TabView::new(&mut sel).tab("id", "Label", "sf_name", \|ui\| { }).show(ui)` |
| `.alert("Title", isPresented: $show) { }` | `Alert::new("Title", &mut show).destructive_action("Delete").cancel().show(ctx)` |
| `.sheet(isPresented: $open) { }` | `Sheet::new("id", &mut open, "Title").show(ctx, \|ui\| { })` |
| `VStack(spacing: 8) { }` | `VStack::new().spacing(8.0).show(ui, \|ui\| { })` |
| `HStack { }.padding().background(.gray)` | `HStack::new().padding(12.0).background(p.surface_raised).show(ui, \|ui\| { })` |
| `Spacer()` | `Spacer::trailing(ui, \|ui\| { /* trailing content */ })` |
| `Spacer().frame(height: 16)` | `Spacer::fixed(16.0).show(ui)` |
| `LabeledContent("Email") { Text(email) }` | `LabeledContent::new("Email", &email).show(ui)` |
| `ScrollView { }` | `ScrollView::vertical().show(ui, \|ui\| { })` |
| `Color.gray.opacity(0.1)` | `p.text_muted.opacity(0.1)` (via `ColorExt` trait) |

### VStack / HStack Modifiers

Stacks support SwiftUI-style modifier chains:
```rust
VStack::new()
    .spacing(8.0)          // gap between children
    .padding(16.0)         // uniform inner padding
    .padding_xy(16.0, 8.0) // horizontal, vertical padding
    .background(color)     // fill color
    .corner_radius(10.0)   // rounded corners
    .border(0.5, color)    // stroke
    .frame_width(300.0)    // explicit width
    .frame_height(200.0)   // explicit height
    .show(ui, |ui| { ... });
```

When no visual modifiers are set, VStack/HStack are zero-overhead wrappers around `ui.vertical()`/`ui.horizontal()`. When any visual modifier is set, they automatically wrap content in an `egui::Frame`.

### Spacer Pattern

In egui's immediate mode, Spacer can't push siblings like in SwiftUI. Use `Spacer::trailing()` to render trailing content right-aligned:
```rust
HStack::new().show(ui, |ui| {
    Label::new("Title").show(ui);
    Spacer::trailing(ui, |ui| {        // everything inside is right-aligned
        Button::new("Action").show(ui);
    });
});
```

For vertical bottom-push: `Spacer::bottom(ui, |ui| { ... })`.
For fixed gaps: `Spacer::fixed(16.0).show(ui)`.

## Cookbook — Common Patterns

### Settings page with sidebar navigation

```rust
use egui_swift::prelude::*;

NavigationSplitView::new("settings")
    .sidebar_width(160.0)
    .show(ctx, |sidebar, detail| {
        sidebar.show(|ui| {
            Label::heading("Settings").show(ui);
            Spacer::fixed(8.0).show(ui);
            Divider::new().show(ui);
            Spacer::fixed(8.0).show(ui);

            for (id, label, sf) in [("general", "General", "gear"), ("about", "About", "info.circle")] {
                let icon = egui_swift::image::sf_symbol(sf);
                if NavRow::new(label).icon(icon).active(selected == id).show(ui).clicked() {
                    selected = id.to_string();
                }
            }
        });
        detail.show(|ui| {
            match selected.as_str() {
                "about" => about_view(ui),
                _ => general_view(ui),
            }
        });
    });
```

### Form with toggles, picker, and validation

```rust
fn general_view(ui: &mut egui::Ui) {
    Label::heading("General").show(ui);
    Spacer::fixed(16.0).show(ui);

    Section::new().header("Appearance").show(ui, |ui| {
        Toggle::new(&mut dark_mode).label("Dark mode").show(ui);
        Spacer::fixed(4.0).show(ui);
        let langs = vec![("en".into(), "English"), ("es".into(), "Spanish")];
        Picker::new("Language", &mut language, &langs).show(ui);
    });

    Spacer::fixed(12.0).show(ui);

    Section::new().header("Account").show(ui, |ui| {
        TextField::new(&mut name).label("Name").placeholder("Your name").show(ui);
        Spacer::fixed(8.0).show(ui);
        TextField::new(&mut email).label("Email").placeholder("email@example.com").show(ui);
    });

    Spacer::fixed(16.0).show(ui);

    let valid = !name.trim().is_empty() && email.contains('@');
    ButtonRow::show(ui, |ui| {
        Button::new("Cancel").style(ButtonStyle::Bordered).show(ui);
        Button::new("Save").style(ButtonStyle::BorderedProminent).enabled(valid).show(ui);
    });
}
```

### Heading with trailing action button

```rust
HStack::new().show(ui, |ui| {
    Label::heading("Documents").show(ui);
    Spacer::trailing(ui, |ui| {
        if Button::new("+ New").style(ButtonStyle::BorderedProminent).small(true).show(ui).clicked() {
            // handle action
        }
    });
});
```

### List with search filtering

```rust
SearchField::new(&mut query).show(ui);
Spacer::fixed(8.0).show(ui);

let filtered: Vec<_> = items.iter().enumerate()
    .filter(|(_, item)| query.is_empty() || item.name.to_lowercase().contains(&query.to_lowercase()))
    .collect();

List::new().inset_grouped().divider_inset(16.0).show(ui, |list| {
    for (i, item) in filtered {
        if list.row(i == selected, |ui| {
            HStack::new().spacing(8.0).show(ui, |ui| {
                Image::system_name(&item.icon).show(ui);
                VStack::new().spacing(2.0).show(ui, |ui| {
                    Text::new(&item.name).font(Font::Callout).show(ui);
                    Text::new(&item.detail).font(Font::Caption).secondary().show(ui);
                });
            });
        }).clicked() {
            selected = i;
        }
    }
});
```

### Card grid

```rust
ScrollView::vertical().show(ui, |ui| {
    for item in &items {
        Card::new()
            .padding(16.0)
            .background(p.surface_raised)
            .corner_radius(10.0)
            .border(0.5, p.border_subtle)
            .show(ui, |ui| {
                HStack::new().show(ui, |ui| {
                    StatusDot::new(item.status_color).show(ui);
                    VStack::new().spacing(4.0).show(ui, |ui| {
                        Text::new(&item.title).font(Font::Callout).bold(true).show(ui);
                        Text::new(&item.subtitle).font(Font::Caption).secondary().show(ui);
                    });
                    Spacer::trailing(ui, |ui| {
                        Button::new("View").style(ButtonStyle::Bordered).small(true).show(ui);
                    });
                });
            });
    }
});
```

### Confirmation dialog

```rust
// In your state struct: show_delete: bool

// Render the alert (call every frame — it auto-hides when not open):
let action = Alert::new("Delete item?", &mut self.show_delete)
    .message("This action cannot be undone.")
    .destructive_action("Delete")
    .cancel()
    .show(ctx);

if action == AlertAction::Destructive {
    self.items.remove(self.selected);
}

// Trigger it from a button:
if Button::new("Delete").style(ButtonStyle::Destructive).show(ui).clicked() {
    self.show_delete = true;
}
```

### Tab-based app layout

```rust
TabView::new(&mut self.selected_tab)
    .tab("home", "Home", "house", |ui| {
        Label::heading("Home").show(ui);
        // home content...
    })
    .tab("search", "Search", "magnifyingglass", |ui| {
        Label::heading("Search").show(ui);
        SearchField::new(&mut self.query).show(ui);
    })
    .tab("settings", "Settings", "gear", |ui| {
        Label::heading("Settings").show(ui);
        // settings content...
    })
    .show(ui);
```

### Detail view with LabeledContent

```rust
Label::heading(&item.name).show(ui);
Spacer::fixed(16.0).show(ui);

Section::new().header("Details").show(ui, |ui| {
    LabeledContent::new("Status", "Active").show(ui);
    LabeledContent::new("Created", "2024-01-15").show(ui);
    LabeledContent::new("Size", "2.4 MB").show(ui);
    LabeledContent::labeled("Tags").show_with(ui, |ui| {
        HStack::new().spacing(4.0).show(ui, |ui| {
            Badge::new(3).show(ui);
            Text::new("documents").font(Font::Caption).secondary().show(ui);
        });
    });
});
```

### Complete minimal app skeleton

```rust
use egui_swift::prelude::*;

fn main() -> eframe::Result {
    eframe::run_native("My App", eframe::NativeOptions::default(), Box::new(|cc| {
        egui_swift::theme::apply_macos_style(&cc.egui_ctx);
        Ok(Box::new(MyApp::default()))
    }))
}

#[derive(Default)]
struct MyApp { /* state fields */ }

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let p = ctx.palette();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(p.surface))
            .show(ctx, |ui| {
                Label::heading("Hello, egui-swift!").show(ui);
            });
    }
}
```

## Adding a New Component

1. Create `src/my_component.rs` with the builder pattern:
   ```rust
   use crate::colors;

   pub struct MyComponent<'a> { /* fields */ }

   impl<'a> MyComponent<'a> {
       pub fn new(required: &'a str) -> Self { ... }
       pub fn option(mut self, val: bool) -> Self { self.option = val; self }
       pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
           let p = colors::palette(ui);
           // render with p.text_primary, p.accent, etc.
       }
   }
   ```
2. Add `pub mod my_component;` to `lib.rs`.
3. Add `pub use crate::my_component::MyComponent;` to `prelude.rs`.
4. If it maps to a SwiftUI type, add a type alias in `swiftui_compat.rs` and re-export in `prelude.rs`.

### Conventions

- All `show()` methods return `egui::Response` (never void).
- Colors come from `colors::palette(ui)` or `ui.palette()`, never hardcoded.
- Semi-transparent colors via `color.opacity(0.1)` (ColorExt trait), not `from_rgba_premultiplied`.
- Animations use `helpers::animate_bool(ui, id, value, seconds)`.
- Shadows use `helpers::paint_shadow(ui, rect, rounding, spread, color)`.
- Font sizes come from `Font` enum or `Layout::` constants, not magic numbers.
- Border strokes are 0.5px (macOS hairline convention).
- Corner radii: 10px cards, 8px controls, 16px pills, 20px chat bubbles.
