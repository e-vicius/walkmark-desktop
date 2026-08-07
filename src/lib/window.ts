import { getCurrentWindow } from "@tauri-apps/api/window";

/** Safe during Vite-only previews where Tauri hasn't injected internals yet. */
export function windowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return "main";
  }
}

export function isHudWindow(): boolean {
  return windowLabel() === "hud";
}

/** macOS overlay title bar needs a fixed inset for the traffic-light buttons. */
export function isMacOS(): boolean {
  if (typeof navigator === "undefined") return false;
  return /Mac/i.test(navigator.userAgent) || /Mac/i.test(navigator.platform);
}
