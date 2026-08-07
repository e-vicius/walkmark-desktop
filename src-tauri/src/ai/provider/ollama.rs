//! Ollama's native chat API, for models running on this machine.

use serde_json::{json, Value};

use super::{plain_schema, Ask};
use crate::error::{AppError, Result};

pub fn url(base: &str) -> String {
    format!("{base}/api/chat")
}

pub fn body(model: &str, ask: &Ask<'_>) -> Value {
    let mut messages = Vec::new();
    if !ask.system.is_empty() {
        messages.push(json!({ "role": "system", "content": ask.system }));
    }
    messages.push(json!({
        "role": "user",
        "content": ask.prompt,
        "images": ask.images.iter().map(|i| i.b64()).collect::<Vec<_>>(),
    }));

    let mut body = json!({
        "model": model,
        "messages": messages,
        // Streaming would let us show tokens arriving, but every step is written
        // in parallel and thrown at a JSON parser, so there is nothing to show.
        "stream": false,
        "options": {
            "temperature": ask.temperature,
            "num_predict": ask.max_tokens,
        },
        // Small local models love to editorialise around their JSON. Asking the
        // runtime to constrain the grammar is far more reliable than asking the
        // model nicely.
        "think": false,
    });
    if !ask.schema.is_null() {
        body["format"] = plain_schema(&ask.schema);
    }
    body
}

pub fn text(payload: &str) -> Result<String> {
    let root: Value = serde_json::from_str(payload)?;
    let text = root
        .pointer("/message/content")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if text.trim().is_empty() {
        return Err(AppError::ApiRejected(
            "The local model returned an empty response. Try a larger model.".into(),
        ));
    }
    Ok(text.to_string())
}

/// "Does this work" for a local model is really "is the daemon up and is the
/// model downloaded", both of which are instant to answer.
pub async fn check(http: &reqwest::Client, base: &str, model: &str) -> Result<()> {
    let response = http
        .get(format!("{base}/api/tags"))
        .send()
        .await
        .map_err(|_| {
            AppError::LocalRuntimeUnavailable(format!(
                "Could not reach Ollama at {base}. Make sure it is installed and running."
            ))
        })?;

    let payload = response.text().await.unwrap_or_default();
    let installed = installed_models(&payload);
    if installed.iter().any(|name| matches(name, model)) {
        return Ok(());
    }
    Err(AppError::LocalModelMissing(model.to_string()))
}

pub fn installed_models(payload: &str) -> Vec<String> {
    serde_json::from_str::<Value>(payload)
        .ok()
        .and_then(|root| root.get("models").and_then(Value::as_array).cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|m| {
            m.get("name")
                .or_else(|| m.get("model"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect()
}

/// Ollama reports `llava:latest` for what everybody types as `llava`.
pub fn matches(installed: &str, wanted: &str) -> bool {
    let normalize = |s: &str| {
        let s = s.trim();
        match s.split_once(':') {
            Some((name, "latest")) => name.to_string(),
            _ => s.to_string(),
        }
    };
    normalize(installed) == normalize(wanted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::provider::InlineImage;

    #[test]
    fn images_ride_alongside_the_message_rather_than_inside_it() {
        let ask = Ask {
            system: "be helpful",
            prompt: "what now",
            images: vec![InlineImage(vec![0, 1])],
            schema: crate::ai::prompt::step_schema(),
            schema_name: "step",
            temperature: 0.35,
            max_tokens: 512,
        };
        let body = body("qwen3-vl:8b", &ask);
        let user = &body["messages"][1];
        assert_eq!(user["content"], "what now");
        assert_eq!(user["images"].as_array().unwrap().len(), 1);
        assert!(body["format"]["properties"]["title"].is_object());
        assert_eq!(body["options"]["num_predict"], 512);
    }

    #[test]
    fn reads_the_message_content() {
        let payload = r#"{"message":{"role":"assistant","content":"{\"title\":\"Go\"}"}}"#;
        assert_eq!(text(payload).unwrap(), "{\"title\":\"Go\"}");
    }

    #[test]
    fn an_empty_answer_suggests_a_bigger_model() {
        let error = text(r#"{"message":{"content":""}}"#).unwrap_err().to_string();
        assert!(error.contains("larger model"), "{error}");
    }

    #[test]
    fn the_latest_tag_is_optional_when_matching_installed_models() {
        assert!(matches("llava:latest", "llava"));
        assert!(matches("llava", "llava:latest"));
        assert!(matches("qwen3-vl:8b", "qwen3-vl:8b"));
        assert!(!matches("qwen3-vl:4b", "qwen3-vl:8b"));
    }

    #[test]
    fn reads_installed_models_out_of_a_tags_response() {
        let payload = r#"{"models":[{"name":"qwen3-vl:8b"},{"name":"moondream:latest"}]}"#;
        assert_eq!(
            installed_models(payload),
            vec!["qwen3-vl:8b".to_string(), "moondream:latest".to_string()]
        );
    }
}
