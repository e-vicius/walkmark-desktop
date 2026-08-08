use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager};

use crate::error::{AppError, Result};
use crate::models::{Project, ProjectSummary, Provider, Settings, StepStatus};

const KEYRING_SERVICE: &str = "app.steppy";
/// Gemini-only builds stored one unlabelled key here.
const LEGACY_KEYRING_SERVICE: &str = "app.stepsy.gemini";
/// Stepsy renamed to Steppy — move provider keys across on first launch.
const PREVIOUS_KEYRING_SERVICE: &str = "app.stepsy";

/// ```text
/// <app data>/
///   settings.json
///   projects/<id>/project.json
///   projects/<id>/frames/*.png
/// ```
pub fn data_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Other(format!("Could not locate the app data directory: {e}")))?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn projects_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = data_dir(app)?.join("projects");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn project_dir(app: &AppHandle, id: &str) -> Result<PathBuf> {
    validate_id(id)?;
    let dir = projects_dir(app)?.join(id);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn frames_dir(app: &AppHandle, id: &str) -> Result<PathBuf> {
    let dir = project_dir(app, id)?.join("frames");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn frame_path(app: &AppHandle, project_id: &str, frame: &str) -> Result<PathBuf> {
    // Frame names come from the frontend, so refuse anything that could escape
    // the project directory.
    if frame.is_empty() || frame.contains(['/', '\\']) || frame.contains("..") {
        return Err(AppError::Invalid(format!("Invalid frame name `{frame}`.")));
    }
    Ok(frames_dir(app, project_id)?.join(frame))
}

/// Project ids are generated internally, but they arrive back over IPC so we
/// re-validate rather than trusting them as path components.
fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(AppError::Invalid(format!("Invalid project id `{id}`.")));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

fn settings_path(app: &AppHandle) -> Result<PathBuf> {
    Ok(data_dir(app)?.join("settings.json"))
}

pub fn load_settings(app: &AppHandle) -> Settings {
    migrate_legacy_key(app);
    migrate_previous_keyring(app);
    let mut settings: Settings = settings_path(app)
        .ok()
        .filter(|p| p.exists())
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .map(|mut raw| {
            migrate_settings(&mut raw);
            raw
        })
        // A settings file written by an older build should never block startup.
        .and_then(|raw| serde_json::from_value(raw).ok())
        .unwrap_or_default();

    settings.normalize_products();

    // Persist the migration now rather than waiting for the next save, so the
    // work happens once instead of on every launch.
    let _ = save_settings(app, &settings);
    settings
}

/// Bring a settings file written by an older build up to the current shape.
///
/// Two things change over time: the file used to hold a single top-level
/// `model` from when Gemini was the only option, and model names get retired by
/// the providers. Both are fixed here so nobody opens the app to a 404 from a
/// model that was turned off months ago.
fn migrate_settings(raw: &mut serde_json::Value) {
    use serde_json::{json, Value};

    let Some(root) = raw.as_object_mut() else {
        return;
    };

    // Pre-provider files: one `model`, always Gemini.
    if let Some(Value::String(model)) = root.remove("model") {
        if !root.contains_key("providers") {
            root.insert(
                "providers".into(),
                json!({ Provider::Gemini.id(): { "model": model } }),
            );
        }
    }
    root.entry("provider")
        .or_insert_with(|| json!(Provider::Gemini.id()));

    // A single global glossary becomes one product profile.
    if let Some(Value::String(glossary)) = root.remove("glossary") {
        if !root.contains_key("products") && !glossary.trim().is_empty() {
            let id = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
            root.insert(
                "products".into(),
                json!([{
                    "id": id,
                    "name": "General",
                    "vocabulary": glossary.trim().lines().filter(|l| !l.trim().is_empty()).map(|line| {
                        json!({ "id": uuid::Uuid::new_v4().simple().to_string()[..12].to_string(), "term": line.trim(), "explanation": "" })
                    }).collect::<Vec<_>>(),
                }]),
            );
            root.insert("defaultProductId".into(), json!(id));
        }
    }

    let Some(providers) = root.get_mut("providers").and_then(Value::as_object_mut) else {
        return;
    };
    for (id, config) in providers.iter_mut() {
        let Some(provider) = Provider::parse(id) else {
            continue;
        };
        let Some(model) = config.get("model").and_then(Value::as_str) else {
            continue;
        };
        if let Some(replacement) = crate::ai::catalog::migrate_model(provider, model) {
            config["model"] = json!(replacement);
        }
    }
}

pub fn save_settings(app: &AppHandle, settings: &Settings) -> Result<()> {
    let mut settings = settings.clone();
    settings.normalize_products();
    write_json_atomic(&settings_path(app)?, &settings)
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

fn project_file(app: &AppHandle, id: &str) -> Result<PathBuf> {
    Ok(project_dir(app, id)?.join("project.json"))
}

pub fn save_project(app: &AppHandle, project: &Project) -> Result<()> {
    write_json_atomic(&project_file(app, &project.id)?, project)
}

pub fn load_project(app: &AppHandle, id: &str) -> Result<Project> {
    let path = project_file(app, id)?;
    let raw = fs::read_to_string(&path)
        .map_err(|_| AppError::NotFound(format!("Project `{id}` could not be opened.")))?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn delete_project(app: &AppHandle, id: &str) -> Result<()> {
    let dir = project_dir(app, id)?;
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

pub fn list_projects(app: &AppHandle) -> Result<Vec<ProjectSummary>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(projects_dir(app)?)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        // Skip half-written or hand-edited directories instead of failing the
        // whole library listing.
        let Ok(project) = load_project(app, &id) else {
            continue;
        };
        let cover = project
            .steps
            .iter()
            .find(|s| s.include)
            .or_else(|| project.steps.first())
            .and_then(|s| frame_path(app, &id, &s.frame).ok())
            .filter(|p| p.exists())
            .map(|p| p.to_string_lossy().into_owned());

        out.push(ProjectSummary {
            id,
            title: project.title,
            updated_at: project.updated_at,
            step_count: project.steps.len(),
            ready_count: project
                .steps
                .iter()
                .filter(|s| s.status == StepStatus::Ready)
                .count(),
            cover,
        });
    }
    out.sort_by_key(|p| std::cmp::Reverse(p.updated_at));
    Ok(out)
}

/// Drop frames that no step references any more, so deleting steps actually
/// reclaims disk space.
pub fn prune_frames(app: &AppHandle, project: &Project) -> Result<()> {
    let dir = frames_dir(app, &project.id)?;
    let mut keep = std::collections::HashSet::new();
    for step in &project.steps {
        keep.insert(step.frame.clone());
        keep.extend(step.alternates.iter().cloned());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if !keep.contains(name) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// API keys
// ---------------------------------------------------------------------------
//
// Keys live in an owner-only file under the app data directory. macOS Keychain
// is consulted once to migrate older installs, then keys are copied to disk so
// dev rebuilds (which change the app signature) do not prompt every launch.
// An in-memory cache avoids repeated disk reads during a session.

static KEY_CACHE: OnceLock<Mutex<HashMap<Provider, String>>> = OnceLock::new();

fn key_cache() -> &'static Mutex<HashMap<Provider, String>> {
    KEY_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Load every stored key into memory. Call once during startup.
pub fn warm_api_key_cache(app: &AppHandle) {
    let mut cache = key_cache().lock().expect("api key cache");
    cache.clear();
    for provider in Provider::ALL {
        if !provider.needs_key() {
            continue;
        }
        if let Some(key) = read_key(app, provider) {
            cache.insert(provider, key);
        }
    }
}

/// Prefer the OS keychain. Some Linux desktops ship without a Secret Service
/// provider, so fall back to an owner-only file rather than losing the feature.
pub fn set_api_key(app: &AppHandle, provider: Provider, key: &str) -> Result<()> {
    let key = crate::limits::clamp_trim(key, crate::limits::API_KEY);
    if key.is_empty() {
        return clear_api_key(app, provider);
    }
    write_fallback_key(app, provider, &key)?;
    key_cache()
        .lock()
        .expect("api key cache")
        .insert(provider, key.clone());
    // Best-effort keychain mirror for installs that already rely on it.
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, provider.id()) {
        let _ = entry.set_password(&key);
    }
    Ok(())
}

pub fn get_api_key(app: &AppHandle, provider: Provider) -> Option<String> {
    if let Some(key) = key_cache()
        .lock()
        .expect("api key cache")
        .get(&provider)
        .cloned()
    {
        return Some(key);
    }
    let key = read_key(app, provider)?;
    key_cache()
        .lock()
        .expect("api key cache")
        .insert(provider, key.clone());
    Some(key)
}

pub fn clear_api_key(app: &AppHandle, provider: Provider) -> Result<()> {
    key_cache()
        .lock()
        .expect("api key cache")
        .remove(&provider);
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, provider.id()) {
        let _ = entry.delete_credential();
    }
    let _ = fs::remove_file(fallback_key_path(app, provider)?);
    Ok(())
}

/// Which providers currently have a usable credential, for the settings UI.
pub fn providers_with_keys(app: &AppHandle) -> Vec<Provider> {
    Provider::ALL
        .into_iter()
        .filter(|p| p.needs_key() && get_api_key(app, *p).is_some())
        .collect()
}

fn read_key(app: &AppHandle, provider: Provider) -> Option<String> {
    if let Ok(path) = fallback_key_path(app, provider) {
        if let Ok(secret) = fs::read_to_string(&path) {
            let secret = secret.trim().to_string();
            if !secret.is_empty() {
                return Some(secret);
            }
        }
    }

    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, provider.id()) {
        if let Ok(secret) = entry.get_password() {
            let secret = secret.trim().to_string();
            if !secret.is_empty() {
                // Copy into the file store so the next launch skips Keychain.
                let _ = write_fallback_key(app, provider, &secret);
                return Some(secret);
            }
        }
    }

    None
}

fn fallback_key_path(app: &AppHandle, provider: Provider) -> Result<PathBuf> {
    Ok(data_dir(app)?.join(format!("credentials.{}", provider.id())))
}

fn write_fallback_key(app: &AppHandle, provider: Provider, key: &str) -> Result<()> {
    let path = fallback_key_path(app, provider)?;
    fs::write(&path, key)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Earlier builds only knew about Gemini and stored one unlabelled key. Move it
/// under the Gemini name the first time we see it, so upgrading doesn't look
/// like being signed out.
fn migrate_legacy_key(app: &AppHandle) {
    let Ok(legacy_file) = data_dir(app).map(|d| d.join("credentials")) else {
        return;
    };
    let legacy_entry = keyring::Entry::new(LEGACY_KEYRING_SERVICE, "default").ok();
    let existing = legacy_entry
        .as_ref()
        .and_then(|e| e.get_password().ok())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            fs::read_to_string(&legacy_file)
                .ok()
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
        });

    let Some(key) = existing else { return };
    if get_api_key(app, Provider::Gemini).is_none() {
        let _ = set_api_key(app, Provider::Gemini, &key);
    }
    if let Some(entry) = legacy_entry {
        let _ = entry.delete_credential();
    }
    let _ = fs::remove_file(legacy_file);
}

/// Move API keys from the old Stepsy keychain service to Steppy.
fn migrate_previous_keyring(app: &AppHandle) {
    for provider in Provider::ALL {
        if !provider.needs_key() {
            continue;
        }
        if get_api_key(app, provider).is_some() {
            continue;
        }
        let Ok(entry) = keyring::Entry::new(PREVIOUS_KEYRING_SERVICE, provider.id()) else {
            continue;
        };
        let Ok(key) = entry.get_password() else {
            continue;
        };
        if key.trim().is_empty() {
            continue;
        }
        let _ = set_api_key(app, provider, key.trim());
        let _ = entry.delete_credential();
    }
}

/// Write to a sibling temp file then rename, so a crash mid-save can never
/// leave a truncated project.json behind.
fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn migrated(mut raw: serde_json::Value) -> Settings {
        migrate_settings(&mut raw);
        serde_json::from_value(raw).expect("migrated settings should still deserialize")
    }

    #[test]
    fn a_gemini_only_settings_file_keeps_its_model() {
        let settings = migrated(json!({
            "model": "gemini-2.5-flash",
            "audience": "a colleague",
            "tone": "neutral",
            "language": "English",
            "concurrency": 3,
            "capture": {
                "sampleIntervalMs": 600, "sensitivity": 0.55, "minGapMs": 1200,
                "settle": true, "maxWidth": 1800, "countdownSecs": 3, "hideWindow": true
            },
            "theme": "system",
            "onboarded": true
        }));

        assert_eq!(settings.provider, Provider::Gemini);
        assert_eq!(settings.config_for(settings.provider).model, "gemini-3.5-flash-lite");
        assert!(settings.onboarded);
        assert_eq!(settings.concurrency, 3);
    }

    #[test]
    fn a_retired_model_is_replaced_with_one_that_still_exists() {
        let settings = migrated(json!({
            "model": "gemini-2.0-flash",
            "audience": "a colleague",
            "tone": "neutral",
            "language": "English",
            "concurrency": 3,
            "capture": {
                "sampleIntervalMs": 600, "sensitivity": 0.55, "minGapMs": 1200,
                "settle": true, "maxWidth": 1800, "countdownSecs": 3, "hideWindow": true
            },
            "theme": "system",
            "onboarded": true
        }));

        assert_eq!(settings.config_for(settings.provider).model, "gemini-3.5-flash-lite");
    }

    #[test]
    fn a_current_settings_file_is_left_alone() {
        let settings = migrated(json!({
            "provider": "ollama",
            "providers": { "ollama": { "model": "qwen3-vl:8b" } },
            "audience": "a colleague",
            "tone": "neutral",
            "language": "English",
            "concurrency": 3,
            "capture": {
                "sampleIntervalMs": 600, "sensitivity": 0.55, "minGapMs": 1200,
                "settle": true, "maxWidth": 1800, "countdownSecs": 3, "hideWindow": true
            },
            "theme": "system",
            "onboarded": true
        }));

        assert_eq!(settings.provider, Provider::Ollama);
        assert_eq!(settings.config_for(settings.provider).model, "qwen3-vl:8b");
    }

    #[test]
    fn an_unconfigured_provider_falls_back_to_the_catalog_default() {
        let settings = Settings::default();
        assert_eq!(settings.config_for(settings.provider).model, "gemini-3.6-flash");
        assert_eq!(
            settings.config_for(Provider::Anthropic).model,
            "claude-sonnet-5"
        );
        assert_eq!(
            settings.config_for(Provider::Ollama).base_url,
            "http://127.0.0.1:11434"
        );
    }

    #[test]
    fn a_custom_endpoint_overrides_the_default() {
        let settings = Settings {
            provider: Provider::Compatible,
            providers: [(
                Provider::Compatible,
                crate::models::ProviderConfig {
                    model: "local/llava".into(),
                    base_url: "http://localhost:1234/v1/".into(),
                },
            )]
            .into(),
            ..Default::default()
        };
        let active = settings.config_for(settings.provider);
        assert_eq!(active.model, "local/llava");
        // The trailing slash would otherwise produce a double slash in URLs.
        assert_eq!(active.base_url, "http://localhost:1234/v1");
    }
}
