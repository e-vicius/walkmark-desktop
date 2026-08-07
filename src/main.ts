import { mount } from "svelte";

import App from "./App.svelte";
import { isHudWindow, isMacOS } from "./lib/window";
import "./app.css";

if (isHudWindow()) {
  document.documentElement.dataset.window = "hud";
}

if (isMacOS()) {
  document.documentElement.dataset.platform = "macos";
}

mount(App, { target: document.getElementById("root")! });
