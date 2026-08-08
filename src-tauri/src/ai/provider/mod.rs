//! One client, four wire protocols.
//!
//! Everything the rest of the app needs — an outline, a step description, a
//! cheap credential check — is expressed once here. The per-provider modules
//! only know how to turn an [`Ask`] into a request body and a response body
//! back into text, which keeps the retry, backoff and error-mapping rules
//! identical no matter who is answering.

mod anthropic;
mod gemini;
mod mistral;
mod ollama;
mod openai;

use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use crate::error::{AppError, Result};
use crate::models::Provider;

/// Ollama spells `llava` as `llava:latest`; the local pane needs the same
/// comparison when deciding whether a catalog entry is already downloaded.
pub use ollama::matches as ollama_name_matches;

const MAX_ATTEMPTS: u32 = 4;
/// Cloud providers answer in seconds. A local model on a laptop can genuinely
/// take minutes for the outline pass, and timing it out would be worse than
/// waiting.
const CLOUD_TIMEOUT: Duration = Duration::from_secs(120);
const LOCAL_TIMEOUT: Duration = Duration::from_secs(900);

#[derive(Debug, Clone, Deserialize)]
pub struct Outline {
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StepText {
    pub title: String,
    pub body: String,
}

/// A JPEG ready to be inlined into a request.
pub struct InlineImage(pub Vec<u8>);

impl InlineImage {
    fn b64(&self) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&self.0)
    }
}

/// A provider-neutral request: some instructions, some pictures, and the shape
/// of the answer we expect back.
pub struct Ask<'a> {
    pub system: &'a str,
    pub prompt: &'a str,
    pub images: Vec<InlineImage>,
    /// Plain JSON Schema. Each provider translates it into its own flavour of
    /// structured output.
    pub schema: Value,
    /// Identifier some providers require for the schema.
    pub schema_name: &'static str,
    pub temperature: f32,
    pub max_tokens: u32,
}

pub struct Client {
    http: reqwest::Client,
    provider: Provider,
    model: String,
    base_url: String,
    api_key: String,
}

impl Client {
    pub fn new(
        provider: Provider,
        model: String,
        base_url: String,
        api_key: String,
    ) -> Result<Self> {
        let model = crate::limits::clamp_trim(&model, crate::limits::MODEL_ID);
        if model.is_empty() {
            return Err(AppError::Invalid(
                "No model is selected. Choose one in Settings.".into(),
            ));
        }

        let base_url = {
            let raw = crate::limits::clamp_trim(&base_url, crate::limits::BASE_URL)
                .trim_end_matches('/')
                .to_string();
            if raw.is_empty() {
                provider.default_base_url().trim_end_matches('/').to_string()
            } else {
                raw.to_string()
            }
        };
        if base_url.is_empty() {
            return Err(AppError::Invalid(
                "This provider needs a server address. Add one in Settings.".into(),
            ));
        }

        let mut builder = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(if provider.is_local() {
                LOCAL_TIMEOUT
            } else {
                CLOUD_TIMEOUT
            });
        // A system proxy configured for the open internet will happily swallow
        // requests to a daemon on this machine.
        if is_loopback(&base_url) {
            builder = builder.no_proxy();
        }

        Ok(Self {
            http: builder.build()?,
            provider,
            model,
            base_url,
            api_key: crate::limits::clamp_trim(&api_key, crate::limits::API_KEY),
        })
    }

    /// The cheapest possible round trip, so Settings can tell somebody their
    /// setup works before they sit through a whole generation run.
    ///
    /// Listing models only checks the key — it deliberately does not depend on
    /// whichever model happens to be selected in Settings.
    pub async fn check(&self) -> Result<()> {
        if self.provider.needs_key() && self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }
        // For a local model, "does it work" mostly means "is it downloaded", and
        // asking the daemon is instant where a generation is not.
        if self.provider.is_local() {
            return ollama::check(&self.http, &self.base_url, &self.model).await;
        }

        match self.provider {
            Provider::Gemini => self.check_get(&gemini::models_url(&self.base_url)).await,
            Provider::OpenAi | Provider::Mistral | Provider::OpenRouter => {
                self.check_get(&openai::models_url(&self.base_url)).await
            }
            Provider::Compatible => match self.check_get(&openai::models_url(&self.base_url)).await {
                Ok(()) => Ok(()),
                Err(_) => self.check_chat().await,
            },
            Provider::Anthropic => self.check_chat().await,
            Provider::Ollama => unreachable!(),
        }
    }

    async fn check_get(&self, url: &str) -> Result<()> {
        let request = self.authorize(self.http.get(url));
        let response = request.send().await.map_err(AppError::Network)?;
        let status = response.status();
        let payload = response.text().await.unwrap_or_default();

        if status.is_success() {
            return Ok(());
        }

        let message = self
            .error_message(&payload)
            .unwrap_or_else(|| format!("{} returned HTTP {}.", self.provider.label(), status));

        match status.as_u16() {
            401 | 403 => Err(AppError::ApiRejected(format!(
                "{message} Check that your {} API key is valid.",
                self.provider.label()
            ))),
            _ => Err(AppError::ApiRejected(message)),
        }
    }

    async fn check_chat(&self) -> Result<()> {
        self.send(&Ask {
            system: "",
            prompt: "Reply with the single word: ok",
            images: Vec::new(),
            schema: Value::Null,
            schema_name: "check",
            temperature: 0.0,
            max_tokens: 16,
        })
        .await
        .map(|_| ())
    }

    pub async fn outline(
        &self,
        system: &str,
        prompt: &str,
        images: Vec<InlineImage>,
    ) -> Result<Outline> {
        self.ask(&Ask {
            system,
            prompt,
            images,
            schema: super::prompt::outline_schema(),
            schema_name: "outline",
            temperature: 0.3,
            max_tokens: 4096,
        })
        .await
    }

    pub async fn describe(
        &self,
        system: &str,
        prompt: &str,
        image: InlineImage,
    ) -> Result<StepText> {
        self.ask(&Ask {
            system,
            prompt,
            images: vec![image],
            schema: super::prompt::step_schema(),
            schema_name: "step",
            temperature: 0.35,
            max_tokens: 1024,
        })
        .await
    }

    async fn ask<T: for<'de> Deserialize<'de>>(&self, ask: &Ask<'_>) -> Result<T> {
        let text = self.send(ask).await?;
        parse_json(&text).ok_or_else(|| {
            AppError::ApiRejected(format!(
                "{} returned something that wasn't the expected JSON. Try running it again{}.",
                self.provider.label(),
                if self.provider.is_local() {
                    ", or pick a larger local model"
                } else {
                    ""
                }
            ))
        })
    }

    /// Sends the request, retrying only the failures worth retrying: rate
    /// limits, transient 5xx, and dropped connections.
    async fn send(&self, ask: &Ask<'_>) -> Result<String> {
        if self.provider == Provider::Mistral
            && !ask.images.is_empty()
            && !crate::ai::catalog::model_has_vision(Provider::Mistral, &self.model)
        {
            return Err(AppError::Invalid(format!(
                "The Mistral model `{}` cannot read screenshots. Pick Mistral Small or Large in \
the write menu.",
                self.model
            )));
        }

        let mut last: Option<AppError> = None;

        for attempt in 0..MAX_ATTEMPTS {
            if attempt > 0 {
                // Exponential backoff with jitter, so a burst of parallel step
                // requests doesn't retry in lockstep.
                let base = 700u64 << (attempt - 1);
                let jitter = (uuid::Uuid::new_v4().as_u128() % 400) as u64;
                tokio::time::sleep(Duration::from_millis(base + jitter)).await;
            }

            // `.json()` already sets `Content-Type`. Setting it again breaks some
            // providers (notably Mistral behind the system proxy) with opaque 422s.
            let request = self.http.post(self.url()).json(&self.body(ask));
            let request = self.authorize(request);

            let response = match request.send().await {
                Ok(r) => r,
                Err(e) if e.is_timeout() || e.is_connect() => {
                    if self.provider.is_local() && e.is_connect() {
                        return Err(self.unreachable());
                    }
                    last = Some(AppError::Network(e));
                    continue;
                }
                Err(e) => return Err(AppError::Network(e)),
            };

            let status = response.status();
            let payload = response.text().await.unwrap_or_default();

            if status.is_success() {
                return self.extract(&payload);
            }

            let message = self
                .error_message(&payload)
                .unwrap_or_else(|| format!("{} returned HTTP {}.", self.provider.label(), status));

            match status.as_u16() {
                429 | 500 | 502 | 503 | 504 => last = Some(AppError::ApiRejected(message)),
                401 | 403 => {
                    return Err(AppError::ApiRejected(format!(
                        "{message} Check that your {} key is valid and can use `{}`.",
                        self.provider.label(),
                        self.model
                    )))
                }
                404 if self.provider.is_local() => {
                    return Err(AppError::LocalModelMissing(self.model.clone()))
                }
                404 => {
                    return Err(AppError::ApiRejected(format!(
                        "The model `{}` was not found. It may have been retired, or your key may \
not have access to it.",
                        self.model
                    )))
                }
                _ => return Err(AppError::ApiRejected(message)),
            }
        }

        Err(last.unwrap_or_else(|| {
            AppError::ApiRejected(format!(
                "{} did not respond after several attempts.",
                self.provider.label()
            ))
        }))
    }

    fn url(&self) -> String {
        match self.provider {
            Provider::Gemini => gemini::url(&self.base_url, &self.model),
            Provider::Anthropic => anthropic::url(&self.base_url),
            Provider::Ollama => ollama::url(&self.base_url),
            Provider::OpenAi | Provider::OpenRouter | Provider::Mistral | Provider::Compatible => openai::url(&self.base_url),
        }
    }

    fn authorize(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.api_key.is_empty() {
            return request;
        }
        match self.provider {
            Provider::Gemini => request.header("x-goog-api-key", &self.api_key),
            Provider::Anthropic => request
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", anthropic::VERSION),
            _ => request.bearer_auth(&self.api_key),
        }
    }

    fn body(&self, ask: &Ask<'_>) -> Value {
        match self.provider {
            Provider::Gemini => gemini::body(&self.model, ask),
            Provider::Anthropic => anthropic::body(&self.model, ask),
            Provider::Ollama => ollama::body(&self.model, ask),
            Provider::OpenAi => openai::body(&self.model, ask, openai::Flavor::OpenAi),
            Provider::Mistral => mistral::body(&self.model, ask),
            Provider::OpenRouter | Provider::Compatible => {
                openai::body(&self.model, ask, openai::Flavor::Compatible)
            }
        }
    }

    fn extract(&self, payload: &str) -> Result<String> {
        match self.provider {
            Provider::Gemini => gemini::text(payload),
            Provider::Anthropic => anthropic::text(payload),
            Provider::Ollama => ollama::text(payload),
            Provider::OpenAi | Provider::OpenRouter | Provider::Compatible => openai::text(payload),
            Provider::Mistral => mistral::text(payload),
        }
    }

    fn error_message(&self, payload: &str) -> Option<String> {
        let root: Value = serde_json::from_str(payload).ok()?;

        if let Some(text) = root.pointer("/error/message").and_then(Value::as_str) {
            return Some(text.to_owned());
        }

        if let Some(message) = root.get("message") {
            if let Some(text) = message.as_str() {
                return Some(text.to_owned());
            }
            if let Some(formatted) = format_validation_errors(message.get("detail").unwrap_or(message))
            {
                return Some(formatted);
            }
        }

        if let Some(formatted) = format_validation_errors(root.get("detail")?) {
            return Some(formatted);
        }

        root.get("error").and_then(Value::as_str).map(str::to_owned)
    }

    fn unreachable(&self) -> AppError {
        AppError::LocalRuntimeUnavailable(format!(
            "Could not reach Ollama at {}. Make sure it is installed and running.",
            self.base_url
        ))
    }
}

fn format_validation_errors(detail: &Value) -> Option<String> {
    let items = detail.as_array()?;
    let messages: Vec<String> = items
        .iter()
        .filter_map(|item| {
            let msg = item.get("msg")?.as_str()?;
            let loc = item
                .get("loc")
                .and_then(Value::as_array)
                .map(|parts| {
                    parts
                        .iter()
                        .filter_map(Value::as_str)
                        .filter(|part| *part != "body")
                        .collect::<Vec<_>>()
                        .join(".")
                })
                .filter(|path| !path.is_empty());
            Some(match loc {
                Some(path) => format!("{msg} ({path})"),
                None => msg.to_owned(),
            })
        })
        .collect();
    if messages.is_empty() {
        None
    } else {
        Some(messages.join("; "))
    }
}

fn is_loopback(base_url: &str) -> bool {
    let host = base_url
        .split("://")
        .nth(1)
        .unwrap_or(base_url)
        .split('/')
        .next()
        .unwrap_or_default();
    let host = host.rsplit_once(':').map(|(h, _)| h).unwrap_or(host);
    matches!(host, "localhost" | "127.0.0.1" | "0.0.0.0" | "[::1]" | "::1")
}

/// Models sometimes wrap JSON in a ```json fence or add a sentence around it,
/// especially the small local ones. Rather than failing the step, dig the
/// object out.
fn parse_json<T: for<'de> Deserialize<'de>>(text: &str) -> Option<T> {
    let trimmed = text.trim();
    if let Ok(value) = serde_json::from_str(trimmed) {
        return Some(value);
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str(&trimmed[start..=end]).ok()
}

// ---------------------------------------------------------------------------
// Schema translation
// ---------------------------------------------------------------------------

/// Strip the Gemini-only hints so other providers don't reject the schema.
fn plain_schema(schema: &Value) -> Value {
    let mut out = schema.clone();
    strip(&mut out);
    return out;

    fn strip(value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.remove("propertyOrdering");
                for (_, v) in map.iter_mut() {
                    strip(v);
                }
            }
            Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
}

/// OpenAI's strict mode refuses optional properties and open objects, so every
/// property becomes required and every object closed. Fields the model has
/// nothing to say about come back empty rather than absent, which is what our
/// deserialisers already expect.
fn strict_schema(schema: &Value) -> Value {
    let mut out = plain_schema(schema);
    tighten(&mut out);
    return out;

    fn tighten(value: &mut Value) {
        let Some(map) = value.as_object_mut() else {
            if let Some(items) = value.as_array_mut() {
                items.iter_mut().for_each(tighten);
            }
            return;
        };
        if map.get("type").and_then(Value::as_str) == Some("object") {
            map.insert("additionalProperties".into(), Value::Bool(false));
            if let Some(properties) = map.get("properties").and_then(Value::as_object) {
                let names: Vec<Value> = properties.keys().cloned().map(Value::String).collect();
                map.insert("required".into(), Value::Array(names));
            }
        }
        for (_, v) in map.iter_mut() {
            tighten(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outline_schema() -> Value {
        crate::ai::prompt::outline_schema()
    }

    #[test]
    fn plain_schema_drops_gemini_only_keys() {
        let schema = plain_schema(&outline_schema());
        assert!(schema.get("propertyOrdering").is_none());
        assert!(schema.pointer("/properties/title").is_some());
    }

    #[test]
    fn strict_schema_requires_every_property_and_closes_objects() {
        let schema = strict_schema(&outline_schema());
        let required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required.contains(&"prerequisites"), "{required:?}");
        assert_eq!(schema["additionalProperties"], Value::Bool(false));
    }

    #[test]
    fn parses_json_wrapped_in_a_code_fence() {
        let value: StepText =
            parse_json("```json\n{\"title\":\"Open it\",\"body\":\"Click.\"}\n```").unwrap();
        assert_eq!(value.title, "Open it");
    }

    #[test]
    fn parses_json_with_a_chatty_preamble() {
        let value: StepText =
            parse_json("Sure! Here you go:\n{\"title\":\"Save\",\"body\":\"Click Save.\"}").unwrap();
        assert_eq!(value.body, "Click Save.");
    }

    #[test]
    fn rejects_text_with_no_json_in_it() {
        assert!(parse_json::<StepText>("I cannot help with that.").is_none());
    }

    #[test]
    fn loopback_detection_covers_the_usual_spellings() {
        assert!(is_loopback("http://127.0.0.1:11434"));
        assert!(is_loopback("http://localhost:1234/v1"));
        assert!(!is_loopback("https://api.openai.com/v1"));
    }

    /// `Client` holds a `reqwest::Client`, which isn't `Debug`, so `unwrap_err`
    /// is off the table.
    fn expect_error(result: Result<Client>) -> String {
        match result {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected the client to be rejected"),
        }
    }

    #[test]
    fn formats_mistral_validation_errors() {
        let payload = r#"{"object":"error","message":{"detail":[{"type":"extra_forbidden","loc":["body","temperature"],"msg":"Extra inputs are not permitted","input":0.35}]},"type":"invalid_request_error"}"#;
        let client = Client::new(
            Provider::Mistral,
            "mistral-small-latest".into(),
            String::new(),
            "k".into(),
        )
        .unwrap();
        let message = client.error_message(payload).unwrap();
        assert!(message.contains("Extra inputs are not permitted"), "{message}");
        assert!(message.contains("temperature"), "{message}");
    }

    #[test]
    fn a_blank_model_is_rejected_before_any_request() {
        let error = expect_error(Client::new(
            Provider::Gemini,
            "  ".into(),
            String::new(),
            "k".into(),
        ));
        assert!(error.contains("No model is selected"), "{error}");
    }

    #[test]
    fn a_custom_endpoint_without_an_address_is_rejected() {
        let error = expect_error(Client::new(
            Provider::Compatible,
            "some-model".into(),
            String::new(),
            String::new(),
        ));
        assert!(error.contains("server address"), "{error}");
    }

    #[test]
    fn every_provider_builds_a_body_and_a_url() {
        for provider in Provider::ALL {
            let base = if provider == Provider::Compatible {
                "http://example.test/v1".to_string()
            } else {
                String::new()
            };
            let client = Client::new(provider, "test-model".into(), base, "key".into()).unwrap();
            let ask = Ask {
                system: "be helpful",
                prompt: "describe this",
                images: vec![InlineImage(vec![1, 2, 3])],
                schema: outline_schema(),
                schema_name: "outline",
                temperature: 0.3,
                max_tokens: 512,
            };
            assert!(client.url().starts_with("http"), "{}", provider.id());
            let body = client.body(&ask);
            let encoded = serde_json::to_string(&body).unwrap();
            assert!(
                encoded.contains("describe this"),
                "{} dropped the prompt",
                provider.id()
            );
            assert!(
                encoded.contains("AQID"),
                "{} dropped the image",
                provider.id()
            );
        }
    }
}
