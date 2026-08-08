# Local models

Run AI writing entirely on your hardware. No API key, no data sent to Steppy's servers or third-party cloud APIs.

Recording, editing, and export stay local either way. Only the **Write** action sends screenshots to a model — with local models that endpoint is on your machine or LAN.

## Ollama

1. Install [Ollama](https://ollama.com/download) and ensure the background service is running.
2. In Steppy, open **Settings → Model** and select **On this Mac** (Ollama provider).
3. Download a **vision** model from the catalog (vision badge). Text-only models cannot read screenshots.
4. Pick the model in the document toolbar and write as usual.

Steppy talks to Ollama at `http://127.0.0.1:11434` by default.

### Catalog and downloads

The in-app catalog lists recommended vision models with size and RAM hints. Click **Download** to pull via Ollama; progress shows in Settings.

You can also enter a **custom model id** (e.g. `my-model:tag`) and pull or select it.

### Recommended models

Pick based on available RAM:

| Model (examples) | Tradeoff |
| --- | --- |
| Moondream 2B | Smallest; terse output |
| Qwen3-VL 4B / 8B | Strong text reading for size |
| Gemma 3 / 4 | Good non-English UI |
| Llama 3.2 Vision 11B | Dense screen text |

The catalog marks whether your machine likely has enough memory.

### Concurrency

Ollama writes **one step at a time** regardless of the Concurrency slider — local models are slower and have less context headroom.

## LM Studio and compatible servers

Any **OpenAI-compatible** HTTP API works:

1. Start LM Studio (or vLLM, llama.cpp server, etc.) and enable the local server.
2. In Steppy **Settings → Model**, choose **Custom endpoint**.
3. Set the **base URL** (e.g. `http://127.0.0.1:1234/v1`) and a vision-capable **model id**.
4. API key is optional — leave blank if the server does not require one.

Steppy sends chat-completions-style requests with image attachments the same way it does for cloud vision models.

## Troubleshooting local models

| Issue | Fix |
| --- | --- |
| "Looking for Ollama…" | Install and start Ollama |
| Model missing | Download from catalog or `ollama pull <name>` |
| Out of memory | Pick a smaller catalog entry |
| Garbled or empty steps | Confirm the model supports vision; try Qwen3-VL or larger |

See [Troubleshooting](troubleshooting.md).

## Local vs cloud

| | Local (Ollama / compatible) | Cloud (BYOK or Steppy Cloud) |
| --- | --- | --- |
| Data leaves your machine | No (LAN only if remote server) | Yes, when writing |
| Setup | Install runtime + model | API key or Cloud account |
| Speed | GPU/CPU dependent | Usually faster on frontier models |
| Cost | Free after hardware | Provider usage or subscription |
| Offline | Yes, once model is downloaded | No |

## Privacy

Local writing never sends frames to the public internet unless your custom endpoint is on a remote host you control. See [Privacy and data](privacy.md).
