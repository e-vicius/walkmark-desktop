# Troubleshooting

## macOS permissions

Steppy needs two permissions on macOS:

### Screen Recording

Required to capture screenshots. If denied:

1. Open **System Settings → Privacy & Security → Screen Recording**
2. Enable **Steppy**
3. Quit and reopen Steppy if it was already running

Use **Grant access** or **System Settings** from the onboarding banner or library.

### Accessibility

Required to detect clicks, typing, and scrolling for timely step capture. Without it, Steppy falls back to visual-only detection, which is less reliable.

1. Open **System Settings → Privacy & Security → Accessibility**
2. Enable **Steppy**

## Recording issues

### No steps captured

- Interact with the target app — Steppy captures on input or screen change, not on idle screens.
- Press **Shift + Alt + M** (or **Capture this moment** in the HUD) to force a step.
- Lower **Minimum gap** in Settings → Recording if steps feel too sparse.
- Increase **Pause after each action** if frames fire before dialogs open.

### Steps fire too often

- Raise **Minimum gap between steps**.
- Turn off **Visual fallback** if you have Accessibility permission on macOS.

### Wrong window captured

- Stop and start a new recording with the correct **window** source (not full monitor) if the task stays in one app.

### Floating HUD did not appear

- Check **Get out of the way while recording** in Settings → Recording.
- A non-fatal toast may explain if the HUD window could not be created.

## AI writing issues

### "Select an AI model"

Pick a provider in **Settings → Model**, choose a **vision** model, and save an API key (cloud) or start Ollama (local).

### "That key didn't work"

The provider rejected the key. Check the key is complete, not expired, and has API access enabled on the provider's dashboard.

### Steps fail with an error

Open the failed step card — the red banner shows the provider message. Common causes:

- Rate limits — lower **Concurrency** or wait and **Retry**
- Model cannot read images — pick a vision-capable model
- Context too large — fewer steps per guide or smaller screenshots

### Locked steps not rewriting

Steps you edited by hand are **locked** so a full rewrite does not overwrite your fixes. Unlock by editing again, or use the sparkles icon on a single step to rewrite just that one.

### Writing repeats itself

Run **Rewrite everything** after fixing outline issues, or rewrite individual steps. The two-pass outline should prevent most repetition; report persistent cases on GitHub.

## Local models (Ollama)

### Ollama not running

Install from [ollama.com](https://ollama.com/download) and ensure the menu bar / background service is active. **Settings → Model → On this Mac** shows connection status.

### Model missing

Pull a **vision** model from the catalog in Settings. Text-only models cannot read screenshots.

### Out of memory

Pick a smaller model from the catalog; entries show whether your machine has enough RAM.

## Export issues

### Export button disabled

Include at least one step (eye icon open) in the document.

### Unwritten steps in export

Steps without body text export with title placeholders and screenshots. Amber warning in the export dialog lists how many are incomplete.

### PDF looks wrong

PDF uses print layout (A4). For web-style reading, use **HTML** export.

## Data and backups

Projects live in plain folders under app data. To back up or move machines, copy the entire `projects/` directory and `settings.json`.

See [Getting started — Where projects live](getting-started.md#where-projects-live) for paths per OS.

## Still stuck?

[Open an issue on GitHub](https://github.com/e-vicius/steppy/issues) with your OS version, Steppy version, and what you expected vs what happened. Screenshots of error toasts help.
