//! SwiftUI-style NavigationStack — push/pop drill-down navigation.
//!
//! ```ignore
//! // State: holds the navigation path
//! struct MyApp {
//!     nav: NavPath,
//!     items: Vec<Item>,
//! }
//!
//! // In update():
//! NavigationStack::new(&mut self.nav).show(ui, |ui, nav| {
//!     // Root view — shown when stack is empty
//!     Label::heading("Items").show(ui);
//!     List::new().show(ui, |list| {
//!         for item in &self.items {
//!             if list.item(|ui| {
//!                 Text::new(&item.name).show(ui);
//!             }).clicked() {
//!                 nav.push(item.id.clone());
//!             }
//!         }
//!     });
//! });
//!
//! // Check if we're on a detail page:
//! if let Some(item_id) = self.nav.current() {
//!     // render detail for item_id
//! }
//! ```

use crate::button::{Button, ButtonStyle};
use crate::colors;
use crate::divider::Divider;
use crate::icons;
use crate::typography::Font;

/// The navigation path — a stack of string IDs representing pushed views.
///
/// Store this in your app state. `NavigationStack` borrows it mutably.
#[derive(Default, Debug, Clone)]
pub struct NavPath {
    stack: Vec<String>,
}

impl NavPath {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// The currently active (topmost) destination ID, or `None` if at root.
    pub fn current(&self) -> Option<&str> {
        self.stack.last().map(|s| s.as_str())
    }

    /// How deep the navigation stack is (0 = at root).
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Whether we're at the root level.
    pub fn is_root(&self) -> bool {
        self.stack.is_empty()
    }

    /// Push a new destination onto the stack.
    pub fn push(&mut self, id: impl Into<String>) {
        self.stack.push(id.into());
    }

    /// Pop the topmost destination (go back one level).
    pub fn pop(&mut self) -> Option<String> {
        self.stack.pop()
    }

    /// Pop all the way back to root.
    pub fn pop_to_root(&mut self) {
        self.stack.clear();
    }
}

/// A handle passed to the root content closure for triggering navigation.
pub struct NavController<'a> {
    path: &'a mut NavPath,
}

impl<'a> NavController<'a> {
    /// Push a new view onto the navigation stack.
    pub fn push(&mut self, id: impl Into<String>) {
        self.path.push(id);
    }
}

/// Stack-based push/pop navigation.
pub struct NavigationStack<'a> {
    path: &'a mut NavPath,
    show_back_button: bool,
}

impl<'a> NavigationStack<'a> {
    pub fn new(path: &'a mut NavPath) -> Self {
        Self {
            path,
            show_back_button: true,
        }
    }

    /// Whether to show the automatic back button (default: true).
    pub fn show_back_button(mut self, show: bool) -> Self {
        self.show_back_button = show;
        self
    }

    /// Show the navigation stack.
    ///
    /// The `root` closure renders the root view and receives a `NavController`
    /// for pushing new destinations.
    ///
    /// After calling `show()`, check `path.current()` to determine if a
    /// detail view should be rendered. This two-step pattern lets you
    /// render different views based on the destination ID:
    ///
    /// ```ignore
    /// NavigationStack::new(&mut self.nav).show(ui, |ui, nav| {
    ///     // root content...
    ///     if some_button.clicked() { nav.push("detail"); }
    /// });
    ///
    /// if let Some(dest) = self.nav.current() {
    ///     match dest {
    ///         "detail" => detail_view(ui),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn show(
        self,
        ui: &mut egui::Ui,
        root: impl FnOnce(&mut egui::Ui, &mut NavController<'_>),
    ) {
        let p = colors::palette(ui);

        if self.path.is_root() {
            // Root view — no back button, just render content.
            let mut controller = NavController { path: self.path };
            root(ui, &mut controller);
        } else {
            // Detail view — show back button header, then content.
            if self.show_back_button {
                ui.horizontal(|ui| {
                    if Button::new(&format!("{} Back", icons::CHEVRON_LEFT))
                        .style(ButtonStyle::Borderless)
                        .small(true)
                        .show(ui)
                        .clicked()
                    {
                        self.path.pop();
                    }

                    // Show breadcrumb of the current depth.
                    if self.path.depth() > 1 {
                        ui.label(
                            egui::RichText::new(format!("({} levels deep)", self.path.depth()))
                                .size(Font::Caption.size())
                                .color(p.text_muted),
                        );
                    }
                });
                Divider::new().show(ui);
                ui.add_space(8.0);
            }

            let mut controller = NavController { path: self.path };
            root(ui, &mut controller);
        }
    }
}
