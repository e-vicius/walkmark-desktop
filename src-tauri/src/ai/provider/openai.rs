//! OpenAI's chat-completions API, and everything that imitates it.
//!
//! The two flavours differ in the small ways that break third-party servers:
//! OpenAI's reasoning models want `max_completion_tokens` and reject
//! `temperature`, while LM Studio, llama.cpp and vLLM want the classic fields
//! and have never heard of `reasoning_effort`.

use serde_json::{json, Value};

use super::{strict_schema, plain_schema, Ask};
use crate::error::{AppError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    OpenAi,
    Compatible,
}

pub fn url(base: &str) -> String {
    // People paste both "…/v1" and the bare host; accept either.
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    }
}

pub fn body(model: &str, ask: &Ask<'_>, flavor: Flavor) -> Value {
    let mut content = vec![json!({ "type": "text", "text": ask.prompt })];
    content.extend(ask.images.iter().map(|image| {
        json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:image/jpeg;base64,{}", image.b64()),
                // Interface labels are small; the cheap tier resizes them away.
                "detail": "high"
            }
        })
    }));

    let mut messages = Vec::new();
    if !ask.system.is_empty() {
        messages.push(json!({ "role": "system", "content": ask.system }));
    }
    messages.push(json!({ "role": "user", "content": content }));

    let mut body = json!({ "model": model, "messages": messages });

    match flavor {
        Flavor::OpenAi => {
            body["max_completion_tokens"] = json!(ask.max_tokens);
            // The mechanical "look at this screen, say what to click" task gets
            // no better with more reasoning, only slower.
            body["reasoning_effort"] = json!("low");
        }
        Flavor::Compatible => {
            body["max_tokens"] = json!(ask.max_tokens);
            body["temperature"] = json!(ask.temperature);
        }
    }

    if !ask.schema.is_null() {
        body["response_format"] = json!({
            "type": "json_schema",
            "json_schema": {
                "name": ask.schema_name,
                // Strict mode is exact but demands a schema shape that many
                // OpenAI-compatible servers reject outright.
                "strict": flavor == Flavor::OpenAi,
                "schema": match flavor {
                    Flavor::OpenAi => strict_schema(&ask.schema),
                    Flavor::Compatible => plain_schema(&ask.schema),
                },
            }
        });
    }

    body
}

pub fn text(payload: &str) -> Result<String> {
    let root: Value = serde_json::from_str(payload)?;
    let choice = root
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .ok_or_else(|| AppError::ApiRejected("The server returned no content.".into()))?;

    let text = choice
        .pointer("/message/content")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if text.trim().is_empty() {
        let reason = choice
            .get("finish_reason")
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
    fn accepts_a_base_url_with_or_without_the_version_segment() {
        assert_eq!(
            url("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            url("http://localhost:1234"),
            "http://localhost:1234/v1/chat/completions"
        );
        assert_eq!(
            url("https://gateway.test/openai/chat/completions"),
            "https://gateway.test/openai/chat/completions"
        );
    }

    #[test]
    fn openai_uses_reasoning_fields_and_compatible_uses_classic_ones() {
        let openai = body("gpt-5.6-terra", &ask(), Flavor::OpenAi);
        assert_eq!(openai["max_completion_tokens"], 512);
        assert_eq!(openai["reasoning_effort"], "low");
        assert!(openai.get("temperature").is_none());

        let compatible = body("local-model", &ask(), Flavor::Compatible);
        assert_eq!(compatible["max_tokens"], 512);
        assert!(compatible.get("temperature").is_some());
        assert!(compatible.get("reasoning_effort").is_none());
    }

    #[test]
    fn only_openai_gets_the_strict_schema_third_party_servers_reject() {
        let openai = body("gpt-5.6-terra", &ask(), Flavor::OpenAi);
        assert_eq!(openai["response_format"]["json_schema"]["strict"], true);
        assert_eq!(
            openai["response_format"]["json_schema"]["schema"]["additionalProperties"],
            false
        );

        let compatible = body("local-model", &ask(), Flavor::Compatible);
        assert_eq!(compatible["response_format"]["json_schema"]["strict"], false);
    }

    #[test]
    fn the_system_prompt_becomes_its_own_message() {
        let body = body("m", &ask(), Flavor::OpenAi);
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["role"], "user");
    }

    #[test]
    fn reads_the_assistant_message() {
        let payload = r#"{"choices":[{"message":{"content":"{\"title\":\"Go\"}"}}]}"#;
        assert_eq!(text(payload).unwrap(), "{\"title\":\"Go\"}");
    }

    #[test]
    fn surfaces_a_truncated_response() {
        let payload = r#"{"choices":[{"finish_reason":"length","message":{"content":""}}]}"#;
        let error = text(payload).unwrap_err().to_string();
        assert!(error.contains("cut short"), "{error}");
    }
}
