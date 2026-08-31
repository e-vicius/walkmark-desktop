use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use image::RgbaImage;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::detect;
use super::input;
use crate::error::{AppError, Result};
use crate::models::{CaptureSettings, RecordingState, RecordingTick, Step};
use crate::window;

pub const EVT_TICK: &str = "recording:tick";
pub const EVT_STEP: &str = "recording:step";
pub const EVT_ALTERNATE: &str = "recording:alternate";
pub const EVT_ERROR: &str = "recording:error";
pub const EVT_STOPPED: &str = "recording:stopped";

/// How many consecutive capture failures we tolerate before giving up. A window
/// that gets closed mid-recording shouldn't kill the session instantly, but it
/// also shouldn't spin forever.
const MAX_CONSECUTIVE_FAILURES: u32 = 6;

/// How many samples we'll wait for the screen to go quiet before committing a
/// step anyway. Without a limit, anything permanently in motion — a video, a
/// progress spinner, a blinking terminal — would suppress every step for the
/// whole session.
const MAX_SETTLE_WAITS: u32 = 4;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AlternateEvent {
    step_id: String,
    frame: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorEvent {
    message: String,
    fatal: bool,
}

pub struct SessionConfig {
    pub source_id: String,
    pub frames_dir: PathBuf,
    pub settings: CaptureSettings,
}

struct Control {
    stop: AtomicBool,
    paused: AtomicBool,
    /// Set by the "mark step" hotkey; consumed by the next sample.
    mark: AtomicBool,
    step_count: AtomicUsize,
    elapsed_ms: AtomicU64,
    state: Mutex<RecordingState>,
}

/// A live recording. Dropping this stops the worker thread.
pub struct Session {
    ctl: Arc<Control>,
    handle: Option<JoinHandle<()>>,
    pub source_label: String,
}

impl Session {
    pub fn start(app: AppHandle, cfg: SessionConfig) -> Result<Session> {
        if !super::has_permission() {
            return Err(AppError::PermissionDenied);
        }
        std::fs::create_dir_all(&cfg.frames_dir)?;

        let ctl = Arc::new(Control {
            stop: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            mark: AtomicBool::new(false),
            step_count: AtomicUsize::new(0),
            elapsed_ms: AtomicU64::new(0),
            state: Mutex::new(RecordingState::Counting),
        });

        // The capture target is resolved on the worker thread so it never has to
        // cross a thread boundary; the channel just relays "did it work".
        let (ready_tx, ready_rx) = mpsc::channel::<Result<String>>();
        let worker_ctl = Arc::clone(&ctl);
        let handle = thread::Builder::new()
            .name("walkmark-capture".into())
            .spawn(move || run(app, cfg, worker_ctl, ready_tx))
            .map_err(|e| AppError::Other(format!("Could not start the capture thread: {e}")))?;

        match ready_rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(source_label)) => Ok(Session {
                ctl,
                handle: Some(handle),
                source_label,
            }),
            Ok(Err(e)) => {
                let _ = handle.join();
                Err(e)
            }
            Err(_) => {
                ctl.stop.store(true, Ordering::SeqCst);
                Err(AppError::Capture(
                    "The capture source did not respond in time.".into(),
                ))
            }
        }
    }

    pub fn state(&self) -> RecordingState {
        *self.ctl.state.lock()
    }

    pub fn step_count(&self) -> usize {
        self.ctl.step_count.load(Ordering::Relaxed)
    }

    pub fn set_paused(&self, paused: bool) {
        self.ctl.paused.store(paused, Ordering::SeqCst);
    }

    pub fn mark(&self) {
        self.ctl.mark.store(true, Ordering::SeqCst);
    }

    pub fn stopping_tick(&self) -> RecordingTick {
        RecordingTick {
            state: RecordingState::Stopping,
            elapsed_ms: self.ctl.elapsed_ms.load(Ordering::Relaxed),
            step_count: self.step_count(),
            activity: 0.0,
            countdown: 0,
        }
    }

    /// Signals the worker and waits for it to finish writing the last frame.
    pub fn stop(mut self) {
        self.ctl.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.ctl.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run(
    app: AppHandle,
    cfg: SessionConfig,
    ctl: Arc<Control>,
    ready: mpsc::Sender<Result<String>>,
) {
    let target = match super::resolve(&cfg.source_id) {
        Ok(t) => t,
        Err(e) => {
            let _ = ready.send(Err(e));
            return;
        }
    };
    let label = target.label();
    if ready.send(Ok(label)).is_err() {
        return;
    }

    let s = &cfg.settings;
    let threshold = detect::threshold_for(s.sensitivity);
    let interval = Duration::from_millis(s.sample_interval_ms.clamp(200, 5000));

    // --- Countdown -------------------------------------------------------
    // Gives the user time to switch to the app they're documenting.
    let countdown_start = Instant::now();
    let countdown = Duration::from_secs(s.countdown_secs as u64);
    while countdown_start.elapsed() < countdown && !ctl.stop.load(Ordering::SeqCst) {
        let remaining = countdown.saturating_sub(countdown_start.elapsed());
        emit_tick(
            &app,
            &ctl,
            RecordingTick {
                state: RecordingState::Counting,
                elapsed_ms: 0,
                step_count: 0,
                activity: 0.0,
                countdown: remaining.as_secs() as u32 + 1,
            },
        );
        thread::sleep(Duration::from_millis(100));
    }

    *ctl.state.lock() = RecordingState::Recording;
    let started = Instant::now();
    // Time spent paused is excluded so step offsets stay meaningful.
    let mut paused_total = Duration::ZERO;
    let mut paused_since: Option<Instant> = None;

    let input_ok = input::ensure_listener();
    let use_input = input_ok;
    let use_visual = s.visual_fallback || !input_ok;
    if use_input {
        input::set_enabled(true);
    } else if !s.visual_fallback {
        let _ = app.emit(
            EVT_ERROR,
            ErrorEvent {
                message: "Input monitoring is off — enable Accessibility for Walkmark in System \
Settings, or turn on visual fallback in Recording settings."
                    .into(),
                fatal: false,
            },
        );
    }

    let input_settle = Duration::from_millis(s.input_settle_ms.clamp(0, 2000));
    let mut input_pending: Option<Instant> = None;
    let mut committed_sig: Option<detect::Signature> = None;
    let mut prev_sig: Option<detect::Signature> = None;
    let mut prev_frame: Option<RgbaImage> = None;
    let mut last_commit_at: Option<Duration> = None;
    // Step waiting for its "just after" alternate frame.
    let mut pending_alternate: Option<String> = None;
    let mut failures = 0u32;
    let mut settle_waits = 0u32;

    loop {
        let cycle_start = Instant::now();
        if ctl.stop.load(Ordering::SeqCst) {
            break;
        }

        // --- Pause -------------------------------------------------------
        if ctl.paused.load(Ordering::SeqCst) {
            if ctl.stop.load(Ordering::SeqCst) {
                break;
            }
            if paused_since.is_none() {
                paused_since = Some(Instant::now());
                *ctl.state.lock() = RecordingState::Paused;
            }
            emit_tick(
                &app,
                &ctl,
                RecordingTick {
                    state: RecordingState::Paused,
                    elapsed_ms: elapsed_ms(started, paused_total, paused_since),
                    step_count: ctl.step_count.load(Ordering::Relaxed),
                    activity: 0.0,
                    countdown: 0,
                },
            );
            thread::sleep(Duration::from_millis(40));
            continue;
        }
        if let Some(since) = paused_since.take() {
            paused_total += since.elapsed();
            *ctl.state.lock() = RecordingState::Recording;
            // Force a fresh baseline: the screen almost certainly changed while
            // we weren't watching, and that shouldn't count as one big step.
            prev_sig = None;
        }

        // --- Sample ------------------------------------------------------
        if ctl.stop.load(Ordering::SeqCst) {
            break;
        }
        // Frames are kept at full resolution until one is actually committed:
        // most samples are discarded, and resizing every one of them would cost
        // far more than the screen grab itself.
        let frame = match target.grab() {
            Ok(img) => {
                failures = 0;
                img
            }
            Err(e) => {
                failures += 1;
                if failures >= MAX_CONSECUTIVE_FAILURES {
                    let _ = app.emit(
                        EVT_ERROR,
                        ErrorEvent {
                            message: format!(
                                "Lost access to the capture source and stopped recording. {e}"
                            ),
                            fatal: true,
                        },
                    );
                    break;
                }
                thread::sleep(interval);
                continue;
            }
        };

        let sig = detect::signature(&frame);
        let activity = prev_sig.as_ref().map_or(0.0, |p| detect::distance(p, &sig));
        let drift = committed_sig
            .as_ref()
            .map_or(1.0, |c| detect::distance(c, &sig));
        let now = Duration::from_millis(elapsed_ms(started, paused_total, paused_since));

        // The frame right after a commit is kept as an alternate so the user can
        // fix a screenshot taken a beat too early.
        if let Some(step_id) = pending_alternate.take() {
            if let Ok(name) = write_frame(&cfg.frames_dir, &frame, now.as_millis() as u64, s.max_width)
            {
                let _ = app.emit(
                    EVT_ALTERNATE,
                    AlternateEvent {
                        step_id,
                        frame: name,
                    },
                );
            }
        }

        // --- Decide ------------------------------------------------------
        let forced = ctl.mark.swap(false, Ordering::SeqCst);
        let gap_ok = last_commit_at
            .map(|t| now.saturating_sub(t) >= Duration::from_millis(s.min_gap_ms))
            .unwrap_or(true);

        if use_input && input::take_trigger() {
            input_pending = Some(Instant::now());
        }

        let input_ready = input_pending.is_some_and(|t| t.elapsed() >= input_settle);

        // On the very first sample there is nothing to settle against, so treat
        // it as stable and capture the starting state immediately.
        let settled = !s.settle || prev_sig.is_none() || detect::is_settled(activity);
        let changed = drift > threshold;

        // Count how long we've been holding a change back waiting for calm, and
        // give up once the screen proves it isn't going to stop moving.
        settle_waits = if changed && gap_ok && !settled {
            settle_waits + 1
        } else {
            0
        };
        let waited_long_enough = settle_waits >= MAX_SETTLE_WAITS;
        let visual_commit =
            use_visual && changed && gap_ok && (settled || waited_long_enough);
        let input_commit = use_input && input_ready && gap_ok;
        let first_sample = committed_sig.is_none();

        if forced || first_sample || input_commit || visual_commit {
            if input_commit {
                input_pending = None;
            }
            settle_waits = 0;
            let manual = forced;
            match write_frame(&cfg.frames_dir, &frame, now.as_millis() as u64, s.max_width) {
                Ok(name) => {
                    let mut step = Step::new(name, now.as_millis() as u64, manual);
                    // Whatever was on screen a moment before is often the better
                    // shot (before a menu closed, before a toast vanished).
                    if let Some(prev) = prev_frame.as_ref() {
                        if let Ok(alt) = write_frame(
                            &cfg.frames_dir,
                            prev,
                            now.as_millis().saturating_sub(1) as u64,
                            s.max_width,
                        ) {
                            step.alternates.push(alt);
                        }
                    }
                    pending_alternate = Some(step.id.clone());
                    ctl.step_count.fetch_add(1, Ordering::Relaxed);
                    last_commit_at = Some(now);
                    committed_sig = Some(sig.clone());
                    let _ = app.emit(EVT_STEP, &step);
                }
                Err(e) => {
                    let _ = app.emit(
                        EVT_ERROR,
                        ErrorEvent {
                            message: format!("A screenshot could not be saved: {e}"),
                            fatal: false,
                        },
                    );
                }
            }
        }

        emit_tick(
            &app,
            &ctl,
            RecordingTick {
                state: RecordingState::Recording,
                elapsed_ms: now.as_millis() as u64,
                step_count: ctl.step_count.load(Ordering::Relaxed),
                activity,
                countdown: 0,
            },
        );

        prev_sig = Some(sig);
        prev_frame = Some(frame);

        // Keep a steady cadence regardless of how long capture + hashing took.
        let spent = cycle_start.elapsed();
        if spent < interval {
            let remaining = interval - spent;
            // Poll the stop flag rather than sleeping through it, so Stop feels
            // immediate even with a slow sample interval.
            let deadline = Instant::now() + remaining;
            while Instant::now() < deadline {
                if ctl.stop.load(Ordering::SeqCst)
                    || ctl.mark.load(Ordering::SeqCst)
                    || (use_input && input::has_trigger())
                {
                    break;
                }
                thread::sleep(Duration::from_millis(40).min(deadline - Instant::now()));
            }
        }
    }

    input::set_enabled(false);

    // Normal stop takes the session out of AppState before joining; only
    // unexpected worker exits need to tear down the HUD and clear state here.
    let orphan = app
        .try_state::<crate::state::AppState>()
        .is_some_and(|state| state.session.lock().is_some());

    if orphan {
        *ctl.state.lock() = RecordingState::Idle;
        emit_tick(
            &app,
            &ctl,
            RecordingTick {
                state: RecordingState::Idle,
                elapsed_ms: elapsed_ms(started, paused_total, paused_since),
                step_count: ctl.step_count.load(Ordering::Relaxed),
                activity: 0.0,
                countdown: 0,
            },
        );
        window::recording_worker_finished(&app);
    }
}

fn elapsed_ms(started: Instant, paused_total: Duration, paused_since: Option<Instant>) -> u64 {
    let extra = paused_since.map(|s| s.elapsed()).unwrap_or_default();
    started
        .elapsed()
        .saturating_sub(paused_total + extra)
        .as_millis() as u64
}

fn emit_tick(app: &AppHandle, ctl: &Control, tick: RecordingTick) {
    ctl.elapsed_ms.store(tick.elapsed_ms, Ordering::Relaxed);
    let _ = app.emit(EVT_TICK, tick);
}

/// Frame names lead with the offset so a plain directory listing is in
/// chronological order, and end with a random suffix to avoid collisions when
/// two frames land in the same millisecond.
fn write_frame(
    dir: &Path,
    image: &RgbaImage,
    offset_ms: u64,
    max_width: u32,
) -> Result<String> {
    let name = format!(
        "{:09}-{}.png",
        offset_ms,
        &uuid::Uuid::new_v4().simple().to_string()[..6]
    );
    // Retina captures are downscaled here, at the one point where it's worth
    // paying for a high-quality filter.
    let scaled = super::downscale(image.clone(), max_width);
    image::DynamicImage::ImageRgba8(scaled)
        .to_rgb8()
        .save(dir.join(&name))?;
    Ok(name)
}
