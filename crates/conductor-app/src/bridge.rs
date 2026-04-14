use std::sync::Arc;

use parking_lot::RwLock;

use conductor_core::state::AppState;

/// Thread-safe shared state wrapper.
///
/// The UI thread acquires a read lock each frame.
/// Background tasks acquire write locks briefly to push updates.
#[derive(Clone)]
pub struct SharedState {
    state: Arc<RwLock<AppState>>,
    ctx: egui::Context,
}

impl SharedState {
    pub fn new(state: AppState, ctx: egui::Context) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
            ctx,
        }
    }

    pub fn read(&self) -> parking_lot::RwLockReadGuard<'_, AppState> {
        self.state.read()
    }

    pub fn write(&self) -> parking_lot::RwLockWriteGuard<'_, AppState> {
        self.state.write()
    }

    /// Acquire write lock, apply a mutation, then request repaint.
    pub fn mutate(&self, f: impl FnOnce(&mut AppState)) {
        {
            let mut state = self.state.write();
            f(&mut state);
        }
        self.ctx.request_repaint();
    }

    pub fn ctx(&self) -> &egui::Context {
        &self.ctx
    }
}
