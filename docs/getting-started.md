# Getting started

Steppy turns a workflow you perform once into step-by-step documentation someone else can follow. This guide covers install, your first recording, permissions, the library, and writing instructions.

## Install

Download the latest release for your platform from [GitHub Releases](https://github.com/e-vicius/steppy-desktop/releases/latest):

| Platform | File | Notes |
| --- | --- | --- |
| **macOS** | `Steppy_*_aarch64.dmg` or `*_x64.dmg` | Apple Silicon or Intel |
| **Windows** | `*_x64-setup.exe` or `*_arm64-setup.exe` | x64 or native ARM64 (Surface, Snapdragon PCs) |
| **Linux** | `*_amd64.AppImage` or `*.deb` | AppImage is portable; `.deb` for Debian/Ubuntu |

You can also download from [steppy.app](https://steppy.app/#download) — the site detects your OS and recommends the right build.

### Build from source

See [Build from source](build-from-source.md) if you prefer to compile locally or contribute to the repo.

## First launch

On first open, Steppy shows **onboarding**:

1. **Permissions** (macOS) — Screen Recording and Accessibility.
2. **Pick a model** — choose a cloud provider and paste an API key, or install Ollama for local models.

You can skip onboarding and configure later in **Settings → Model**. Recording works without AI; writing needs a vision-capable model.

## The library

The **library** is your home screen. Each card is a saved guide with a thumbnail, title, step count, and last-edited time.

| Action | How |
| --- | --- |
| Open a guide | Click its card |
| New recording | **New guide** (top bar) or **⌘R** / **Ctrl+R** |
| Delete a guide | Card menu → **Delete** — removes the project folder from disk |

An empty library shows a single **Record your first guide** button.

## First recording

1. Click **New guide**.
2. In the source picker, choose **audience**, **voice**, **language**, and **product** (optional — defaults come from [Settings → Writing](settings.md)).
3. Pick a **screen or window**. Steppy only captures that area.
4. Click **Record**. A countdown runs (default 3 seconds) so you can switch to the target app.
5. Do the task normally. Steppy saves a screenshot when it detects a meaningful change, or when you mark a step manually.
6. Click **Stop** in the HUD or press **Shift + Alt + S**.

You land in the **editor**: step rail on the left, document on the right.

See [Recording](recording.md) for HUD controls, pause, and tuning capture sensitivity.

## Permissions

### macOS

Steppy needs two permissions:

| Permission | Why |
| --- | --- |
| **Screen Recording** | Capture screenshots of the area you selected |
| **Accessibility** | Detect clicks, typing, and scrolling so steps fire at the right moment |

Grant both in **System Settings → Privacy & Security**. If Steppy was denied earlier, remove it from the list and try again so macOS shows the prompt.

The library banner links directly to the right pane when permission is missing.

### Windows

Screen capture is granted when you pick a source in the recorder. No separate Accessibility step.

### Linux

Depends on your desktop environment. **X11** generally works out of the box. **Wayland** may require a portal-compatible compositor; if capture fails, try a full-monitor source or check your distro's screen-sharing settings.

## Write instructions

1. Open the guide in the editor.
2. Pick a **model** in the toolbar (overrides the default from Settings for this document).
3. Click **Write N steps** (only unwritten steps) or **Rewrite** (all unlocked steps).
4. Steppy runs a **two-pass** write: outline from all frames, then each step body in order. Progress appears on step cards.
5. Edit anything that reads wrong, then **Export** (**⌘E** / **Ctrl+E**).

You can skip AI entirely and type titles and instructions yourself.

See [AI writing](ai-writing.md) for providers, models, and tips.

## Where projects live

Everything stays on disk under your platform app data folder:

| OS | Path |
| --- | --- |
| **macOS** | `~/Library/Application Support/app.steppy.desktop/` |
| **Windows** | `%APPDATA%\app.steppy.desktop\` |
| **Linux** | `~/.local/share/app.steppy.desktop/` (or `$XDG_DATA_HOME`) |

Each guide is a folder:

```
projects/<uuid>/
  project.json    metadata, steps, annotations
  frames/         PNG screenshots
```

Copy the whole `projects/` directory (and `settings.json` if you want preferences) to back up or move machines. Steppy does not sync this for you.

API keys live in `credentials.<provider>` files in the same app data folder, mirrored to the OS keychain when available.

## Next steps

- [Recording](recording.md) — sources, HUD, manual marks, preferences
- [Editing guides](editing.md) — reorder, merge, annotations, locked steps
- [Settings](settings.md) — models, writing voice, recording tuning
- [Keyboard shortcuts](keyboard-shortcuts.md)
- [Troubleshooting](troubleshooting.md)
