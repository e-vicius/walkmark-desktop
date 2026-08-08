# AI writing

Steppy uses a **two-pass** approach so guides read as one coherent document, not isolated captions per screenshot.

## Pass 1 — outline

The model sees **every included frame in order** (images + sequence) and produces:

- Overall **title** and **summary**
- **Before you start** prerequisites (only when inferable from the recording)
- A short **imperative title** for each step

This pass runs once per write job. It keeps step names consistent and stops the model from repeating the same action three times.

## Pass 2 — step bodies

Each step is written separately. The model receives:

- The overall task and outline
- The planned title for this step
- Text of **earlier steps** (so instructions do not contradict or repeat)
- **Only this step's screenshot** (keeps requests smaller)

Steps write in parallel up to **Concurrency** (Settings → Model, default 3). Ollama always writes one at a time.

## Picking a model

### Default vs per-document

- **Settings → Model** — default provider and model for new write jobs
- **Toolbar model picker** — override for the open document without changing defaults

The picker lists providers marked **Ready** (API key saved or Ollama running).

### Providers

| Provider | API key | Vision models | Notes |
| --- | --- | --- | --- |
| **Google Gemini** | Yes | Yes | Generous free tier; default Gemini 3.6 Flash |
| **OpenAI** | Yes | Yes | GPT-5.6 family |
| **Anthropic** | Yes | Yes | Strong on dense UI text |
| **Mistral** | Yes | Yes | European-hosted |
| **OpenRouter** | Yes | Yes | One key, many third-party models |
| **On this Mac** | No | Yes (Ollama) | Local downloads — see [Local models](local-models.md) |
| **Custom endpoint** | Optional | Depends on server | LM Studio, vLLM, OpenAI-compatible APIs |

**Vision is required.** Text-only models cannot read screenshots and will fail or hallucinate.

### Model notes in the catalog

Each suggested model in Settings shows a one-line note (speed vs quality). You can also type a **custom model id** if your provider serves a model not listed.

## API keys (bring your own key)

1. Open **Settings → Model**.
2. Select a cloud provider.
3. Follow the in-app **key guide** (link to the provider's console).
4. Paste the key and save — Steppy **verifies** it with a test request before storing.

Keys are saved in your app data folder (`credentials.<provider>`) and mirrored to the OS secret store when available. Steppy never uploads keys to Steppy's servers.

Remove a key by clearing the field and saving, or delete the credentials file while Steppy is quit.

## Write scopes

| Action | When to use |
| --- | --- |
| **Write N steps** | Some steps are still empty |
| **Rewrite** (toolbar) | Regenerate every **unlocked** step |
| **Sparkles** (one step) | Fix a single step after editing the outline |

Progress events update step cards live. Failed steps show the provider error on the card.

## Tone, audience, and language

Defaults in **Settings → Writing** (and the source picker before recording):

| Setting | Effect |
| --- | --- |
| **Who is this for?** | Audience description in every prompt |
| **Voice** | Neutral, Friendly, Formal, or Playful |
| **Language** | Output language for titles and bodies |

**Products and vocabulary** — terms and definitions grouped by product. See [Settings](settings.md).

## Concurrency and timeouts

**Settings → Model → Concurrency** (1–8, default 3): how many step requests run at once.

- Lower if you hit rate limits or timeouts
- Disabled for Ollama — local hardware writes sequentially

## Steppy Cloud

**Steppy Cloud** (coming soon) runs models for you with no API key — a separate opt-in plan. The open source app remains fully usable with BYOK or local models.

## Tips for better output

- Record the happy path in order; skip unrelated detours.
- **Blur or cover** secrets before writing.
- **Lock** polished steps before a full rewrite.
- Add **vocabulary** entries for product names the model might guess wrong.
- Pick a **vision** model sized for your UI complexity — dense dashboards benefit from larger models.
- If step titles are wrong, run **Rewrite** after fixing the outline manually, or rewrite individual steps.

## Privacy

Screenshots and prompts are sent **only when you click Write**, and only to the provider or endpoint you configured. See [Privacy and data](privacy.md).
