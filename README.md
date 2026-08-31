<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" alt="Walkmark logo" width="128" />
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="MIT License" /></a>
</p>

# Walkmark

Turn a workflow you do once into step-by-step documentation someone else can follow.

Walkmark is a native desktop app. You pick a screen or window, do the task, and it captures each meaningful step with a screenshot. When you are ready, an AI model writes the instructions. Export as Markdown, a single HTML file, or PDF.

<p align="center">
  <img src="docs/screenshots/readme-editor.png" alt="Walkmark document editor with AI-written steps and export" width="900" />
</p>

**Website:** [walkmark.app](https://walkmark.app) · **Docs:** [walkmark.app/docs](https://walkmark.app/docs)  
**Download:** [Latest release](https://github.com/e-vicius/walkmark-desktop/releases/latest)  
**Marketing site source:** [walkmark-landing](https://github.com/e-vicius/walkmark-landing)

## Download

| Platform | Install |
| --- | --- |
| **macOS** (Apple Silicon) | [`.dmg` aarch64](https://github.com/e-vicius/walkmark-desktop/releases/latest/download/Walkmark_0.1.1_aarch64.dmg) |
| **macOS** (Intel) | [`.dmg` x64](https://github.com/e-vicius/walkmark-desktop/releases/latest/download/Walkmark_0.1.1_x64.dmg) |
| **Windows** (x64) | [`.exe` installer](https://github.com/e-vicius/walkmark-desktop/releases/latest/download/Walkmark_0.1.1_x64-setup.exe) |
| **Windows** (ARM64) | [`.exe` installer](https://github.com/e-vicius/walkmark-desktop/releases/latest/download/Walkmark_0.1.1_arm64-setup.exe) |
| **Linux** | [AppImage](https://github.com/e-vicius/walkmark-desktop/releases/latest/download/Walkmark_0.1.1_amd64.AppImage) · [`.deb`](https://github.com/e-vicius/walkmark-desktop/releases/latest/download/walkmark_0.1.1_amd64.deb) |

Asset names match tag `v0.1.1`. See [all releases](https://github.com/e-vicius/walkmark-desktop/releases) if a newer version is available.

Requires macOS 11+, Windows 10+, or a recent Linux desktop with screen capture.

## Open source vs Cloud

| | **Open source (this repo)** | **Walkmark Cloud** |
| --- | --- | --- |
| Status | Available now on [GitHub](https://github.com/e-vicius/walkmark-desktop) | [Waitlist](https://walkmark.app/#cloud) — not open yet |
| Price | Free, MIT licence — no subscription | Pricing at launch |
| AI writing | Ollama or LM Studio on your machine, or bring your own API keys | Hosted models in the same desktop app — no API keys |
| Your data | Recording, editing, and export stay on your computer | Sharable guide links when Cloud launches |
| Account | None required | Sign in when Cloud launches |
| Enterprise | — | SSO, audit logs, on-prem — [hello@walkmark.app](mailto:hello@walkmark.app?subject=Walkmark%20Enterprise) |

The open source app is fully usable without Cloud: projects and screenshots stay on disk, recording and export work offline, and your keys never pass through our servers when you write with a local model or your own provider.

## How it works

1. **Record** — Walkmark samples the screen, detects meaningful changes, and saves a frame per step. You can mark moments by hand.
2. **Review** — Reorder, merge, blur sensitive areas, swap frames, edit titles.
3. **Write** — Your chosen model turns screenshots into instructions. Steps you edited yourself are left alone. [How AI writing works →](docs/ai-writing.md)
4. **Export** — Markdown (+ images folder), self-contained HTML, or print-ready PDF.

## Privacy

Recording, editing, and export stay on your machine. Cloud is opt-in only: local models keep writing on your hardware, or you connect Walkmark Cloud or your own API keys. [Full privacy guide →](docs/privacy.md)

## Documentation

Full guides at [walkmark.app/docs](https://walkmark.app/docs) or in [`docs/`](docs/README.md).

**Using Walkmark:** [Getting started](docs/getting-started.md) · [Recording](docs/recording.md) · [Editing](docs/editing.md) · [Settings](docs/settings.md) · [AI writing](docs/ai-writing.md) · [Local models](docs/local-models.md) · [Export](docs/export.md) · [Shortcuts](docs/keyboard-shortcuts.md) · [Troubleshooting](docs/troubleshooting.md) · [Privacy](docs/privacy.md)

**Developers:** [Build from source](docs/build-from-source.md) · [Architecture](docs/architecture.md) · [Contributing](CONTRIBUTING.md)

## Build from source

```bash
git clone https://github.com/e-vicius/walkmark-desktop.git
cd walkmark
pnpm install
pnpm tauri dev      # development
pnpm tauri build    # installers in src-tauri/target/release/bundle/
```

See [docs/build-from-source.md](docs/build-from-source.md) and [CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

MIT. See [LICENSE](LICENSE).
