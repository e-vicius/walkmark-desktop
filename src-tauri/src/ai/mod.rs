pub mod catalog;
pub mod prompt;
pub mod provider;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Semaphore;

use crate::error::{AppError, Result};
use crate::imaging;
use crate::models::{GenerationProgress, Project, Settings, Step, StepStatus};
use crate::state::AppState;
use crate::storage;

pub const EVT_PROGRESS: &str = "ai:progress";
pub const EVT_STEP: &str = "ai:step";
pub const EVT_OUTLINE: &str = "ai:outline";
pub const EVT_DONE: &str = "ai:done";

/// Frames sent for the whole-task outline are small on purpose: we only need
/// enough detail to tell the steps apart, and 24 large images would be slow
/// and expensive for no benefit.
const OUTLINE_IMAGE_WIDTH: u32 = 640;
const OUTLINE_MAX_IMAGES: usize = 24;

/// Detail frames. Every provider downsamples somewhere above this, and 1280
/// keeps small UI labels legible.
const STEP_IMAGE_WIDTH: u32 = 1280;

/// Local models are slower and have far less context to spend, so they get
/// smaller pictures and fewer of them. Sending a 1280px screenshot to a 4B
/// model on a laptop is how you wait two minutes for a worse sentence.
const LOCAL_STEP_IMAGE_WIDTH: u32 = 896;
const LOCAL_OUTLINE_MAX_IMAGES: usize = 10;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutlineEvent {
    title: String,
    summary: String,
    prerequisites: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DoneEvent {
    cancelled: bool,
    succeeded: usize,
    failed: usize,
}

/// Which steps a run should touch.
pub enum Scope {
    /// Everything that has never been written, leaving edited steps alone.
    Missing,
    /// Everything included in the document, overwriting existing text.
    All,
    /// Specific steps, regardless of their lock state.
    Only(Vec<String>),
}

/// Builds a client from whatever the user has configured, so no caller needs to
/// know which provider is active or where its credentials live.
pub fn client_for(app: &AppHandle, settings: &Settings) -> Result<provider::Client> {
    let config = settings.active();
    let api_key = if settings.provider.needs_key() {
        storage::get_api_key(app, settings.provider).ok_or(AppError::MissingApiKey)?
    } else {
        String::new()
    };
    provider::Client::new(settings.provider, config.model, config.base_url, api_key)
}

pub async fn run(app: AppHandle, scope: Scope, cancel: Arc<AtomicBool>) -> Result<()> {
    let state = app.state::<AppState>();
    let settings: Settings = state.settings.lock().clone();
    let local = settings.provider.is_local();

    let project: Project = state
        .project
        .lock()
        .clone()
        .ok_or_else(|| AppError::NotFound("No document is open.".into()))?;

    let targets = select_targets(&project, &scope);
    if targets.is_empty() {
        emit_done(&app, false, 0, 0);
        return Ok(());
    }

    let client = Arc::new(client_for(&app, &settings)?);
    let vocabulary = settings
        .resolve_product(project.product_id.as_deref())
        .map(|p| p.format_vocabulary())
        .filter(|text| !text.is_empty());
    let system = prompt::system_instruction(&settings, vocabulary.as_deref());
    let frames_dir = storage::frames_dir(&app, &project.id)?;

    let total = targets.len();
    emit_progress(&app, 0, total, true, Some("Reviewing the recording".into()));

    // --- Pass 1: understand the whole task -------------------------------
    let write_outline = matches!(scope, Scope::All | Scope::Missing) && total > 1;
    let outline = if write_outline {
        match build_outline(&client, &system, &project, &frames_dir, local).await {
            Ok(o) => Some(o),
            Err(e) => {
                // An outline is a quality boost, not a requirement — a failure
                // here shouldn't cost the user the whole run.
                emit_progress(
                    &app,
                    0,
                    total,
                    true,
                    Some(format!("Continuing without an overview ({e})")),
                );
                None
            }
        }
    } else {
        None
    };

    let included: Vec<String> = project
        .steps
        .iter()
        .filter(|s| s.include)
        .map(|s| s.id.clone())
        .collect();

    if let Some(outline) = &outline {
        let mut guard = state.project.lock();
        if let Some(p) = guard.as_mut() {
            if !outline.title.trim().is_empty() {
                p.title = outline.title.trim().to_string();
            }
            p.summary = outline.summary.trim().to_string();
            p.prerequisites = outline
                .prerequisites
                .iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            p.touch();
            let _ = storage::save_project(&app, p);
            let _ = app.emit(
                EVT_OUTLINE,
                OutlineEvent {
                    title: p.title.clone(),
                    summary: p.summary.clone(),
                    prerequisites: p.prerequisites.clone(),
                },
            );
        }
    }

    // Planned titles give every parallel worker the same view of what comes
    // before it, which is what keeps the finished document coherent.
    let planned: Vec<(String, String)> = match &outline {
        Some(o) => included
            .iter()
            .cloned()
            .zip(o.steps.iter().cloned())
            .collect(),
        None => Vec::new(),
    };
    let planned_title = |id: &str| -> String {
        planned
            .iter()
            .find(|(sid, _)| sid == id)
            .map(|(_, t)| t.clone())
            .unwrap_or_default()
    };

    let task_title = state
        .project
        .lock()
        .as_ref()
        .map(|p| p.title.clone())
        .unwrap_or_default();

    // --- Pass 2: write each step ----------------------------------------
    mark_queued(&app, &targets);

    // One local model serves one request at a time no matter how many we send,
    // so parallelism there only buys queueing and memory pressure.
    let concurrency = if local {
        1
    } else {
        settings.concurrency.clamp(1, 8)
    };
    let permits = Arc::new(Semaphore::new(concurrency));
    let mut set = tokio::task::JoinSet::new();

    for step_id in targets.clone() {
        let Some((position, step)) = position_of(&state, &step_id) else {
            continue;
        };

        let earlier: Vec<String> = included
            .iter()
            .take_while(|id| **id != step_id)
            .map(|id| {
                let t = planned_title(id);
                if t.is_empty() {
                    title_of(&state, id)
                } else {
                    t
                }
            })
            .filter(|t| !t.is_empty())
            .collect();
        // Only the nearest few matter; sending twenty makes the prompt long
        // without making the writing better.
        let preceding: Vec<String> = earlier
            .iter()
            .skip(earlier.len().saturating_sub(4))
            .cloned()
            .collect();

        let text_prompt = prompt::step_prompt(
            position,
            included.len().max(1),
            &task_title,
            &planned_title(&step_id),
            &preceding,
        );

        let client = Arc::clone(&client);
        let permits = Arc::clone(&permits);
        let cancel = Arc::clone(&cancel);
        let system = system.clone();
        let frame_path = frames_dir.join(&step.frame);
        let annotations = step.annotations.clone();

        set.spawn(async move {
            let _permit = permits.acquire().await;
            if cancel.load(Ordering::SeqCst) {
                return (step_id, Err(AppError::Cancelled));
            }

            let width = if local {
                LOCAL_STEP_IMAGE_WIDTH
            } else {
                STEP_IMAGE_WIDTH
            };
            let image = match prepare_image(&frame_path, &annotations, width) {
                Ok(bytes) => bytes,
                Err(e) => return (step_id, Err(e)),
            };

            let result = client.describe(&system, &text_prompt, image).await;
            (step_id, result)
        });
    }

    let mut done = 0usize;
    let mut failed = 0usize;
    while let Some(joined) = set.join_next().await {
        let Ok((step_id, result)) = joined else {
            continue;
        };
        done += 1;

        let updated = {
            let mut guard = state.project.lock();
            let Some(project) = guard.as_mut() else {
                break;
            };
            let Some(step) = project.steps.iter_mut().find(|s| s.id == step_id) else {
                continue;
            };
            match result {
                Ok(text) => {
                    step.title = text.title.trim().trim_end_matches('.').to_string();
                    step.body = text.body.trim().to_string();
                    step.status = StepStatus::Ready;
                    step.error = None;
                    step.locked = false;
                }
                Err(AppError::Cancelled) => {
                    step.status = StepStatus::Draft;
                    step.error = None;
                }
                Err(e) => {
                    failed += 1;
                    step.status = StepStatus::Failed;
                    step.error = Some(e.to_string());
                }
            }
            let updated = step.clone();
            project.touch();
            updated
        };

        let _ = app.emit(EVT_STEP, &updated);
        emit_progress(&app, done, total, true, None);

        // Persist as we go: a crash or a quit halfway through a long run should
        // never throw away work the user already paid for.
        if let Some(project) = state.project.lock().as_ref() {
            let _ = storage::save_project(&app, project);
        }

        if cancel.load(Ordering::SeqCst) {
            set.abort_all();
        }
    }

    let cancelled = cancel.load(Ordering::SeqCst);
    emit_progress(&app, done, total, false, None);
    emit_done(&app, cancelled, done.saturating_sub(failed), failed);
    Ok(())
}

fn select_targets(project: &Project, scope: &Scope) -> Vec<String> {
    match scope {
        Scope::Only(ids) => project
            .steps
            .iter()
            .filter(|s| ids.contains(&s.id))
            .map(|s| s.id.clone())
            .collect(),
        Scope::All => project
            .steps
            .iter()
            .filter(|s| s.include && !s.locked)
            .map(|s| s.id.clone())
            .collect(),
        Scope::Missing => project
            .steps
            .iter()
            .filter(|s| s.include && !s.locked && s.status != StepStatus::Ready)
            .map(|s| s.id.clone())
            .collect(),
    }
}

async fn build_outline(
    client: &provider::Client,
    system: &str,
    project: &Project,
    frames_dir: &std::path::Path,
    local: bool,
) -> Result<provider::Outline> {
    let included: Vec<&Step> = project.steps.iter().filter(|s| s.include).collect();
    let budget = if local {
        LOCAL_OUTLINE_MAX_IMAGES
    } else {
        OUTLINE_MAX_IMAGES
    };
    // Long recordings get an evenly spread sample rather than just the first N,
    // so the outline still reflects the end of the task.
    let stride = (included.len() as f32 / budget as f32).ceil().max(1.0) as usize;
    let sampled: Vec<&Step> = included.iter().copied().step_by(stride).collect();

    let mut images = Vec::with_capacity(sampled.len());
    for step in &sampled {
        images.push(prepare_image(
            &frames_dir.join(&step.frame),
            &step.annotations,
            OUTLINE_IMAGE_WIDTH,
        )?);
    }

    let mut outline = client
        .outline(system, &prompt::outline_prompt(images.len()), images)
        .await?;

    // When we sampled, the returned titles line up with the sample, not with
    // every step; stretch them back out so downstream indexing stays honest.
    if stride > 1 {
        let mut expanded = Vec::with_capacity(included.len());
        for i in 0..included.len() {
            expanded.push(outline.steps.get(i / stride).cloned().unwrap_or_default());
        }
        outline.steps = expanded;
    }
    outline.steps.resize(included.len(), String::new());
    Ok(outline)
}

fn prepare_image(
    path: &std::path::Path,
    annotations: &[crate::models::Annotation],
    width: u32,
) -> Result<provider::InlineImage> {
    let image = imaging::load(path)?;
    // Annotations are burned in first so redacted regions never leave the
    // machine, then the result is shrunk for the request.
    let image = imaging::apply_annotations(image, annotations);
    let image = imaging::fit_width(image, width);
    Ok(provider::InlineImage(imaging::encode_jpeg(&image, 82)?))
}

fn position_of(state: &AppState, step_id: &str) -> Option<(usize, Step)> {
    let guard = state.project.lock();
    let project = guard.as_ref()?;
    let included: Vec<&Step> = project.steps.iter().filter(|s| s.include).collect();
    let position = included.iter().position(|s| s.id == step_id)?;
    Some((position, included[position].clone()))
}

fn title_of(state: &AppState, step_id: &str) -> String {
    state
        .project
        .lock()
        .as_ref()
        .and_then(|p| p.steps.iter().find(|s| s.id == step_id))
        .map(|s| s.title.clone())
        .unwrap_or_default()
}

/// Flip every target to "queued" up front so the list shows the full run
/// immediately instead of revealing it one row at a time.
fn mark_queued(app: &AppHandle, targets: &[String]) {
    let state = app.state::<AppState>();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return;
    };
    for step in project.steps.iter_mut() {
        if targets.contains(&step.id) {
            step.status = StepStatus::Queued;
            step.error = None;
            let _ = app.emit(EVT_STEP, &step.clone());
        }
    }
}

fn emit_progress(app: &AppHandle, done: usize, total: usize, running: bool, message: Option<String>) {
    let _ = app.emit(
        EVT_PROGRESS,
        GenerationProgress {
            done,
            total,
            running,
            message,
        },
    );
}

fn emit_done(app: &AppHandle, cancelled: bool, succeeded: usize, failed: usize) {
    let _ = app.emit(
        EVT_DONE,
        DoneEvent {
            cancelled,
            succeeded,
            failed,
        },
    );
}
