// Mirrors `src-tauri/src/models.rs`. Kept hand-written rather than generated so
// the UI can document each field in the vocabulary the interface uses.

export type SourceKind = "monitor" | "window";

export interface CaptureSource {
  id: string;
  kind: SourceKind;
  name: string;
  detail: string;
  width: number;
  height: number;
  isPrimary: boolean;
  thumbnail: string | null;
}

export type AnnotationKind = "blur" | "redact" | "highlight";

/** Normalised 0..1 coordinates, so annotations survive any resize. */
export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** Highlight outline weight. Omitted means medium. */
export type AnnotationStroke = "thin" | "medium" | "thick";

export interface Annotation {
  id: string;
  kind: AnnotationKind;
  rect: Rect;
  /** `#rrggbb` — used for highlight outlines and redact fills. */
  color?: string;
  /** Highlight outline weight. Ignored for blur and redact. */
  stroke?: AnnotationStroke;
}

export type StepStatus = "draft" | "queued" | "generating" | "ready" | "failed";

export interface Step {
  id: string;
  title: string;
  body: string;
  offsetMs: number;
  frame: string;
  alternates: string[];
  annotations: Annotation[];
  include: boolean;
  /** Set once a human edits the text; "rewrite everything" then skips it. */
  locked: boolean;
  status: StepStatus;
  error: string | null;
  manual: boolean;
}

export interface Project {
  id: string;
  title: string;
  summary: string;
  prerequisites: string[];
  createdAt: string;
  updatedAt: string;
  sourceLabel: string;
  /** Vocabulary profile used when writing this guide. */
  productId?: string | null;
  steps: Step[];
}

export interface ProjectSummary {
  id: string;
  title: string;
  updatedAt: string;
  stepCount: number;
  readyCount: number;
  cover: string | null;
}

export type Tone = "neutral" | "friendly" | "formal" | "playful";

export interface CaptureSettings {
  sampleIntervalMs: number;
  /** Used when visual fallback is on. 0 = huge changes only, 1 = almost everything. */
  sensitivity: number;
  minGapMs: number;
  settle: boolean;
  /** Capture on screen changes when input monitoring is unavailable. */
  visualFallback?: boolean;
  /** Delay after a click, key press, or scroll before the screenshot is taken. */
  inputSettleMs?: number;
  maxWidth: number;
  countdownSecs: number;
  hideWindow: boolean;
}

export type ProviderId =
  | "gemini"
  | "openai"
  | "anthropic"
  | "mistral"
  | "ollama"
  | "openrouter"
  | "compatible";

export interface ProviderConfig {
  model: string;
  /** Empty means "use the provider's default endpoint". */
  baseUrl: string;
}

export interface VocabularyTerm {
  id: string;
  term: string;
  explanation: string;
}

export interface Product {
  id: string;
  name: string;
  /** Preferred terms and what they mean in this product. */
  vocabulary: VocabularyTerm[];
}

export interface Settings {
  provider: ProviderId;
  /** Saved selection per provider. Sparse — absent means "use the default". */
  providers: Partial<Record<ProviderId, ProviderConfig>>;
  audience: string;
  tone: Tone;
  language: string;
  concurrency: number;
  capture: CaptureSettings;
  theme: "system" | "light" | "dark";
  onboarded: boolean;
  products: Product[];
  defaultProductId?: string | null;
}

export type RecordingState = "idle" | "counting" | "recording" | "paused" | "stopping";

export interface RecordingTick {
  state: RecordingState;
  elapsedMs: number;
  stepCount: number;
  /** 0..1 measure of how much the screen just changed. */
  activity: number;
  countdown: number;
}

export interface RecordingStatus {
  active: boolean;
  state: RecordingState;
  stepCount: number;
}

export interface GenerationProgress {
  done: number;
  total: number;
  running: boolean;
  message?: string;
}

export type ExportFormat = "markdown" | "html" | "pdf";

export interface ExportOptions {
  format: ExportFormat;
  includeImages: boolean;
  imageWidth: number;
  includeToc: boolean;
  includeSummary: boolean;
  includePrerequisites: boolean;
  theme: "auto" | "light" | "dark";
}

export interface ExportResult {
  path: string;
  assetsDir: string | null;
  bytes: number;
}

export interface PermissionStatus {
  granted: boolean;
  /** Only macOS gates screen capture today. */
  required: boolean;
  inputGranted: boolean;
  inputRequired: boolean;
}

/** Shape of every error crossing the IPC boundary. */
export interface AppError {
  kind:
    | "permission_denied"
    | "source_unavailable"
    | "already_recording"
    | "not_recording"
    | "missing_api_key"
    | "api_rejected"
    | "local_runtime_unavailable"
    | "local_model_missing"
    | "cancelled"
    | "not_found"
    | "invalid"
    | "capture"
    | "io"
    | "network"
    | "image"
    | "serde"
    | "other";
  message: string;
}

// --- Providers and models ---------------------------------------------------
//
// The catalog is served by the backend rather than duplicated here, so there is
// exactly one place to edit when a provider ships a new generation of models.

export type ModelTier = "recommended" | "fast" | "capable" | "older";

export interface ModelInfo {
  id: string;
  name: string;
  note: string;
  /** Can read screenshots in a write request. Defaults to true when omitted. */
  vision?: boolean;
  tier: ModelTier;
}

export interface ProviderInfo {
  id: ProviderId;
  name: string;
  blurb: string;
  /** Runs on this machine: no key, no cost, nothing leaves the device. */
  local: boolean;
  needsKey: boolean;
  keyUrl: string;
  keyHint: string;
  /** Numbered steps shown when the user needs an API key. */
  keyGuide: string[];
  baseUrlEditable: boolean;
  baseUrlRequired: boolean;
  defaultBaseUrl: string;
  defaultModel: string;
  /** Empty for providers whose models are discovered at runtime. */
  models: ModelInfo[];
}

export interface ProviderCatalog {
  providers: ProviderInfo[];
  /** Provider ids that already have a key stored. */
  configured: ProviderId[];
}

// --- Local models -----------------------------------------------------------

export interface InstalledModel {
  id: string;
  /** Bytes on disk. */
  size: number;
  parameters: string;
  quantization: string;
  /** False for text-only models, which cannot read a screenshot. */
  vision: boolean;
}

export interface LocalCatalogEntry {
  id: string;
  name: string;
  note: string;
  size: number;
  minMemory: number;
  recommended: boolean;
  installed: boolean;
  /** False when this machine has less memory than the model needs. */
  fits: boolean;
}

export interface LocalOverview {
  running: boolean;
  version: string | null;
  endpoint: string;
  models: InstalledModel[];
  totalMemory: number;
  downloadUrl: string;
  catalog: LocalCatalogEntry[];
  downloading: string | null;
}

export interface PullProgress {
  model: string;
  status: string;
  completed: number;
  total: number;
  done: boolean;
  error: string | null;
}
