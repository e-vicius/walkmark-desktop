# Settings

Open **Settings** from the gear icon in the top bar, or press **⌘,** (Mac) / **Ctrl+,** (Windows/Linux).

Settings are saved to `settings.json` in your app data folder and apply to new write jobs immediately.

## Model

Pick the default **provider** and **model** for AI writing. The toolbar model picker can override this per document without changing defaults.

### Provider

Seven providers are built in:

| Provider | API key | Runs on |
| --- | --- | --- |
| Google Gemini | Yes | Cloud |
| OpenAI | Yes | Cloud |
| Anthropic | Yes | Cloud |
| Mistral | Yes | Cloud |
| OpenRouter | Yes | Cloud |
| On this Mac (Ollama) | No | Your machine — label is fixed in the app |
| Custom endpoint | Optional | Your machine or LAN |

Configured providers show a **Ready** badge. Cloud providers need a key pasted once; Steppy verifies it before saving. See [AI writing](ai-writing.md) and [Local models](local-models.md).

### Concurrency

**1–8** steps written in parallel (default **3**). Lower this if requests time out or your provider rate-limits you.

Disabled for **Ollama** — local models should write one step at a time.

## Writing

Defaults for new guides. You can change audience, voice, and language before each recording in the source picker as well.

### Who is this for?

Free-text **audience** description passed to the model. Default: *a colleague who has never used this tool before*.

Examples: *new support agents*, *developers onboarding to our API*, *finance staff who use Excel daily*.

### Voice

| Voice | Effect |
| --- | --- |
| **Neutral** | Clear, direct instructions (default) |
| **Friendly** | Slightly warmer tone |
| **Formal** | More structured, less casual |
| **Playful** | Lighter wording (use sparingly for internal docs) |

### Language

Output language for titles and step bodies. Presets include English, German, French, Spanish, and others, or **Other** with a custom language name.

### Products and vocabulary

Group guides by product (e.g. *General*, *Acme Billing*, *Internal admin*). Each product has a **vocabulary** list:

- **Term** — word the model should use (*Workspace*, *Billing portal*)
- **What it is** — short definition so the model does not invent names

Set a **Default product** for new recordings. Vocabulary is injected into every write prompt for guides tagged with that product.

## Recording

Controls how Steppy decides when to capture a screenshot.

### Minimum gap between steps

Cooldown after each automatic step (default **800 ms**). Stops a single form field from generating ten frames while you type.

### Pause after each action

Delay after a click, key press, or scroll before the screenshot is taken (default **300 ms**). Gives menus and dialogs time to open.

### Countdown before starting

Seconds before recording begins (default **3**). Set to **None** (0) to start immediately.

### Screen check interval

How often Steppy compares frames when using visual fallback (default **200 ms**).

### Stored screenshot width

Max width frames are saved at (default **1800 px**). Larger preserves detail; smaller saves disk space.

### Visual fallback

When **on**, Steppy also captures on significant screen changes if input monitoring is unavailable or as a backup. Adjust **Visual sensitivity** (only big changes → every small change) and **Wait for the screen to settle**.

On macOS with Accessibility permission, input-based capture is preferred; visual fallback is mainly for Linux or when input events are blocked.

### Get out of the way while recording

When **on** (default), the main window hides and a floating **recording HUD** appears at the bottom of the screen. Turn off to keep the Steppy window visible.

## Appearance

**Theme:** System (default), Light, or Dark. Matches your OS setting when set to System.
