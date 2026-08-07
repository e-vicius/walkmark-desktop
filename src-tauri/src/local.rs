//! Managing models that run on this machine.
//!
//! Steppy does not ship an inference engine. It drives [Ollama], which is the
//! one thing on every desktop platform that already solves GPU detection,
//! quantisation and weight storage. What Steppy adds is that you never have to
//! open a terminal: the app finds the daemon, lists what you have, downloads
//! what you don't, and reports progress while it happens.
//!
//! [Ollama]: https://ollama.com

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::ai::catalog;
use crate::error::{AppError, Result};

pub const EVT_PULL: &str = "local:pull";

/// Long enough to survive a slow first byte, short enough that a wrong endpoint
/// doesn't leave the Settings pane spinning.
const PROBE_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledModel {
    pub id: String,
    /// Bytes on disk.
    pub size: u64,
    /// e.g. "8.3B", straight from the daemon.
    pub parameters: String,
    pub quantization: String,
    /// Whether the model can actually look at a screenshot. A text-only model
    /// downloaded by mistake is the single most confusing thing that can happen
    /// here, so we mark it rather than letting it fail at generation time.
    pub vision: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStatus {
    /// Whether the daemon answered.
    pub running: bool,
    pub version: Option<String>,
    pub endpoint: String,
    pub models: Vec<InstalledModel>,
    /// Total physical memory, so the UI can say which models are realistic.
    pub total_memory: u64,
    pub download_url: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullProgress {
    pub model: String,
    /// Short human phrase straight from the daemon, e.g. "pulling manifest".
    pub status: String,
    pub completed: u64,
    pub total: u64,
    pub done: bool,
    pub error: Option<String>,
}

fn client(timeout: Duration) -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        // A proxy meant for the internet must not intercept a loopback call.
        .no_proxy()
        .connect_timeout(Duration::from_secs(3))
        .timeout(timeout)
        .build()?)
}

fn unreachable(endpoint: &str) -> AppError {
    AppError::LocalRuntimeUnavailable(format!(
        "Could not reach Ollama at {endpoint}. Make sure it is installed and running."
    ))
}

/// Everything the Local pane needs, in one round trip per model.
pub async fn status(endpoint: &str) -> LocalStatus {
    let endpoint = normalize(endpoint);
    let mut status = LocalStatus {
        running: false,
        version: None,
        endpoint: endpoint.clone(),
        models: Vec::new(),
        total_memory: total_memory(),
        download_url: "https://ollama.com/download",
    };

    let Ok(http) = client(PROBE_TIMEOUT) else {
        return status;
    };

    let Ok(response) = http.get(format!("{endpoint}/api/version")).send().await else {
        return status;
    };
    if !response.status().is_success() {
        return status;
    }
    status.running = true;
    status.version = response
        .json::<Value>()
        .await
        .ok()
        .and_then(|v| v.get("version").and_then(Value::as_str).map(str::to_owned));

    status.models = installed(&http, &endpoint).await.unwrap_or_default();
    status
}

async fn installed(http: &reqwest::Client, endpoint: &str) -> Result<Vec<InstalledModel>> {
    let payload: Value = http
        .get(format!("{endpoint}/api/tags"))
        .send()
        .await
        .map_err(|_| unreachable(endpoint))?
        .json()
        .await
        .map_err(|_| unreachable(endpoint))?;

    let entries = payload
        .get("models")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut models = Vec::with_capacity(entries.len());
    for entry in entries {
        let Some(id) = entry
            .get("name")
            .or_else(|| entry.get("model"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        models.push(InstalledModel {
            id: id.to_string(),
            size: entry.get("size").and_then(Value::as_u64).unwrap_or(0),
            parameters: string_at(&entry, "/details/parameter_size"),
            quantization: string_at(&entry, "/details/quantization_level"),
            vision: supports_vision(http, endpoint, id).await,
        });
    }
    models.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(models)
}

fn string_at(value: &Value, pointer: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// Recent Ollama reports capabilities directly. Older builds don't, so fall
/// back to the projector field that multimodal models carry.
async fn supports_vision(http: &reqwest::Client, endpoint: &str, model: &str) -> bool {
    let Ok(response) = http
        .post(format!("{endpoint}/api/show"))
        .json(&serde_json::json!({ "model": model }))
        .send()
        .await
    else {
        return false;
    };
    let Ok(payload) = response.json::<Value>().await else {
        return false;
    };
    detect_vision(&payload)
}

fn detect_vision(payload: &Value) -> bool {
    if let Some(capabilities) = payload.get("capabilities").and_then(Value::as_array) {
        return capabilities
            .iter()
            .filter_map(Value::as_str)
            .any(|c| c == "vision");
    }
    if payload.get("projector_info").is_some() {
        return true;
    }
    payload
        .pointer("/details/families")
        .and_then(Value::as_array)
        .map(|families| {
            families
                .iter()
                .filter_map(Value::as_str)
                .any(|f| matches!(f, "clip" | "mllama" | "qwen2vl" | "qwen3vl" | "gemma3"))
        })
        .unwrap_or(false)
}

/// Downloads a model, emitting progress to the frontend as it goes.
pub async fn pull(
    app: AppHandle,
    endpoint: &str,
    model: &str,
    cancel: Arc<AtomicBool>,
) -> Result<()> {
    pull_with(endpoint, model, cancel, |progress| {
        let _ = app.emit(EVT_PULL, progress);
    })
    .await
}

/// Downloads a model, reporting progress through a callback.
///
/// Ollama answers `/api/pull` with a stream of JSON lines rather than one
/// response, so this reads the body as it arrives and forwards each update.
pub async fn pull_with(
    endpoint: &str,
    model: &str,
    cancel: Arc<AtomicBool>,
    mut report: impl FnMut(PullProgress),
) -> Result<()> {
    let endpoint = normalize(endpoint);
    let model = model.trim().to_string();
    if model.is_empty() {
        return Err(AppError::Invalid("No model name was given.".into()));
    }

    // A large model over a slow connection can genuinely take an hour.
    let http = client(Duration::from_secs(60 * 60 * 4))?;
    let mut response = http
        .post(format!("{endpoint}/api/pull"))
        .json(&serde_json::json!({ "model": model, "stream": true }))
        .send()
        .await
        .map_err(|_| unreachable(&endpoint))?;

    if !response.status().is_success() {
        let status = response.status();
        let payload = response.text().await.unwrap_or_default();
        let message = serde_json::from_str::<Value>(&payload)
            .ok()
            .and_then(|v| v.get("error").and_then(Value::as_str).map(str::to_owned))
            .unwrap_or_else(|| format!("Ollama returned HTTP {status}."));
        report(failed(&model, &message));
        return Err(AppError::ApiRejected(message));
    }

    let mut buffer = String::new();
    let mut latest = PullProgress {
        model: model.clone(),
        status: "starting".into(),
        completed: 0,
        total: 0,
        done: false,
        error: None,
    };
    report(latest.clone());

    loop {
        if cancel.load(Ordering::SeqCst) {
            // Dropping the response aborts the transfer. Ollama keeps the blobs
            // it already wrote, so resuming later picks up where this left off.
            drop(response);
            return Err(AppError::Cancelled);
        }

        let chunk = match response.chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => break,
            Err(e) => {
                let message = format!("The download stopped unexpectedly: {e}");
                report(failed(&model, &message));
                return Err(AppError::ApiRejected(message));
            }
        };

        buffer.push_str(&String::from_utf8_lossy(&chunk));
        // The final line of a chunk is usually incomplete; leave it in the
        // buffer for the next one.
        while let Some(newline) = buffer.find('\n') {
            let line: String = buffer.drain(..=newline).collect();
            let Some(update) = parse_line(&model, line.trim()) else {
                continue;
            };
            if let Some(error) = &update.error {
                let message = error.clone();
                report(update);
                return Err(AppError::ApiRejected(message));
            }
            latest = update;
            report(latest.clone());
        }
    }

    latest.done = true;
    latest.status = "ready".into();
    if latest.total > 0 {
        latest.completed = latest.total;
    }
    report(latest);
    Ok(())
}

fn parse_line(model: &str, line: &str) -> Option<PullProgress> {
    if line.is_empty() {
        return None;
    }
    let value: Value = serde_json::from_str(line).ok()?;
    if let Some(error) = value.get("error").and_then(Value::as_str) {
        return Some(failed(model, error));
    }
    Some(PullProgress {
        model: model.to_string(),
        status: value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("downloading")
            .to_string(),
        completed: value.get("completed").and_then(Value::as_u64).unwrap_or(0),
        total: value.get("total").and_then(Value::as_u64).unwrap_or(0),
        done: value.get("status").and_then(Value::as_str) == Some("success"),
        error: None,
    })
}

fn failed(model: &str, message: &str) -> PullProgress {
    PullProgress {
        model: model.to_string(),
        status: "failed".into(),
        completed: 0,
        total: 0,
        done: true,
        error: Some(message.to_string()),
    }
}

pub async fn remove(endpoint: &str, model: &str) -> Result<()> {
    let endpoint = normalize(endpoint);
    let http = client(Duration::from_secs(30))?;
    let response = http
        .delete(format!("{endpoint}/api/delete"))
        .json(&serde_json::json!({ "model": model }))
        .send()
        .await
        .map_err(|_| unreachable(&endpoint))?;

    if response.status().is_success() {
        Ok(())
    } else if response.status().as_u16() == 404 {
        Err(AppError::NotFound(format!(
            "`{model}` is not installed any more."
        )))
    } else {
        Err(AppError::ApiRejected(format!(
            "Ollama could not remove `{model}` (HTTP {}).",
            response.status()
        )))
    }
}

/// Downloadable models, annotated with what's already here and whether this
/// machine can realistically run them.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub note: &'static str,
    pub size: u64,
    pub min_memory: u64,
    pub recommended: bool,
    pub installed: bool,
    /// False when this machine has less memory than the model wants.
    pub fits: bool,
}

pub fn catalog(installed: &[InstalledModel], total_memory: u64) -> Vec<CatalogEntry> {
    catalog::local_catalog()
        .into_iter()
        .map(|entry| CatalogEntry {
            id: entry.id,
            name: entry.name,
            note: entry.note,
            size: entry.size,
            min_memory: entry.min_memory,
            recommended: entry.recommended,
            installed: installed
                .iter()
                .any(|m| crate::ai::provider::ollama_name_matches(&m.id, entry.id)),
            // Unknown memory (an unsupported platform) should not stop anybody.
            fits: total_memory == 0 || total_memory >= entry.min_memory,
        })
        .collect()
}

fn normalize(endpoint: &str) -> String {
    let endpoint = endpoint.trim().trim_end_matches('/');
    if endpoint.is_empty() {
        crate::models::Provider::Ollama.default_base_url().to_string()
    } else if endpoint.contains("://") {
        endpoint.to_string()
    } else {
        format!("http://{endpoint}")
    }
}

/// Physical memory in bytes, or 0 where we can't tell.
fn total_memory() -> u64 {
    #[cfg(target_os = "macos")]
    {
        let mut value: u64 = 0;
        let mut size = std::mem::size_of::<u64>();
        let name = c"hw.memsize";
        // SAFETY: `hw.memsize` is a u64 sysctl; we pass a matching buffer and
        // its size, and ignore the value unless the call reports success.
        let ok = unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                &mut value as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) == 0
        };
        if ok {
            return value;
        }
        0
    }
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|text| {
                text.lines()
                    .find(|l| l.starts_with("MemTotal:"))?
                    .split_whitespace()
                    .nth(1)?
                    .parse::<u64>()
                    .ok()
            })
            .map(|kb| kb * 1024)
            .unwrap_or(0)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn endpoints_are_normalized_into_something_reqwest_accepts() {
        assert_eq!(normalize(""), "http://127.0.0.1:11434");
        assert_eq!(normalize("localhost:11434"), "http://localhost:11434");
        assert_eq!(normalize("http://box.local:11434/"), "http://box.local:11434");
    }

    #[test]
    fn progress_lines_carry_the_byte_counts() {
        let update = parse_line("moondream", r#"{"status":"downloading","completed":5,"total":10}"#)
            .unwrap();
        assert_eq!(update.completed, 5);
        assert_eq!(update.total, 10);
        assert!(!update.done);
    }

    #[test]
    fn the_success_line_ends_the_download() {
        let update = parse_line("moondream", r#"{"status":"success"}"#).unwrap();
        assert!(update.done);
    }

    #[test]
    fn an_error_line_becomes_a_failure() {
        let update = parse_line("nope", r#"{"error":"model not found"}"#).unwrap();
        assert_eq!(update.error.as_deref(), Some("model not found"));
        assert!(update.done);
    }

    #[test]
    fn blank_lines_between_updates_are_ignored() {
        assert!(parse_line("m", "").is_none());
    }

    #[test]
    fn vision_is_read_from_capabilities_when_the_daemon_reports_them() {
        assert!(detect_vision(&json!({ "capabilities": ["completion", "vision"] })));
        assert!(!detect_vision(&json!({ "capabilities": ["completion"] })));
    }

    #[test]
    fn vision_falls_back_to_the_projector_on_older_daemons() {
        assert!(detect_vision(&json!({ "projector_info": { "general.architecture": "clip" } })));
        assert!(detect_vision(&json!({ "details": { "families": ["qwen3vl"] } })));
        assert!(!detect_vision(&json!({ "details": { "families": ["llama"] } })));
    }

    #[test]
    fn the_catalog_marks_what_is_already_downloaded() {
        let installed = vec![InstalledModel {
            id: "moondream:latest".into(),
            size: 0,
            parameters: String::new(),
            quantization: String::new(),
            vision: true,
        }];
        let entries = catalog(&installed, 64_000_000_000);
        let moondream = entries.iter().find(|e| e.id == "moondream").unwrap();
        assert!(moondream.installed);
        assert!(entries.iter().filter(|e| e.installed).count() == 1);
    }

    #[test]
    fn models_larger_than_this_machine_are_flagged() {
        let entries = catalog(&[], 8_000_000_000);
        let big = entries.iter().find(|e| e.id == "gemma4:12b").unwrap();
        let small = entries.iter().find(|e| e.id == "moondream").unwrap();
        assert!(!big.fits);
        assert!(small.fits);
    }

    #[test]
    fn unknown_memory_never_blocks_a_download() {
        assert!(catalog(&[], 0).iter().all(|e| e.fits));
    }
}
