use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::capture::session::Session;
use crate::models::{Project, Settings};

/// A model download in progress. Only one runs at a time: two multi-gigabyte
/// pulls competing for the same connection helps nobody.
pub struct Download {
    pub model: String,
    pub cancel: Arc<AtomicBool>,
}

/// The backend owns the document. The UI keeps a mirror for rendering, but
/// every mutation round-trips through here so background work (AI generation,
/// autosave, the capture thread) can never race with the editor.
#[derive(Default)]
pub struct AppState {
    pub settings: Mutex<Settings>,
    pub project: Mutex<Option<Project>>,
    pub session: Mutex<Option<Session>>,
    /// Set while a generation run is in flight; flipping it cancels the run.
    pub generation: Mutex<Option<Arc<AtomicBool>>>,
    pub download: Mutex<Option<Download>>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Mutex::new(settings),
            ..Default::default()
        }
    }

    pub fn is_generating(&self) -> bool {
        self.generation.lock().is_some()
    }
}
