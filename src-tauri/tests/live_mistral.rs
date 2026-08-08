//! Optional live checks against real provider APIs.
//!
//! Run with:
//!   RUN_LIVE_TESTS=1 cargo test --manifest-path src-tauri/Cargo.toml --test live_mistral -- --nocapture

use std::path::PathBuf;

use steppy_lib::probe::{self, provider, Provider};

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
        .block_on(future)
}

fn credentials_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| {
        home.join("Library/Application Support/app.steppy.desktop/credentials.mistral")
    })
}

fn sample_frame() -> Option<PathBuf> {
    dirs::home_dir().and_then(|home| {
        let root = home.join("Library/Application Support/app.steppy.desktop/projects");
        std::fs::read_dir(root)
            .ok()?
            .filter_map(|e| e.ok())
            .find_map(|entry| {
                let frames = entry.path().join("frames");
                std::fs::read_dir(frames).ok()?.find_map(|f| f.ok().map(|f| f.path()))
            })
    })
}

fn prepare(path: &PathBuf) -> provider::InlineImage {
    let image = probe::load(path).expect("load frame");
    let image = probe::fit_width(image, 1280);
    provider::InlineImage(probe::encode_jpeg(&image, 82).expect("encode jpeg"))
}

#[test]
fn mistral_describes_a_real_frame() {
    if std::env::var("RUN_LIVE_TESTS").ok().as_deref() != Some("1") {
        return;
    }

    let key = std::fs::read_to_string(credentials_path().expect("credentials path"))
        .expect("read mistral key")
        .trim()
        .to_string();
    let frame = sample_frame().expect("sample frame");
    let image = prepare(&frame);

    let client = provider::Client::new(
        Provider::Mistral,
        "mistral-large-latest".into(),
        String::new(),
        key,
    )
    .expect("client");

    let step = block_on(client.describe(
        "You are a writer.\n\nRespond with one JSON object only, no markdown fences or commentary: {\"title\": string, \"body\": string}.",
        "Look at the screenshot and return title and body for step 1 of 5.",
        image,
    ))
    .expect("describe");

    assert!(!step.title.trim().is_empty(), "empty title: {step:?}");
    assert!(!step.body.trim().is_empty(), "empty body: {step:?}");
    eprintln!("title: {}", step.title);
    eprintln!("body: {}", step.body);
}
