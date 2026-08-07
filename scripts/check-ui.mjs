import { chromium } from "playwright";

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();

const errors = [];
page.on("pageerror", (err) => errors.push(String(err)));
page.on("console", (msg) => {
  if (msg.type() === "error") errors.push(msg.text());
});

await page.addInitScript(() => {
  const catalog = {
    providers: [
      {
        id: "gemini",
        name: "Google Gemini",
        blurb: "Generous free tier.",
        local: false,
        needsKey: true,
        keyUrl: "https://aistudio.google.com/app/apikey",
        keyHint: "Starts with AIza",
        keyGuide: [
          "Open Google AI Studio and sign in.",
          "Click Create API key.",
          "Copy the key and paste it in Steppy.",
        ],
        baseUrlEditable: false,
        baseUrlRequired: false,
        defaultBaseUrl: "",
        defaultModel: "gemini-3.6-flash",
        models: [
          {
            id: "gemini-3.6-flash",
            name: "Gemini 3.6 Flash",
            note: "Fast default.",
            tier: "recommended",
          },
        ],
      },
      {
        id: "ollama",
        name: "On this Mac",
        blurb: "Runs offline.",
        local: true,
        needsKey: false,
        keyUrl: "https://ollama.com/download",
        keyHint: "",
        keyGuide: [],
        baseUrlEditable: true,
        baseUrlRequired: false,
        defaultBaseUrl: "http://127.0.0.1:11434",
        defaultModel: "qwen3-vl:8b",
        models: [],
      },
    ],
    configured: [],
  };

  const settings = {
    provider: "gemini",
    providers: {},
    audience: "a colleague",
    tone: "neutral",
    language: "English",
    concurrency: 3,
    capture: {
      sampleIntervalMs: 600,
      sensitivity: 0.55,
      minGapMs: 1200,
      settle: true,
      maxWidth: 1800,
      countdownSecs: 3,
      hideWindow: true,
    },
    theme: "light",
    onboarded: true,
    products: [{ id: "general", name: "General", vocabulary: [] }],
    defaultProductId: "general",
  };

  const local = {
    running: true,
    version: "0.6.2",
    endpoint: "http://127.0.0.1:11434",
    downloadUrl: "https://ollama.com/download",
    totalMemory: 16_000_000_000,
    downloading: false,
    models: [],
    catalog: [
      {
        id: "qwen3-vl:8b",
        name: "Qwen3-VL 8B",
        note: "Recommended.",
        size: 6_100_000_000,
        minMemory: 16_000_000_000,
        recommended: true,
        installed: false,
        fits: true,
      },
    ],
  };

  function mockInvoke(cmd, payload = {}) {
    switch (cmd) {
      case "plugin:event|listen":
        return 1;
      case "plugin:event|unlisten":
        return null;
      case "get_settings":
        return settings;
      case "save_settings":
        return payload.settings;
      case "permission_status":
        return { granted: true, required: false };
      case "provider_catalog":
        return catalog;
      case "list_projects":
        return [];
      case "current_project":
        return null;
      case "recording_status":
        return { active: false, state: "idle", stepCount: 0 };
      case "local_status":
        return local;
      default:
        return null;
    }
  }

  let nextId = 1;
  window.__TAURI_INTERNALS__ = {
    invoke(cmd, args) {
      return Promise.resolve(mockInvoke(cmd, args ?? {}));
    },
    transformCallback() {
      return nextId++;
    },
    unregisterCallback() {},
    convertFileSrc(path) {
      return `asset://${path}`;
    },
    metadata: { currentWindow: { label: "main" } },
  };

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    registerListener() {},
    unregisterListener() {},
  };
});

await page.goto("http://localhost:1420/", { waitUntil: "networkidle" });
await page.waitForTimeout(3000);

const bootText = await page.locator("body").innerText();
const bootHtml = await page.locator("#root").innerHTML();

await page.keyboard.press("Meta+Comma");
await page.waitForTimeout(800);

// Exercise the model settings panel — the most likely crash site.
await page.getByRole("tab", { name: "Model" }).click();
await page.waitForTimeout(300);
await page.getByRole("button", { name: "On this Mac" }).click();
await page.waitForTimeout(1500);

const dialog = await page.locator('[role="dialog"]').count();
const bodyText = await page.locator("body").innerText();

console.log(
  JSON.stringify(
    {
      dialog,
      bootPreview: bootText.slice(0, 300),
      rootLen: bootHtml.length,
      bodyPreview: bodyText.slice(0, 300),
      errors,
    },
    null,
    2,
  ),
);
await page.screenshot({ path: "/tmp/steppy-ui-check.png", fullPage: true });
await browser.close();
