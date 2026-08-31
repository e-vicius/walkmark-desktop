# Privacy and data

Walkmark is built to keep capture and storage on your machine by default.

## Always local

These never require the network:

| Activity | Stored where |
| --- | --- |
| Screen and window recording | `projects/<id>/frames/*.png` |
| Project metadata and edits | `projects/<id>/project.json` |
| Reorder, merge, exclude, annotations | Same project folder |
| Export to Markdown / HTML / PDF | Path you choose at save time |
| App preferences | `settings.json` in app data |
| API keys (BYOK) | `credentials.<provider>` + OS secret store |

Projects live in your platform app data directory as plain folders. Walkmark does not operate a sync service for open source users.

### App data locations

| OS | Path |
| --- | --- |
| macOS | `~/Library/Application Support/app.walkmark.desktop/` |
| Windows | `%APPDATA%\app.walkmark.desktop\` |
| Linux | `~/.local/share/app.walkmark.desktop/` |

You can back up, encrypt, or sync this folder with tools you trust. Walkmark does not encrypt project folders separately from your disk encryption.

## When data leaves your machine

Data is sent **only when you run Write** with a model that is not purely local:

| Setup | Where screenshots and prompts go |
| --- | --- |
| **Ollama / compatible server on localhost** | Your machine only |
| **Compatible server on LAN** | Your network |
| **Bring your own key** (Gemini, OpenAI, Anthropic, Mistral, OpenRouter) | That provider's API |
| **Walkmark Cloud** (coming soon) | Walkmark-hosted models under Cloud terms |

Walkmark does not upload frames in the background. If you never click Write, or you only use local models, cloud providers never receive your screenshots.

### What is sent on Write

For each write job, the configured provider receives:

- Step screenshots (vision models)
- Prompt text (audience, tone, language, vocabulary, step context)
- No unrelated projects or library metadata

Outline pass sends all included frames; step pass sends one frame per step plus prior step text.

## API keys

Keys you enter for BYOK providers are:

- Verified once against the provider API
- Stored locally in app data and mirrored to the OS keychain / Secret Service when available
- Read only by the Walkmark process to sign requests

**Walkmark never receives or stores your keys on our servers.**

Delete keys by removing them in Settings or deleting `credentials.<provider>` while the app is quit.

## Walkmark Cloud

The paid Cloud plan (separate from this open source app) will include its own privacy terms when it launches. The OSS app remains fully usable without an account.

## Telemetry

The open source app does not include analytics or crash reporting today. If that changes, it will be opt-in and documented here.

## Enterprise

Teams with SSO, audit logs, or on-prem deployment requirements can [contact us](mailto:hello@walkmark.app).

## Related

- [Local models](local-models.md) — keep writing on your hardware
- [AI writing](ai-writing.md) — what prompts contain
