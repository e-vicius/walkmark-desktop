//! Mistral's chat-completions API.
//!
//! Vision works reliably with a plain chat request. Structured output goes in
//! the system prompt as JSON instructions — Mistral's `response_format` and tool
//! modes reject several valid-looking OpenAI-shaped payloads when images are
//! attached.

use serde_json::{json, Value};

use super::Ask;
use crate::error::{AppError, Result};

pub fn body(model: &str, ask: &Ask<'_>) -> Value {
    let mut system = ask.system.to_string();
    if !ask.schema.is_null() {
        if !system.is_empty() {
            system.push('\n');
        }
        system.push_str(json_instruction(ask.schema_name));
    }

    let user_content = if ask.images.is_empty() {
        json!(ask.prompt)
    } else {
        let mut parts = vec![json!({ "type": "text", "text": ask.prompt })];
        parts.extend(ask.images.iter().map(|image| {
            json!({
                "type": "image_url",
                "image_url": format!("data:image/jpeg;base64,{}", image.b64()),
            })
        }));
        json!(parts)
    };

    let mut messages = Vec::new();
    if !ask.system.is_empty() {
        messages.push(json!({ "role": "system", "content": system }));
    }
    messages.push(json!({ "role": "user", "content": user_content }));

    json!({
        "model": model,
        "messages": messages,
        "max_tokens": ask.max_tokens,
        "temperature": ask.temperature,
    })
}

fn json_instruction(schema_name: &str) -> &'static str {
    match schema_name {
        "outline" => "Respond with one JSON object only, no markdown fences or commentary: \
{\"title\": string, \"summary\": string, \"prerequisites\": string[], \"steps\": string[]}.",
        _ => "Respond with one JSON object only, no markdown fences or commentary: \
{\"title\": string, \"body\": string}.",
    }
}

pub fn text(payload: &str) -> Result<String> {
    let root: Value = serde_json::from_str(payload)?;
    let message = root
        .pointer("/choices/0/message")
        .ok_or_else(|| AppError::ApiRejected("Mistral returned no content.".into()))?;

    let text = message
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if text.trim().is_empty() {
        let reason = root
            .pointer("/choices/0/finish_reason")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        return Err(AppError::ApiRejected(match reason {
            "length" => {
                "The response was cut short. Try a shorter step or a different model.".into()
            }
            "content_filter" => "The model refused to describe this screenshot. Try redacting \
sensitive regions."
                .into(),
            other => format!("The model returned an empty response ({other})."),
        }));
    }

    Ok(text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::provider::InlineImage;

    fn ask() -> Ask<'static> {
        Ask {
            system: "be helpful",
            prompt: "what now",
            images: vec![InlineImage(vec![0, 1])],
            schema: crate::ai::prompt::step_schema(),
            schema_name: "step",
            temperature: 0.35,
            max_tokens: 512,
        }
    }

    #[test]
    fn vision_requests_use_string_image_urls_and_no_structured_modes() {
        let body = body("mistral-small-latest", &ask());
        assert!(body["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("JSON object"));
        assert!(body["messages"][1]["content"][1]["image_url"].is_string());
        assert!(body.get("response_format").is_none());
        assert!(body.get("tools").is_none());
    }

    #[test]
    fn reads_json_from_the_assistant_message() {
        let payload = r#"{"choices":[{"message":{"content":"{\"title\":\"Go\",\"body\":\"Click.\"}"}}]}"#;
        assert_eq!(text(payload).unwrap(), "{\"title\":\"Go\",\"body\":\"Click.\"}");
    }
}
