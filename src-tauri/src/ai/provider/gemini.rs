//! Google's `generateContent` API.

use serde_json::{json, Value};

use super::Ask;
use crate::error::{AppError, Result};

pub fn url(base: &str, model: &str) -> String {
    format!("{base}/models/{model}:generateContent")
}

pub fn models_url(base: &str) -> String {
    format!("{}/models", base.trim_end_matches('/'))
}

pub fn body(model: &str, ask: &Ask<'_>) -> Value {
    let mut parts = vec![json!({ "text": ask.prompt })];
    parts.extend(ask.images.iter().map(|image| {
        json!({ "inlineData": { "mimeType": "image/jpeg", "data": image.b64() } })
    }));

    let mut config = json!({
        "temperature": ask.temperature,
        "maxOutputTokens": ask.max_tokens,
    });
    if !ask.schema.is_null() {
        config["responseMimeType"] = json!("application/json");
        config["responseSchema"] = ask.schema.clone();
    }
    // Turning a screenshot into one sentence does not need a chain of thought,
    // and paying for one doubles both the latency and the bill.
    config["thinkingConfig"] = thinking_config(model);

    let mut body = json!({
        "contents": [{ "role": "user", "parts": parts }],
        "generationConfig": config,
    });
    if !ask.system.is_empty() {
        body["systemInstruction"] = json!({ "parts": [{ "text": ask.system }] });
    }
    body
}

/// Gemini 3 replaced the numeric `thinkingBudget` with named levels, and sending
/// both is a hard 400. Older models only understand the budget.
fn thinking_config(model: &str) -> Value {
    if model.starts_with("gemini-3") || model.starts_with("gemini-4") {
        // `minimal` isn't accepted by the Pro tier, and `low` is everywhere.
        json!({ "thinkingLevel": "low" })
    } else {
        json!({ "thinkingBudget": 0 })
    }
}

/// Pull the model's text out of the candidate envelope, translating the
/// "no candidate" cases into something a person can act on.
pub fn text(payload: &str) -> Result<String> {
    let root: Value = serde_json::from_str(payload)?;

    if let Some(reason) = root
        .pointer("/promptFeedback/blockReason")
        .and_then(Value::as_str)
    {
        return Err(AppError::ApiRejected(format!(
            "Gemini blocked this screenshot ({reason}). Try excluding the step or redacting \
sensitive regions."
        )));
    }

    let candidate = root
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .ok_or_else(|| AppError::ApiRejected("Gemini returned no content.".into()))?;

    let text: String = candidate
        .pointer("/content/parts")
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(|p| p.get("text").and_then(Value::as_str))
                .collect()
        })
        .unwrap_or_default();

    if text.trim().is_empty() {
        let finish = candidate
            .get("finishReason")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        return Err(AppError::ApiRejected(match finish {
            "SAFETY" => "Gemini blocked this screenshot for safety reasons.".into(),
            "MAX_TOKENS" => {
                "The response was cut short. Try a shorter step or a different model.".into()
            }
            other => format!("Gemini returned an empty response ({other})."),
        }));
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_text_from_a_normal_response() {
        let payload = r#"{"candidates":[{"content":{"parts":[{"text":"{\"title\":\"Hi\"}"}]}}]}"#;
        assert_eq!(text(payload).unwrap(), "{\"title\":\"Hi\"}");
    }

    #[test]
    fn surfaces_blocked_prompts() {
        assert!(text(r#"{"promptFeedback":{"blockReason":"SAFETY"}}"#).is_err());
    }

    #[test]
    fn surfaces_truncated_responses() {
        let payload = r#"{"candidates":[{"finishReason":"MAX_TOKENS","content":{"parts":[]}}]}"#;
        let error = text(payload).unwrap_err().to_string();
        assert!(error.contains("cut short"), "{error}");
    }

    #[test]
    fn three_series_models_get_a_thinking_level_and_never_a_budget() {
        for model in ["gemini-3.6-flash", "gemini-3.1-pro-preview"] {
            let config = thinking_config(model);
            assert_eq!(config["thinkingLevel"], "low", "{model}");
            assert!(config.get("thinkingBudget").is_none(), "{model}");
        }
    }

    #[test]
    fn older_models_still_get_the_budget_they_understand() {
        let config = thinking_config("gemini-2.5-flash");
        assert_eq!(config["thinkingBudget"], 0);
        assert!(config.get("thinkingLevel").is_none());
    }

    #[test]
    fn the_url_is_built_from_the_configured_base() {
        assert_eq!(
            url("https://example.test/v1beta", "gemini-3.6-flash"),
            "https://example.test/v1beta/models/gemini-3.6-flash:generateContent"
        );
    }

    #[test]
    fn the_models_url_lists_available_models() {
        assert_eq!(
            models_url("https://generativelanguage.googleapis.com/v1beta"),
            "https://generativelanguage.googleapis.com/v1beta/models"
        );
    }
}
