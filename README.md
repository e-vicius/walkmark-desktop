# Steppy

Turn a workflow you do once into step-by-step documentation someone else can follow.

Steppy is a native desktop app. You pick a screen or window, do the task, and it captures each meaningful step with a screenshot. When you are ready, an AI model writes the instructions. Export as Markdown, a single HTML file, or PDF.

**Website:** [steppy.app](https://steppy.app) (landing page)  
**Hosted version:** [Steppy Cloud](https://steppy.app#cloud) ($7/mo, no API keys)

## Open source vs Cloud

| | **Steppy (this repo)** | **Steppy Cloud** |
| --- | --- | --- |
| Price | Free, MIT licence | $7/month |
| AI models | Bring your own keys (Gemini, OpenAI, Anthropic, OpenRouter, Ollama) | We run the models for you |
| Usage limits | Your provider's limits | Unlimited writing in the app |
| Accounts | None | Sign in inside the app |
| Extra features | Community driven | Team features shipping later |

Enterprise teams: [contact us](mailto:hello@steppy.app).

## How it works

1. **Record** — Steppy samples the screen, detects meaningful changes, and saves a frame per step. You can mark moments by hand.
2. **Review** — Reorder, merge, blur sensitive areas, swap frames, edit titles.
3. **Write** — Your chosen model turns screenshots into instructions. Steps you edited yourself are left alone.
4. **Export** — Markdown (+ images folder), self-contained HTML, or print-ready PDF.

Screenshots stay on your machine until you ask for AI writing. API keys live in the OS keychain.

## Requirements

- Rust 1.82+ and [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
- Node 18+ and pnpm
- macOS 11+, Windows 10+, or Linux with screen capture
- An API key from your preferred provider (optional if you only edit by hand)

## Running locally

```bash
pnpm install
pnpm tauri dev
```

Build installers:

```bash
pnpm tauri build
```

## Development

```bash
pnpm typecheck
cargo test --manifest-path src-tauri/Cargo.toml
```

## Layout

```
src/                 Svelte front end
src-tauri/src/       Rust: capture, AI, export, storage
```

Projects are plain folders under the platform app-data directory (`project.json` + PNG frames).

## Licence

MIT. See [LICENSE](LICENSE).
