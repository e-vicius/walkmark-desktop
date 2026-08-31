//! End-to-end checks against a stub server.
//!
//! The unit tests cover request building and response parsing in isolation.
//! These cover the part that only shows up over a real socket: headers landing
//! where each provider expects them, chunked JSON-lines arriving in pieces that
//! don't line up with the lines, and cancelling a download mid-transfer.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::Value;
use walkmark_lib::probe::{local, provider, Provider};

/// What the stub saw, so a test can assert on the request as well as use the
/// response.
#[derive(Clone, Default)]
struct Seen {
    path: String,
    headers: Vec<(String, String)>,
    body: Value,
}

struct Stub {
    base: String,
    seen: Arc<Mutex<Vec<Seen>>>,
    _handle: thread::JoinHandle<()>,
}

impl Stub {
    /// Serves `routes` — a function from request path to (status, body) — for a
    /// fixed number of requests, then shuts down.
    fn new<F>(requests: usize, respond: F) -> Self
    where
        F: Fn(&Seen, &mut TcpStream) + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a loopback port");
        let base = format!("http://{}", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorder = Arc::clone(&seen);

        let handle = thread::spawn(move || {
            for _ in 0..requests {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let request = read_request(&mut stream);
                recorder.lock().unwrap().push(request.clone());
                respond(&request, &mut stream);
                let _ = stream.flush();
            }
        });

        Self {
            base,
            seen,
            _handle: handle,
        }
    }

    fn requests(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
}

fn read_request(stream: &mut TcpStream) -> Seen {
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let mut start = String::new();
    reader.read_line(&mut start).unwrap();
    let path = start.split_whitespace().nth(1).unwrap_or("/").to_string();

    let mut headers = Vec::new();
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.trim().split_once(": ") {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.parse().unwrap_or(0);
            }
            headers.push((name.to_ascii_lowercase(), value.to_string()));
        }
    }

    let mut raw = vec![0u8; length];
    if length > 0 {
        reader.read_exact(&mut raw).unwrap();
    }

    Seen {
        path,
        headers,
        body: serde_json::from_slice(&raw).unwrap_or(Value::Null),
    }
}

fn reply(stream: &mut TcpStream, status: u16, body: &str) {
    let _ = write!(
        stream,
        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
}

impl Seen {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

fn image() -> provider::InlineImage {
    provider::InlineImage(vec![0xff, 0xd8, 0xff])
}

// ---------------------------------------------------------------------------
// Each provider, over a real socket
// ---------------------------------------------------------------------------

#[test]
fn gemini_sends_its_key_as_a_header_and_reads_the_candidate_back() {
    let stub = Stub::new(1, |_, stream| {
        reply(
            stream,
            200,
            r#"{"candidates":[{"content":{"parts":[{"text":"{\"title\":\"Open Settings\",\"body\":\"Click the gear.\"}"}]}}]}"#,
        )
    });

    let client = provider::Client::new(
        Provider::Gemini,
        "gemini-3.6-flash".into(),
        stub.base.clone(),
        "test-key".into(),
    )
    .unwrap();

    let step = block_on(client.describe("be helpful", "what now", image())).unwrap();
    assert_eq!(step.title, "Open Settings");

    let request = &stub.requests()[0];
    assert_eq!(request.header("x-goog-api-key"), Some("test-key"));
    assert!(request.path.ends_with("/models/gemini-3.6-flash:generateContent"));
    assert!(request.body["contents"][0]["parts"][1]["inlineData"]["data"].is_string());
}

#[test]
fn openai_sends_a_bearer_token_and_reads_the_choice_back() {
    let stub = Stub::new(1, |_, stream| {
        reply(
            stream,
            200,
            r#"{"choices":[{"message":{"content":"{\"title\":\"Save it\",\"body\":\"Click Save.\"}"}}]}"#,
        )
    });

    let client = provider::Client::new(
        Provider::OpenAi,
        "gpt-5.6-terra".into(),
        stub.base.clone(),
        "sk-test".into(),
    )
    .unwrap();

    let step = block_on(client.describe("be helpful", "what now", image())).unwrap();
    assert_eq!(step.title, "Save it");

    let request = &stub.requests()[0];
    assert_eq!(request.header("authorization"), Some("Bearer sk-test"));
    assert_eq!(request.path, "/v1/chat/completions");
    assert_eq!(request.body["response_format"]["json_schema"]["strict"], true);
}

#[test]
fn mistral_uses_object_image_urls_without_structured_modes() {
    let stub = Stub::new(1, |_, stream| {
        reply(
            stream,
            200,
            r#"{"choices":[{"message":{"content":"{\"title\":\"Save it\",\"body\":\"Click Save.\"}"}}]}"#,
        )
    });

    let client = provider::Client::new(
        Provider::Mistral,
        "mistral-small-latest".into(),
        stub.base.clone(),
        "mistral-test".into(),
    )
    .unwrap();

    let step = block_on(client.describe("be helpful", "what now", image())).unwrap();
    assert_eq!(step.title, "Save it");

    let request = &stub.requests()[0];
    assert_eq!(request.header("authorization"), Some("Bearer mistral-test"));
    assert_eq!(request.path, "/v1/chat/completions");
    assert!(request.body.get("response_format").is_none());
    assert!(request.body.get("tools").is_none());
    assert!(request.body["messages"][1]["content"][1]["image_url"].is_string());
}

#[test]
fn anthropic_sends_its_version_header_and_reads_the_forced_tool_call() {
    let stub = Stub::new(1, |_, stream| {
        reply(
            stream,
            200,
            r#"{"content":[{"type":"tool_use","name":"step","input":{"title":"Pick a plan","body":"Choose one."}}]}"#,
        )
    });

    let client = provider::Client::new(
        Provider::Anthropic,
        "claude-sonnet-5".into(),
        stub.base.clone(),
        "sk-ant-test".into(),
    )
    .unwrap();

    let step = block_on(client.describe("be helpful", "what now", image())).unwrap();
    assert_eq!(step.title, "Pick a plan");

    let request = &stub.requests()[0];
    assert_eq!(request.header("x-api-key"), Some("sk-ant-test"));
    assert!(request.header("anthropic-version").is_some());
    assert_eq!(request.path, "/messages");
    assert_eq!(request.body["tool_choice"]["name"], "step");
}

#[test]
fn ollama_needs_no_key_and_reads_the_message_back() {
    let stub = Stub::new(1, |_, stream| {
        reply(
            stream,
            200,
            r#"{"message":{"role":"assistant","content":"{\"title\":\"Type a name\",\"body\":\"Enter it.\"}"}}"#,
        )
    });

    let client = provider::Client::new(
        Provider::Ollama,
        "qwen3-vl:8b".into(),
        stub.base.clone(),
        String::new(),
    )
    .unwrap();

    let step = block_on(client.describe("be helpful", "what now", image())).unwrap();
    assert_eq!(step.title, "Type a name");

    let request = &stub.requests()[0];
    assert_eq!(request.path, "/api/chat");
    assert!(request.header("authorization").is_none());
    assert_eq!(request.body["messages"][1]["images"].as_array().unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// Failure paths
// ---------------------------------------------------------------------------

#[test]
fn a_rejected_key_reports_the_providers_own_message() {
    let stub = Stub::new(1, |_, stream| {
        reply(stream, 401, r#"{"error":{"message":"API key not valid."}}"#)
    });

    let client = provider::Client::new(
        Provider::Gemini,
        "gemini-3.6-flash".into(),
        stub.base.clone(),
        "wrong".into(),
    )
    .unwrap();

    let error = block_on(client.describe("s", "p", image())).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("API key not valid."), "{message}");
    // And it tells them which model the key needs to reach.
    assert!(message.contains("gemini-3.6-flash"), "{message}");
}

#[test]
fn a_rate_limit_is_retried_and_then_succeeds() {
    let attempt = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&attempt);
    let stub = Stub::new(2, move |_, stream| {
        if flag.swap(true, Ordering::SeqCst) {
            reply(
                stream,
                200,
                r#"{"choices":[{"message":{"content":"{\"title\":\"Done\",\"body\":\"Finished.\"}"}}]}"#,
            );
        } else {
            reply(stream, 429, r#"{"error":{"message":"Slow down."}}"#);
        }
    });

    let client = provider::Client::new(
        Provider::OpenAi,
        "gpt-5.6-terra".into(),
        stub.base.clone(),
        "sk-test".into(),
    )
    .unwrap();

    let step = block_on(client.describe("s", "p", image())).unwrap();
    assert_eq!(step.title, "Done");
    assert_eq!(stub.requests().len(), 2);
}

#[test]
fn a_local_model_that_is_not_downloaded_says_so_plainly() {
    let stub = Stub::new(1, |_, stream| {
        reply(stream, 404, r#"{"error":"model 'nope' not found"}"#)
    });

    let client = provider::Client::new(
        Provider::Ollama,
        "nope".into(),
        stub.base.clone(),
        String::new(),
    )
    .unwrap();

    let error = block_on(client.describe("s", "p", image())).unwrap_err();
    assert_eq!(error.kind(), "local_model_missing");
}

#[test]
fn an_unreachable_local_daemon_is_reported_as_such() {
    // Nothing is listening on this port.
    let client = provider::Client::new(
        Provider::Ollama,
        "qwen3-vl:8b".into(),
        "http://127.0.0.1:1".into(),
        String::new(),
    )
    .unwrap();

    let error = block_on(client.describe("s", "p", image())).unwrap_err();
    assert_eq!(error.kind(), "local_runtime_unavailable");
}

// ---------------------------------------------------------------------------
// Downloading a model
// ---------------------------------------------------------------------------

/// Ollama streams JSON lines, and TCP does not respect line boundaries. Split
/// the payload somewhere awkward to prove the reader reassembles it.
fn stream_pull(stream: &mut TcpStream) {
    let body = concat!(
        "{\"status\":\"pulling manifest\"}\n",
        "{\"status\":\"pulling 1a2b\",\"completed\":250,\"total\":1000}\n",
        "{\"status\":\"pulling 1a2b\",\"completed\":1000,\"total\":1000}\n",
        "{\"status\":\"success\"}\n",
    );
    let _ = write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let split = 45;
    let _ = stream.write_all(&body.as_bytes()[..split]);
    let _ = stream.flush();
    thread::sleep(Duration::from_millis(20));
    let _ = stream.write_all(&body.as_bytes()[split..]);
}

#[test]
fn a_download_reports_progress_and_finishes() {
    let stub = Stub::new(1, |_, stream| stream_pull(stream));

    let (tx, rx) = mpsc::channel();
    let result = block_on(local::pull_with(
        &stub.base,
        "moondream",
        Arc::new(AtomicBool::new(false)),
        move |progress| {
            let _ = tx.send(progress);
        },
    ));
    assert!(result.is_ok(), "{result:?}");

    let updates: Vec<_> = rx.iter().collect();
    let byte_counts: Vec<(u64, u64)> = updates
        .iter()
        .filter(|u| u.total > 0)
        .map(|u| (u.completed, u.total))
        .collect();
    assert!(
        byte_counts.contains(&(250, 1000)),
        "no partial progress in {byte_counts:?}"
    );

    let last = updates.last().unwrap();
    assert!(last.done);
    assert!(last.error.is_none());
    assert_eq!(stub.requests()[0].path, "/api/pull");
}

#[test]
fn a_failure_partway_through_a_download_surfaces_the_reason() {
    let stub = Stub::new(1, |_, stream| {
        let body = concat!(
            "{\"status\":\"pulling manifest\"}\n",
            "{\"error\":\"no space left on device\"}\n",
        );
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
    });

    let (tx, rx) = mpsc::channel();
    let error = block_on(local::pull_with(
        &stub.base,
        "moondream",
        Arc::new(AtomicBool::new(false)),
        move |progress| {
            let _ = tx.send(progress);
        },
    ))
    .unwrap_err();

    assert!(error.to_string().contains("no space left"), "{error}");
    let last = rx.iter().last().unwrap();
    assert_eq!(last.error.as_deref(), Some("no space left on device"));
}

#[test]
fn cancelling_a_download_stops_it() {
    let stub = Stub::new(1, |_, stream| {
        // A long, slow stream the test will interrupt rather than finish.
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
        );
        for completed in (0..100).map(|i| i * 10) {
            let line =
                format!("{{\"status\":\"pulling\",\"completed\":{completed},\"total\":1000}}\n");
            if write!(stream, "{:x}\r\n{line}\r\n", line.len()).is_err() {
                return;
            }
            if stream.flush().is_err() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
    });

    let cancel = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancel);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(120));
        flag.store(true, Ordering::SeqCst);
    });

    let error = block_on(local::pull_with(&stub.base, "moondream", cancel, |_| {})).unwrap_err();
    assert_eq!(error.kind(), "cancelled");
}

#[test]
fn status_reports_installed_models_and_whether_they_can_see() {
    let stub = Stub::new(4, |request, stream| match request.path.as_str() {
        "/api/version" => reply(stream, 200, r#"{"version":"0.6.2"}"#),
        "/api/tags" => reply(
            stream,
            200,
            r#"{"models":[
                {"name":"qwen3-vl:8b","size":6100000000,
                 "details":{"parameter_size":"8.3B","quantization_level":"Q4_K_M"}},
                {"name":"llama3:8b","size":4700000000,
                 "details":{"parameter_size":"8.0B","quantization_level":"Q4_0"}}
            ]}"#,
        ),
        "/api/show" => {
            let vision = request.body["model"].as_str() == Some("qwen3-vl:8b");
            let capabilities = if vision {
                r#"["completion","vision"]"#
            } else {
                r#"["completion"]"#
            };
            reply(stream, 200, &format!(r#"{{"capabilities":{capabilities}}}"#))
        }
        _ => reply(stream, 404, "{}"),
    });

    let status = block_on(local::status(&stub.base));
    assert!(status.running);
    assert_eq!(status.version.as_deref(), Some("0.6.2"));
    assert_eq!(status.models.len(), 2);

    let vision = status.models.iter().find(|m| m.id == "qwen3-vl:8b").unwrap();
    assert!(vision.vision);
    assert_eq!(vision.parameters, "8.3B");

    // A text-only model is downloaded but useless here, and must be marked so.
    let text_only = status.models.iter().find(|m| m.id == "llama3:8b").unwrap();
    assert!(!text_only.vision);
}

#[test]
fn status_stays_calm_when_nothing_is_listening() {
    let status = block_on(local::status("http://127.0.0.1:1"));
    assert!(!status.running);
    assert!(status.models.is_empty());
    // The install link is what the UI offers next, so it must always be there.
    assert!(status.download_url.starts_with("https://"));
}
