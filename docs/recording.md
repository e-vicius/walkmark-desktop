# Recording

Recording is where Steppy watches a screen or window, captures PNG frames at meaningful moments, and builds the step rail for your guide.

## Starting a recording

**New guide** opens the **source picker**:

1. **Writing defaults** — audience, voice, language, product (from [Settings → Writing](settings.md); change per recording here).
2. **Capture source** — every connected monitor and open window, with live thumbnail and resolution.
3. **Record** — starts the countdown and session.

While recording, the main window can hide and a **floating HUD** appears at the bottom of the screen (see [HUD](#recording-hud)).

## Picking a source

Pick the **smallest area** that still shows the whole task:

| Source | Best for |
| --- | --- |
| **Window** | Single-app flows; less desktop clutter in frames |
| **Monitor** | Multi-window workflows, drag across apps, or full-desktop tasks |

The thumbnail updates live — confirm you selected the right target before clicking Record.

## How steps are captured

Steppy uses two mechanisms:

### Input-based capture (preferred on macOS)

When **Accessibility** permission is granted, Steppy listens for clicks, key presses, and scrolls. After each action it waits **Pause after each action** (default 300 ms), then grabs a frame.

This produces frames aligned with what you did, not arbitrary animation mid-flight.

### Visual fallback (optional)

When **Visual fallback** is on in **Settings → Recording**, Steppy also compares periodic screenshots. If the image changes enough — and **Minimum gap between steps** has passed — it saves a step even without a fresh input event.

Useful when:

- Input monitoring is blocked (some Linux setups)
- The UI changes on its own (loading spinners, auto-refresh)
- You need a frame from a slow animation you did not click

Tune **Visual sensitivity** and **Wait for the screen to settle** when visual fallback is enabled.

### Minimum gap

A cooldown (default **800 ms**) between automatic steps. Prevents key repeat, double-clicks, or rapid typing from flooding the guide with near-duplicate frames.

## Recording HUD

When **Get out of the way while recording** is on (default), Steppy minimizes and shows a compact HUD:

| Control | Shortcut (global) | Action |
| --- | --- | --- |
| **Stop** | **Shift + Alt + S** | End recording and open the editor |
| **Mark step** | **Shift + Alt + M** | Force a capture now |
| **Pause / Resume** | **Shift + Alt + P** | Pause capture (timestamps exclude paused time) |

The HUD shows elapsed time, step count, and a simple activity indicator.

Global shortcuts work while Steppy is in the background — switch to your target app and keep working.

## Manual steps

Press **Mark step** (or **Shift + Alt + M**) to force a capture at any moment:

- A visual change is too subtle for auto-detection
- You want a frame before and after the same click
- A slow animation needs the right frame, not the first diff

Manual marks respect the minimum gap like automatic steps.

## Alternate frames

When Steppy captures near the same moment, it may store **alternate** frames for a step. In the editor, open **Different moment** on the screenshot to pick the best frame. See [Editing guides — Pick a different moment](editing.md#pick-a-different-moment).

## Pause and resume

Pause when you need to answer a message or dig through files off-screen. Resume when you are back in the workflow. No steps are captured while paused.

## Recording settings

All tunables live in **Settings → Recording**:

| Setting | Default | What it does |
| --- | --- | --- |
| **Minimum gap between steps** | 0.8 s | Cooldown between automatic captures |
| **Pause after each action** | 0.3 s | Wait after input before screenshot |
| **Countdown before starting** | 3 s | Delay before capture begins (0 = none) |
| **Screen check interval** | 0.2 s | How often to sample after an action |
| **Stored screenshot width** | 1800 px | Max width saved to disk |
| **Visual fallback** | Off | Also capture on screen change |
| **Visual sensitivity** | — | How large a change must be (when fallback on) |
| **Wait for the screen to settle** | — | Avoid half-open menus (when fallback on) |
| **Get out of the way while recording** | On | HUD instead of main window |

Wider screenshots preserve UI detail for AI and export; narrower saves disk space.

## After recording

Steps appear in **capture order**. Before writing or exporting you can:

- Reorder, merge, or delete steps
- Exclude steps from the document
- Annotate screenshots (blur, cover, highlight)
- Pick alternate frames

See [Editing guides](editing.md).

## Platform notes

### macOS

Both **Screen Recording** and **Accessibility** are required for the best experience. Without Accessibility, enable **Visual fallback** or rely on manual marks.

### Windows

Window and monitor capture via the OS APIs. ARM64 and x64 builds behave the same.

### Linux

Wayland support varies by compositor. If window capture fails, try a monitor source. For headless or remote sessions, capture may not be available — Steppy needs a local graphical session.
