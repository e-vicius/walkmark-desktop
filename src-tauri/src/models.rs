use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

fn now() -> DateTime<Utc> {
    Utc::now()
}

fn new_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..12].to_string()
}

// ---------------------------------------------------------------------------
// Capture sources
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Monitor,
    Window,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSource {
    /// Opaque handle of the form `monitor:<id>` or `window:<id>`.
    pub id: String,
    pub kind: SourceKind,
    pub name: String,
    /// Secondary line in the picker: app name for windows, resolution for monitors.
    pub detail: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
    /// Small PNG data URL used for the live preview grid.
    pub thumbnail: Option<String>,
}

// ---------------------------------------------------------------------------
// Annotations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationKind {
    /// Mosaic the region — reversible-looking but destructive in exports.
    Blur,
    /// Paint the region solid so nothing can be recovered.
    Redact,
    /// Draw an accent outline to draw the eye.
    Highlight,
}

/// Rectangle in normalized (0..1) image space so annotations survive
/// re-framing and any downscaling we apply on export.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    /// Clamp to the image and convert to integer pixel bounds.
    pub fn to_pixels(self, img_w: u32, img_h: u32) -> (u32, u32, u32, u32) {
        let x0 = (self.x.clamp(0.0, 1.0) * img_w as f32).round() as u32;
        let y0 = (self.y.clamp(0.0, 1.0) * img_h as f32).round() as u32;
        let x1 = ((self.x + self.w).clamp(0.0, 1.0) * img_w as f32).round() as u32;
        let y1 = ((self.y + self.h).clamp(0.0, 1.0) * img_h as f32).round() as u32;
        (
            x0.min(img_w),
            y0.min(img_h),
            x1.saturating_sub(x0).min(img_w - x0.min(img_w)),
            y1.saturating_sub(y0).min(img_h - y0.min(img_h)),
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationStroke {
    #[default]
    Medium,
    Thin,
    Thick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    #[serde(default = "new_id")]
    pub id: String,
    pub kind: AnnotationKind,
    pub rect: Rect,
    /// `#rrggbb` — highlight outline and redact fill.
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub stroke: Option<AnnotationStroke>,
}

// ---------------------------------------------------------------------------
// Steps
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Captured, no description yet.
    Draft,
    Queued,
    Generating,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub id: String,
    pub title: String,
    pub body: String,
    /// Milliseconds from the start of the recording session.
    pub offset_ms: u64,
    /// File name (not path) of the chosen frame inside the project's `frames/` dir.
    pub frame: String,
    /// Neighbouring frames the user can swap in when the capture was mistimed.
    #[serde(default)]
    pub alternates: Vec<String>,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    #[serde(default = "default_true")]
    pub include: bool,
    /// Set once a human edits the text — regeneration then skips this step
    /// unless it is explicitly targeted.
    #[serde(default)]
    pub locked: bool,
    pub status: StepStatus,
    #[serde(default)]
    pub error: Option<String>,
    /// True when the step was pinned by the user rather than auto-detected.
    #[serde(default)]
    pub manual: bool,
}

fn default_true() -> bool {
    true
}

impl Step {
    pub fn new(frame: String, offset_ms: u64, manual: bool) -> Self {
        Self {
            id: new_id(),
            title: String::new(),
            body: String::new(),
            offset_ms,
            frame,
            alternates: Vec::new(),
            annotations: Vec::new(),
            include: true,
            locked: false,
            status: StepStatus::Draft,
            error: None,
            manual,
        }
    }
}

// ---------------------------------------------------------------------------
// Product vocabulary
// ---------------------------------------------------------------------------

/// One preferred term and what it means in this product.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyTerm {
    #[serde(default = "new_id")]
    pub id: String,
    #[serde(default)]
    pub term: String,
    #[serde(default)]
    pub explanation: String,
}

fn parse_vocabulary_string(raw: &str) -> Vec<VocabularyTerm> {
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            Some(VocabularyTerm {
                id: new_id(),
                term: line.to_string(),
                explanation: String::new(),
            })
        })
        .collect()
}

fn deserialize_vocabulary<'de, D>(deserializer: D) -> Result<Vec<VocabularyTerm>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct VocabVisitor;

    impl<'de> Visitor<'de> for VocabVisitor {
        type Value = Vec<VocabularyTerm>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a vocabulary string or array of terms")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            Ok(parse_vocabulary_string(value))
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut terms = Vec::new();
            while let Some(term) = seq.next_element::<VocabularyTerm>()? {
                terms.push(term);
            }
            Ok(terms)
        }
    }

    deserializer.deserialize_any(VocabVisitor)
}

/// Named vocabulary profile for a product the user documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    #[serde(default = "new_id")]
    pub id: String,
    pub name: String,
    /// Preferred terms and what they mean in this product.
    #[serde(default, deserialize_with = "deserialize_vocabulary")]
    pub vocabulary: Vec<VocabularyTerm>,
}

impl Product {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            name: name.into(),
            vocabulary: Vec::new(),
        }
    }

    pub fn normalize_vocabulary(&mut self) {
        for term in &mut self.vocabulary {
            if term.id.is_empty() {
                term.id = new_id();
            }
        }
    }

    /// Rendered for the model prompt.
    pub fn format_vocabulary(&self) -> String {
        self.vocabulary
            .iter()
            .filter(|t| !t.term.trim().is_empty())
            .map(|t| {
                let term = t.term.trim();
                let explanation = t.explanation.trim();
                if explanation.is_empty() {
                    format!("- {term}")
                } else {
                    format!("- {term} — {explanation}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ---------------------------------------------------------------------------
// Project
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub source_label: String,
    /// Which product vocabulary applies to this guide.
    #[serde(default)]
    pub product_id: Option<String>,
    #[serde(default)]
    pub steps: Vec<Step>,
}

impl Project {
    pub fn new(title: impl Into<String>, source_label: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            title: title.into(),
            summary: String::new(),
            prerequisites: Vec::new(),
            created_at: now(),
            updated_at: now(),
            source_label: source_label.into(),
            product_id: None,
            steps: Vec::new(),
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = now();
    }
}

/// Lightweight row for the project library, so we never load every frame list
/// just to render the home screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub title: String,
    pub updated_at: DateTime<Utc>,
    pub step_count: usize,
    pub ready_count: usize,
    /// Absolute path to the cover frame, for `convertFileSrc`.
    pub cover: Option<String>,
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Neutral,
    Friendly,
    Formal,
    Playful,
}

impl Tone {
    pub fn describe(self) -> &'static str {
        match self {
            Tone::Neutral => "plain, matter-of-fact and neutral",
            Tone::Friendly => "warm and encouraging, but never chatty",
            Tone::Formal => "formal and precise, suitable for a compliance manual",
            Tone::Playful => "light and personable, with the occasional bit of character",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSettings {
    /// How often we grab a frame while recording.
    pub sample_interval_ms: u64,
    /// 0 = only huge changes become steps, 1 = almost every change does.
    /// Used only when [`Self::visual_fallback`] is enabled.
    pub sensitivity: f32,
    /// Never emit two steps closer together than this.
    pub min_gap_ms: u64,
    /// Wait for the screen to stop changing before committing a step, so we
    /// don't capture half-open menus or mid-flight animations.
    /// Used only when [`Self::visual_fallback`] is enabled.
    pub settle: bool,
    /// Fall back to visual change detection when input monitoring is unavailable
    /// or as a supplement to catch transitions input misses.
    #[serde(default)]
    pub visual_fallback: bool,
    /// After a click, key press or scroll, wait this long before taking the
    /// screenshot so menus and typed text have time to appear.
    #[serde(default = "default_input_settle_ms")]
    pub input_settle_ms: u64,
    /// Longest edge of stored frames. Keeps retina captures from ballooning.
    pub max_width: u32,
    pub countdown_secs: u32,
    /// Hide the main window while recording and show the floating HUD instead.
    pub hide_window: bool,
}

fn default_input_settle_ms() -> u64 {
    300
}

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            sample_interval_ms: 200,
            sensitivity: 0.55,
            min_gap_ms: 800,
            settle: true,
            visual_fallback: false,
            input_settle_ms: 300,
            max_width: 1800,
            countdown_secs: 3,
            hide_window: true,
        }
    }
}

/// Where the writing happens. Everything behind this enum speaks a different
/// wire protocol, but the rest of the app only ever sees `Provider`.
#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Provider {
    #[default]
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "mistral")]
    Mistral,
    /// One API key, many cloud models through openrouter.ai.
    #[serde(rename = "openrouter")]
    OpenRouter,
    /// Models running on this machine, managed through Ollama.
    #[serde(rename = "ollama")]
    Ollama,
    /// Any server that speaks the OpenAI chat-completions protocol — LM Studio,
    /// llama.cpp, vLLM, a corporate gateway.
    #[serde(rename = "compatible")]
    Compatible,
}

impl Provider {
    pub const ALL: [Provider; 7] = [
        Provider::Gemini,
        Provider::OpenAi,
        Provider::Anthropic,
        Provider::Mistral,
        Provider::OpenRouter,
        Provider::Ollama,
        Provider::Compatible,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Provider::Gemini => "gemini",
            Provider::OpenAi => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Mistral => "mistral",
            Provider::OpenRouter => "openrouter",
            Provider::Ollama => "ollama",
            Provider::Compatible => "compatible",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Provider::ALL.into_iter().find(|p| p.id() == id)
    }

    /// Human name, used in error messages so they read naturally.
    pub fn label(self) -> &'static str {
        match self {
            Provider::Gemini => "Gemini",
            Provider::OpenAi => "OpenAI",
            Provider::Anthropic => "Claude",
            Provider::Mistral => "Mistral",
            Provider::OpenRouter => "OpenRouter",
            Provider::Ollama => "Ollama",
            Provider::Compatible => "The server",
        }
    }

    /// Local providers never need a key and never send anything off the machine.
    pub fn is_local(self) -> bool {
        matches!(self, Provider::Ollama)
    }

    pub fn needs_key(self) -> bool {
        !matches!(self, Provider::Ollama)
    }

    /// A custom endpoint is required for `Compatible` and optional elsewhere.
    pub fn default_base_url(self) -> &'static str {
        match self {
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Provider::OpenAi => "https://api.openai.com/v1",
            Provider::Anthropic => "https://api.anthropic.com/v1",
            Provider::Mistral => "https://api.mistral.ai/v1",
            Provider::OpenRouter => "https://openrouter.ai/api/v1",
            Provider::Ollama => "http://127.0.0.1:11434",
            Provider::Compatible => "",
        }
    }
}

/// The model and endpoint chosen for one provider. Kept per provider so that
/// switching to Ollama to try something and switching back doesn't lose the
/// cloud model you had picked.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    #[serde(default)]
    pub model: String,
    /// Empty means "use the provider's default endpoint".
    #[serde(default)]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Which provider generation runs against right now.
    #[serde(default)]
    pub provider: Provider,
    /// Saved selection per provider, keyed by `Provider::id`.
    #[serde(default)]
    pub providers: BTreeMap<Provider, ProviderConfig>,
    /// Free-text description of who will read the doc.
    pub audience: String,
    pub tone: Tone,
    pub language: String,
    /// Parallel requests. Higher is faster but hits rate limits sooner, and
    /// local models generally want this at 1.
    pub concurrency: usize,
    pub capture: CaptureSettings,
    pub theme: String,
    pub onboarded: bool,
    /// Named vocabulary profiles. At least one is always present.
    #[serde(default = "default_products")]
    pub products: Vec<Product>,
    /// Last product picked when starting a recording.
    #[serde(default)]
    pub default_product_id: Option<String>,
}

fn default_products() -> Vec<Product> {
    vec![Product::new("General")]
}

impl Default for Settings {
    fn default() -> Self {
        let general = Product::new("General");
        let default_id = general.id.clone();
        Self {
            provider: Provider::default(),
            providers: BTreeMap::new(),
            audience: "a colleague who has never used this tool before".into(),
            tone: Tone::Neutral,
            language: "English".into(),
            concurrency: 3,
            capture: CaptureSettings::default(),
            theme: "system".into(),
            onboarded: false,
            products: vec![general],
            default_product_id: Some(default_id),
        }
    }
}

impl Settings {
    /// Guarantee a usable product list after loading older settings files.
    pub fn clamp_text(&mut self) {
        self.audience = crate::limits::clamp(&self.audience, crate::limits::AUDIENCE);
        self.language = crate::limits::clamp(&self.language, crate::limits::LANGUAGE);
        for product in &mut self.products {
            product.name =
                crate::limits::clamp(&product.name, crate::limits::PRODUCT_NAME);
            for term in &mut product.vocabulary {
                term.term =
                    crate::limits::clamp(&term.term, crate::limits::VOCABULARY_TERM);
                term.explanation = crate::limits::clamp(
                    &term.explanation,
                    crate::limits::VOCABULARY_EXPLANATION,
                );
            }
        }
        for config in self.providers.values_mut() {
            config.model = crate::limits::clamp(&config.model, crate::limits::MODEL_ID);
            config.base_url =
                crate::limits::clamp(&config.base_url, crate::limits::BASE_URL);
        }
    }

    pub fn normalize_products(&mut self) {
        if self.products.is_empty() {
            let general = Product::new("General");
            self.default_product_id = Some(general.id.clone());
            self.products.push(general);
        }
        for product in &mut self.products {
            product.normalize_vocabulary();
        }
        self.clamp_text();
        if self.default_product_id.is_none()
            || self
                .default_product_id
                .as_ref()
                .is_some_and(|id| self.product(id).is_none())
        {
            self.default_product_id = self.products.first().map(|p| p.id.clone());
        }
    }

    pub fn product(&self, id: &str) -> Option<&Product> {
        self.products.iter().find(|p| p.id == id)
    }

    pub fn resolve_product(&self, id: Option<&str>) -> Option<&Product> {
        id.and_then(|id| self.product(id))
    }

    pub fn config_for(&self, provider: Provider) -> ProviderConfig {
        let saved = self.providers.get(&provider);
        let model = saved
            .map(|c| c.model.trim())
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| crate::ai::catalog::default_model(provider))
            .to_string();
        let base_url = saved
            .map(|c| c.base_url.trim())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| provider.default_base_url())
            .trim_end_matches('/')
            .to_string();
        ProviderConfig { model, base_url }
    }
}

// ---------------------------------------------------------------------------
// Events emitted to the frontend
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecordingState {
    Idle,
    Counting,
    Recording,
    Paused,
    Stopping,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingTick {
    pub state: RecordingState,
    pub elapsed_ms: u64,
    pub step_count: usize,
    /// 0..1 measure of how much the screen just changed.
    pub activity: f32,
    pub countdown: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationProgress {
    pub done: usize,
    pub total: usize,
    pub running: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Markdown,
    Html,
    Pdf,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::Markdown => "md",
            ExportFormat::Html => "html",
            ExportFormat::Pdf => "pdf",
        }
    }
}
