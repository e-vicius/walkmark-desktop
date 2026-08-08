use std::sync::atomic::{AtomicBool, Ordering};

use tauri::webview::Color;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub const MAIN: &str = "main";
pub const HUD: &str = "hud";

const HUD_W: f64 = 300.0;
const HUD_H: f64 = 84.0;
/// Distance from the bottom of the screen, clear of the macOS Dock.
const HUD_BOTTOM_INSET: f64 = 96.0;

static SHORTCUTS_REGISTERED: AtomicBool = AtomicBool::new(false);

/// Swaps the app into "get out of the way" mode: the main window steps aside
/// and a small always-on-top HUD takes over, so the user can actually see the
/// thing they're documenting.
pub fn enter_recording_mode(app: &AppHandle, hide_main: bool) {
    if let Err(e) = open_hud(app) {
        // The HUD is a convenience. Losing it must not abort the recording.
        let _ = app.emit(
            "recording:error",
            serde_json::json!({
                "message": format!("The floating controls could not be shown: {e}"),
                "fatal": false
            }),
        );
    }
    if hide_main {
        if let Some(main) = app.get_webview_window(MAIN) {
            let _ = main.minimize();
        }
    }
    register_shortcuts(app);
}

pub fn leave_recording_mode(app: &AppHandle) {
    unregister_shortcuts(app);
    if let Some(hud) = app.get_webview_window(HUD) {
        let _ = hud.close();
    }
    if let Some(main) = app.get_webview_window(MAIN) {
        let _ = main.unminimize();
        let _ = main.show();
        let _ = main.set_focus();
    }
}

/// Called when the capture worker exits on its own (fatal error, lost source).
pub fn recording_worker_finished(app: &AppHandle) {
    leave_recording_mode(app);
    if let Some(state) = app.try_state::<crate::state::AppState>() {
        *state.session.lock() = None;
    }
}

fn open_hud(app: &AppHandle) -> tauri::Result<()> {
    if let Some(existing) = app.get_webview_window(HUD) {
        existing.show()?;
        return Ok(());
    }

    let mut builder = WebviewWindowBuilder::new(app, HUD, WebviewUrl::App("index.html".into()))
        .title("Steppy recording")
        .inner_size(HUD_W, HUD_H)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .background_color(Color(0, 0, 0, 0))
        .shadow(false)
        .always_on_top(true)
        .devtools(false)
        .skip_taskbar(true)
        // Recording follows the user between Spaces, so the controls must too.
        .visible_on_all_workspaces(true)
        // Taking focus would pull the user out of the app they're documenting.
        .focused(false);

    if let Ok(Some(monitor)) = app.primary_monitor() {
        let scale = monitor.scale_factor();
        let size = monitor.size().to_logical::<f64>(scale);
        let position = monitor.position().to_logical::<f64>(scale);
        builder = builder.position(
            position.x + (size.width - HUD_W) / 2.0,
            position.y + size.height - HUD_H - HUD_BOTTOM_INSET,
        );
    }

    builder.build()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Global shortcuts
// ---------------------------------------------------------------------------

/// Registered only while recording. Holding these globally the whole time would
/// take shortcuts away from every other app for no reason.
fn shortcuts() -> [(Shortcut, &'static str); 3] {
    let modifiers = Modifiers::SHIFT | Modifiers::ALT;
    [
        (Shortcut::new(Some(modifiers), Code::KeyS), "stop"),
        (Shortcut::new(Some(modifiers), Code::KeyM), "mark"),
        (Shortcut::new(Some(modifiers), Code::KeyP), "pause"),
    ]
}

fn register_shortcuts(app: &AppHandle) {
    if SHORTCUTS_REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }
    for (shortcut, action) in shortcuts() {
        let action = action.to_string();
        let _ = app
            .global_shortcut()
            .on_shortcut(shortcut, move |app, _shortcut, event| {
                // Fire on press only; otherwise every keystroke acts twice.
                if event.state() == ShortcutState::Pressed {
                    let _ = app.emit("recording:shortcut", action.clone());
                }
            });
    }
}

fn unregister_shortcuts(app: &AppHandle) {
    if !SHORTCUTS_REGISTERED.swap(false, Ordering::SeqCst) {
        return;
    }
    for (shortcut, _) in shortcuts() {
        let _ = app.global_shortcut().unregister(shortcut);
    }
}
