mod ai;
mod capture;
mod commands;
mod error;
mod export;
mod imaging;
pub mod local;
mod models;
mod state;
mod storage;
mod window;

use tauri::Manager;

use state::AppState;

/// Thin re-exports for `examples/probe_capture.rs`, which checks the capture
/// backend on a real machine without booting the whole app.
pub mod probe {
    use std::time::{Duration, Instant};

    pub use crate::capture::detect::threshold_for;
    pub use crate::capture::{has_permission, list_sources};
    pub use crate::export::{
        html, markdown, pdf, ExportOptions, RenderedImage, RenderedStep,
    };
    pub use crate::imaging::{apply_annotations, encode_png, fit_width, load};
    pub use crate::models::{ExportFormat, Project, Provider};

    /// Exposed for `tests/wire.rs`, which drives the provider clients and the
    /// model downloader against a stub HTTP server.
    pub use crate::ai::provider;
    pub use crate::local;

    pub fn grab(source_id: &str) -> Result<(u32, u32, Duration), String> {
        let target = crate::capture::resolve(source_id).map_err(|e| e.to_string())?;
        let image = target.grab().map_err(|e| e.to_string())?;
        let started = Instant::now();
        let _ = crate::capture::detect::signature(&image);
        Ok((image.width(), image.height(), started.elapsed()))
    }

    /// One sample of the detector's view of the screen, for calibrating the
    /// sensitivity curve against what real applications actually do.
    pub struct Sample {
        pub at: Duration,
        /// Change since the previous sample.
        pub activity: f32,
        /// Change since the last frame the detector would have committed.
        pub drift: f32,
        pub committed: bool,
    }

    /// Replays the live detector over the real screen for `duration`, reporting
    /// what it would have captured at the given sensitivity.
    pub fn trace(
        source_id: &str,
        duration: Duration,
        interval: Duration,
        sensitivity: f32,
        min_gap: Duration,
    ) -> Result<Vec<Sample>, String> {
        use crate::capture::detect;

        let target = crate::capture::resolve(source_id).map_err(|e| e.to_string())?;
        let threshold = detect::threshold_for(sensitivity);
        let started = Instant::now();

        let mut out = Vec::new();
        let mut prev: Option<detect::Signature> = None;
        let mut committed: Option<detect::Signature> = None;
        let mut last_commit: Option<Duration> = None;

        while started.elapsed() < duration {
            let cycle = Instant::now();
            let image = target.grab().map_err(|e| e.to_string())?;
            let sig = detect::signature(&image);
            let at = started.elapsed();

            let activity = prev.as_ref().map_or(0.0, |p| detect::distance(p, &sig));
            let drift = committed.as_ref().map_or(1.0, |c| detect::distance(c, &sig));
            let gap_ok = last_commit.is_none_or(|t| at.saturating_sub(t) >= min_gap);
            let settled = prev.is_none() || detect::is_settled(activity);
            let commit = drift > threshold && gap_ok && settled;

            if commit {
                committed = Some(sig.clone());
                last_commit = Some(at);
            }
            out.push(Sample {
                at,
                activity,
                drift,
                committed: commit,
            });
            prev = Some(sig);

            if let Some(rest) = interval.checked_sub(cycle.elapsed()) {
                std::thread::sleep(rest);
            }
        }
        Ok(out)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let settings = storage::load_settings(&handle);
            app.manage(AppState::new(settings));

            // The window starts hidden so the user never sees an unstyled flash
            // while the frontend boots; the UI reveals it when it's ready.
            if let Some(main) = app.get_webview_window(window::MAIN) {
                let _ = main.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() != window::MAIN {
                    return;
                }
                // Closing mid-recording must stop the capture thread, otherwise
                // it keeps writing frames for a document nobody can see.
                let state = window.state::<AppState>();
                let session = state.session.lock().take();
                if let Some(session) = session {
                    session.stop();
                }
                let cancel = state.generation.lock().clone();
                if let Some(flag) = cancel {
                    flag.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                // A half-finished model download would otherwise keep the
                // process alive after the window is gone.
                let download = state
                    .download
                    .lock()
                    .as_ref()
                    .map(|d| std::sync::Arc::clone(&d.cancel));
                if let Some(flag) = download {
                    flag.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::permission_status,
            commands::request_permission,
            commands::open_privacy_settings,
            commands::provider_catalog,
            commands::set_api_key,
            commands::clear_api_key,
            commands::verify_provider,
            commands::local_status,
            commands::download_model,
            commands::cancel_download,
            commands::remove_model,
            commands::list_sources,
            commands::recording_status,
            commands::start_recording,
            commands::pause_recording,
            commands::mark_step,
            commands::stop_recording,
            commands::append_step,
            commands::attach_alternate,
            commands::list_projects,
            commands::open_project,
            commands::current_project,
            commands::close_project,
            commands::delete_project,
            commands::update_project_meta,
            commands::frame_path,
            commands::update_step,
            commands::reorder_steps,
            commands::delete_steps,
            commands::merge_steps,
            commands::generate,
            commands::cancel_generation,
            commands::suggest_export_name,
            commands::export_document,
            commands::copy_as_markdown,
            commands::reveal_in_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Steppy");
}
