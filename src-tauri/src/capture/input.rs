//! Global mouse, keyboard and scroll listener.
//!
//! Steps are captured when the user interacts with the machine, not when the
//! screen happens to look different. One listener thread runs for the lifetime
//! of the app; recording sessions enable and drain it.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread::{self, JoinHandle};

use rdev::{Button, Event, EventType, Key};

struct Listener {
    enabled: Arc<AtomicBool>,
    triggered: Arc<AtomicBool>,
    _handle: JoinHandle<()>,
}

static LISTENER: OnceLock<Listener> = OnceLock::new();

thread_local! {
    static MODIFIERS: RefCell<ModifierState> = RefCell::new(ModifierState::default());
}

#[derive(Default)]
struct ModifierState {
    shift: bool,
    alt: bool,
}

/// Start the global listener once, but only when Accessibility is granted.
/// Returns false when input monitoring is unavailable on this machine.
pub fn ensure_listener() -> bool {
    if !has_input_permission() {
        return false;
    }

    LISTENER.get_or_init(|| {
        let enabled = Arc::new(AtomicBool::new(false));
        let triggered = Arc::new(AtomicBool::new(false));
        let worker_enabled = Arc::clone(&enabled);
        let worker_triggered = Arc::clone(&triggered);

        let handle = thread::Builder::new()
            .name("steppy-input".into())
            .spawn(move || {
                // rdev defaults to "main thread" mode; Tauri's listener runs on a
                // worker thread and will segfault on keypress without this (macOS).
                #[cfg(target_os = "macos")]
                rdev::set_is_main_thread(false);

                if let Err(error) = rdev::listen(move |event| {
                    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        handle_event(&event, &worker_enabled, &worker_triggered);
                    }))
                    .is_err()
                    {
                        eprintln!("steppy input listener dropped an event after a panic");
                    }
                }) {
                    eprintln!("steppy input listener stopped: {error:?}");
                }
            });

        let handle = match handle {
            Ok(handle) => handle,
            Err(error) => {
                eprintln!("steppy input listener could not start: {error}");
                return Listener {
                    enabled,
                    triggered,
                    _handle: thread::Builder::new()
                        .spawn(|| {})
                        .expect("spawn noop input listener fallback"),
                };
            }
        };

        Listener {
            enabled,
            triggered,
            _handle: handle,
        }
    });

    true
}

fn handle_event(event: &Event, enabled: &AtomicBool, triggered: &AtomicBool) {
    if !enabled.load(Ordering::Relaxed) {
        return;
    }
    if event_triggers_step(event) {
        triggered.store(true, Ordering::SeqCst);
    }
}

pub fn set_enabled(enabled: bool) {
    if let Some(listener) = LISTENER.get() {
        listener.enabled.store(enabled, Ordering::SeqCst);
        if !enabled {
            listener.triggered.store(false, Ordering::SeqCst);
        }
    }
}

pub fn take_trigger() -> bool {
    LISTENER
        .get()
        .is_some_and(|l| l.triggered.swap(false, Ordering::SeqCst))
}

pub fn has_trigger() -> bool {
    LISTENER
        .get()
        .is_some_and(|l| l.triggered.load(Ordering::Relaxed))
}

// ---------------------------------------------------------------------------
// macOS Accessibility — required for global input on macOS
// ---------------------------------------------------------------------------

pub fn has_input_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        accessibility_trusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[cfg(target_os = "macos")]
fn accessibility_trusted() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

#[cfg(target_os = "macos")]
pub fn request_input_permission() {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
    }
    // Passing null prompts the system dialog once.
    unsafe {
        AXIsProcessTrustedWithOptions(std::ptr::null());
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_input_permission() {}

fn event_triggers_step(event: &Event) -> bool {
    MODIFIERS.with(|mods| {
        let mut mods = mods.borrow_mut();
        match event.event_type {
            EventType::ButtonPress(Button::Left | Button::Right | Button::Middle) => true,
            EventType::Wheel { delta_x, delta_y } => delta_x != 0 || delta_y != 0,
            EventType::KeyPress(key) => {
                mods.on_press(key);
                if mods.is_steppy_shortcut(key) || is_modifier(key) {
                    false
                } else {
                    true
                }
            }
            EventType::KeyRelease(key) => {
                mods.on_release(key);
                false
            }
            _ => false,
        }
    })
}

impl ModifierState {
    fn on_press(&mut self, key: Key) {
        match key {
            Key::ShiftLeft | Key::ShiftRight => self.shift = true,
            Key::Alt | Key::AltGr => self.alt = true,
            _ => {}
        }
    }

    fn on_release(&mut self, key: Key) {
        match key {
            Key::ShiftLeft | Key::ShiftRight => self.shift = false,
            Key::Alt | Key::AltGr => self.alt = false,
            _ => {}
        }
    }

    fn is_steppy_shortcut(&self, key: Key) -> bool {
        self.shift
            && self.alt
            && matches!(key, Key::KeyS | Key::KeyM | Key::KeyP)
    }
}

fn is_modifier(key: Key) -> bool {
    matches!(
        key,
        Key::ShiftLeft
            | Key::ShiftRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::Alt
            | Key::AltGr
            | Key::MetaLeft
            | Key::MetaRight
            | Key::CapsLock
            | Key::NumLock
            | Key::ScrollLock
            | Key::Function
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steppy_shortcuts_are_ignored() {
        MODIFIERS.with(|mods| {
            *mods.borrow_mut() = ModifierState {
                shift: true,
                alt: true,
            };
        });
        assert!(!event_triggers_step(&Event {
            event_type: EventType::KeyPress(Key::KeyM),
            time: std::time::SystemTime::UNIX_EPOCH,
            name: None,
        }));
    }

    #[test]
    fn typing_triggers_a_step() {
        MODIFIERS.with(|mods| mods.borrow_mut().shift = false);
        assert!(event_triggers_step(&Event {
            event_type: EventType::KeyPress(Key::KeyA),
            time: std::time::SystemTime::UNIX_EPOCH,
            name: None,
        }));
    }

    #[test]
    fn clicks_and_scrolls_trigger() {
        assert!(event_triggers_step(&Event {
            event_type: EventType::ButtonPress(Button::Left),
            time: std::time::SystemTime::UNIX_EPOCH,
            name: None,
        }));
        assert!(event_triggers_step(&Event {
            event_type: EventType::Wheel {
                delta_x: 0,
                delta_y: 1,
            },
            time: std::time::SystemTime::UNIX_EPOCH,
            name: None,
        }));
    }
}
