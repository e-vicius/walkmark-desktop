# Architecture

Walkmark is a **Tauri 2** desktop app: a **Svelte 5** frontend and a **Rust** backend in one process. Screen capture, AI calls, and file I/O run in Rust; the UI talks to Rust through Tauri **commands** and **events**.

## High-level flow

```
Record → frames on disk → edit in UI → Write (AI) → export Markdown/HTML/PDF
```

1. **Capture** samples a monitor or window, detects meaningful changes, saves PNG frames.
2. **Storage** persists projects as `project.json` + `frames/*.png` under app data.
3. **AI** runs a two-pass write: outline all frames, then per-step bodies (see [AI writing](ai-writing.md)).
4. **Export** renders included steps to Markdown (+ images folder), self-contained HTML, or PDF.

## Repository layout

```
src/                      Svelte 5 UI
  lib/
    store.svelte.ts       App state, dialogs, generation progress
    api.ts                Typed wrappers for Tauri invoke()
    components/           Library, editor, settings, export, HUD triggers
src-tauri/src/
  lib.rs                  Command registration, app setup
  commands.rs             IPC handlers (projects, capture, AI, export)
  capture/                Source listing, session, input monitoring, detect
  ai/                     Prompts, generation orchestration, model catalog
  ai/provider/            HTTP clients per provider (gemini, openai, …)
  export/                 markdown.rs, html.rs, pdf.rs
  imaging.rs              Resize, encode, burn annotations into frames
  storage.rs              settings.json, projects, API key files
  local.rs                Ollama status, model pull/remove
  window.rs               Main window + recording HUD, global shortcuts
  models.rs               Shared types and defaults
  state.rs                AppState (settings, open project, recording session)
```

## Frontend ↔ backend

### Commands (request/response)

Examples: `list_projects`, `start_recording`, `generate`, `export_document`, `save_settings`.

Defined in `commands.rs`, invoked from `api.ts` via `@tauri-apps/api/core`.

### Events (push from Rust)

Examples:

| Event | Purpose |
| --- | --- |
| `recording:tick` | Timer, step count, activity meter for HUD |
| `recording:step` | New frame captured |
| `recording:stopped` | Session finished |
| `recording:shortcut` | Global shortcut fired (stop/mark/pause) |
| `ai:progress` | Write job progress |
| `ai:step` | Single step finished writing |
| `ai:done` | Write job complete |
| `local:pull` | Ollama model download progress |

Subscribed in `store.svelte.ts` and `events.ts`.

## Capture pipeline

1. User picks a **CaptureSource** (monitor or window) from `xcap`.
2. `session.rs` runs a loop: input events (macOS) + periodic frame diff (`detect.rs`).
3. When a step fires, frame is resized to **max width** from settings and saved as PNG.
4. `project.json` gets a new `Step` with `frame` filename and timestamp.

Manual marks and global shortcuts call the same capture path.

## AI pipeline

1. **Outline pass** — all included frame images + `outline_prompt` → title, summary, prerequisites, step titles (JSON schema).
2. **Step pass** — for each step: one image + prior step text + `step_prompt` → title + body.
3. Provider chosen from settings; concurrency from settings (except Ollama = 1).

Prompts live in `ai/prompt.rs`. Provider-specific HTTP in `ai/provider/*.rs`.

## Export pipeline

1. Load project + included steps with rendered frames (annotations applied in `imaging.rs`).
2. **Markdown** — `.md` + `images/` directory beside it.
3. **HTML** — single file, base64 images, inlined CSS.
4. **PDF** — `printpdf` layout, A4, page numbers.

## Security

- **CSP** in `tauri.conf.json` limits network to provider API hosts + local Ollama.
- **Asset protocol** serves frame PNGs only from `$APPDATA/**`.
- API keys stored in app data credentials files and OS keychain (`app.walkmark`); never uploaded to Walkmark servers.

## Adding a provider

1. Add variant to `Provider` in `models.rs`.
2. Add catalog entries in `ai/catalog.rs` (models, key URL, default base URL).
3. Implement or reuse HTTP client in `ai/provider/`.
4. Wire `authorize`, `body`, `extract` in `ai/provider/mod.rs`.
5. Add provider host to `connect-src` in `tauri.conf.json`.
6. Add id to `ProviderId` in `src/lib/types.ts`.

See `mistral.rs` for a recent addition following this pattern.

## Tests

```bash
pnpm typecheck
cargo test --manifest-path src-tauri/Cargo.toml
```

- `tests/wire.rs` — provider HTTP against a local stub server
- `tests/live_mistral.rs` — optional live test with `MISTRAL_API_KEY`
- Unit tests in `storage.rs`, `ai/catalog.rs`, export helpers

## Related

- [Build from source](build-from-source.md)
- [Contributing](https://github.com/e-vicius/walkmark-desktop/blob/main/CONTRIBUTING.md)
