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

document.addEventListener("contextmenu", (event) => {
  event.preventDefault();
});

mount(App, { target: document.getElementById("root")! });
