//! Anthropic's Messages API.
//!
//! Structured output goes through a forced tool call rather than a response
//! format: define one tool whose input schema is the shape we want, then insist
//! the model calls it. The reply arrives already parsed into an object.

use serde_json::{json, Value};

use super::{plain_schema, Ask};
use crate::error::{AppError, Result};

pub const VERSION: &str = "2023-06-01";

pub fn url(base: &str) -> String {
    if base.ends_with("/messages") {
        base.to_string()
    } else {
        format!("{base}/messages")
    }
}

pub fn body(model: &str, ask: &Ask<'_>) -> Value {
    // Anthropic reads images better when they come before the question about
    // them, which is the opposite of what the other providers prefer.
    let mut content: Vec<Value> = ask
        .images
        .iter()
        .map(|image| {
            json!({
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": "image/jpeg",
                    "data": image.b64()
                }
            })
        })
        .collect();
    content.push(json!({ "type": "text", "text": ask.prompt }));

    let mut body = json!({
        "model": model,
        // Claude 5 thinks adaptively by default and rejects a temperature while
        // it does, so we simply don't send one.
        "max_tokens": ask.max_tokens,
        "messages": [{ "role": "user", "content": content }],
    });
    if !ask.system.is_empty() {
        body["system"] = json!(ask.system);
    }

    if !ask.schema.is_null() {
        body["tools"] = json!([{
            "name": ask.schema_name,
            "description": "Record the finished documentation text.",
            "input_schema": plain_schema(&ask.schema),
        }]);
        body["tool_choice"] = json!({ "type": "tool", "name": ask.schema_name });
    }

    body
}

pub fn text(payload: &str) -> Result<String> {
    let root: Value = serde_json::from_str(payload)?;
    let blocks = root
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::ApiRejected("Claude returned no content.".into()))?;

    // A forced tool call is the normal path; plain text is what a schema-less
    // request (the credential check) gets back.
    if let Some(input) = blocks
        .iter()
        .find(|b| b.get("type").and_then(Value::as_str) == Some("tool_use"))
        .and_then(|b| b.get("input"))
    {
        return Ok(input.to_string());
    }

    let text: String = blocks
        .iter()
        .filter_map(|b| b.get("text").and_then(Value::as_str))
        .collect();

    if text.trim().is_empty() {
        let reason = root
            .get("stop_reason")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        return Err(AppError::ApiRejected(match reason {
            "max_tokens" => {
                "The response was cut short. Try a shorter step or a different model.".into()
            }
            "refusal" => "Claude declined to describe this screenshot. Try redacting sensitive \
regions."
                .into(),
            other => format!("Claude returned an empty response ({other})."),
        }));
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::provider::InlineImage;

    fn ask(schema: Value) -> Ask<'static> {
        Ask {
            system: "be helpful",
            prompt: "what now",
            images: vec![InlineImage(vec![0, 1])],
            schema,
            schema_name: "step",
            temperature: 0.35,
            max_tokens: 512,
        }
    }

    #[test]
    fn a_schema_becomes_a_forced_tool_call() {
        let body = body("claude-sonnet-5", &ask(crate::ai::prompt::step_schema()));
        assert_eq!(body["tools"][0]["name"], "step");
        assert_eq!(body["tool_choice"]["type"], "tool");
        assert!(body["tools"][0]["input_schema"]["properties"]["title"].is_object());
    }

    #[test]
    fn a_schemaless_request_sends_no_tools() {
        let body = body("claude-sonnet-5", &ask(Value::Null));
        assert!(body.get("tools").is_none());
        assert!(body.get("tool_choice").is_none());
    }

    #[test]
    fn never_sends_a_temperature_alongside_adaptive_thinking() {
        let body = body("claude-opus-5", &ask(crate::ai::prompt::step_schema()));
        assert!(body.get("temperature").is_none());
    }

    #[test]
    fn images_come_before_the_question_about_them() {
        let body = body("claude-sonnet-5", &ask(Value::Null));
        let content = body["messages"][0]["content"].as_array().unwrap().clone();
        assert_eq!(content[0]["type"], "image");
        assert_eq!(content[1]["type"], "text");
    }

    #[test]
    fn reads_the_tool_input_as_the_answer() {
        let payload = r#"{"content":[{"type":"tool_use","name":"step","input":{"title":"Go","body":"Click."}}]}"#;
        let parsed: Value = serde_json::from_str(&text(payload).unwrap()).unwrap();
        assert_eq!(parsed["title"], "Go");
    }

    #[test]
    fn falls_back_to_plain_text_blocks() {
        let payload = r#"{"content":[{"type":"text","text":"ok"}]}"#;
        assert_eq!(text(payload).unwrap(), "ok");
    }

    #[test]
    fn surfaces_a_truncated_response() {
        let payload = r#"{"stop_reason":"max_tokens","content":[]}"#;
        let error = text(payload).unwrap_err().to_string();
        assert!(error.contains("cut short"), "{error}");
    }

    #[test]
    fn the_url_tolerates_a_base_that_already_points_at_messages() {
        assert_eq!(
            url("https://api.anthropic.com/v1"),
            "https://api.anthropic.com/v1/messages"
        );
        assert_eq!(
            url("https://proxy.test/v1/messages"),
            "https://proxy.test/v1/messages"
        );
    }
}
