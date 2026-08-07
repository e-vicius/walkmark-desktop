import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import * as api from "./api";
import { windowLabel } from "./window";
import { store } from "./store.svelte";
import type {
  AppError,
  GenerationProgress,
  PullProgress,
  RecordingTick,
  Step,
} from "./types";

/** Wires backend events into the store. Returns a disposer. */
export async function connectEvents(): Promise<UnlistenFn> {
  const isMain = windowLabel() === "main";
  const unlisteners: UnlistenFn[] = [];

  unlisteners.push(
    await listen<RecordingTick>("recording:tick", ({ payload }) => {
      store.recording = payload.state === "idle" ? null : payload;
    }),
  );

  if (isMain) {
    unlisteners.push(
      await listen<Step>("recording:step", async ({ payload }) => {
        const project = store.project;
        if (!project || project.steps.some((s) => s.id === payload.id)) return;
        store.project = { ...project, steps: [...project.steps, payload] };
        try {
          await api.appendStep(payload);
        } catch {
          /* recovered on stop */
        }
      }),
    );

    unlisteners.push(
      await listen<{ stepId: string; frame: string }>(
        "recording:alternate",
        async ({ payload }) => {
          const project = store.project;
          if (!project) return;
          store.project = {
            ...project,
            steps: project.steps.map((s) =>
              s.id === payload.stepId &&
              !s.alternates.includes(payload.frame) &&
              s.frame !== payload.frame
                ? { ...s, alternates: [...s.alternates, payload.frame] }
                : s,
            ),
          };
          try {
            await api.attachAlternate(payload.stepId, payload.frame);
          } catch {
            /* non-fatal */
          }
        },
      ),
    );

    unlisteners.push(
      await listen<{ message: string; fatal: boolean }>(
        "recording:error",
        ({ payload }) => {
          store.notify({
            tone: payload.fatal ? "error" : "info",
            title: payload.fatal ? "Recording stopped" : "Recording hiccup",
            detail: payload.message,
          });
          if (payload.fatal) store.recording = null;
        },
      ),
    );

    unlisteners.push(
      await listen<GenerationProgress>("ai:progress", ({ payload }) => {
        store.generation = payload.running ? payload : null;
      }),
    );

    unlisteners.push(
      await listen<Step>("ai:step", ({ payload }) => {
        const project = store.project;
        if (!project) return;
        store.project = {
          ...project,
          steps: project.steps.map((s) => (s.id === payload.id ? payload : s)),
        };
      }),
    );

    unlisteners.push(
      await listen<{ title: string; summary: string; prerequisites: string[] }>(
        "ai:outline",
        ({ payload }) => {
          const project = store.project;
          if (!project) return;
          store.project = { ...project, ...payload };
        },
      ),
    );

    unlisteners.push(
      await listen<{ cancelled: boolean; succeeded: number; failed: number }>(
        "ai:done",
        ({ payload }) => {
          store.generation = null;
          void store.refreshProjects();
          if (payload.cancelled) {
            store.notify({ tone: "info", title: "Writing stopped" });
          } else if (payload.failed > 0) {
            store.notify({
              tone: "error",
              title: `${payload.failed} of ${payload.failed + payload.succeeded} steps failed`,
              detail: "Open a failed step to see why, then retry just that one.",
            });
          } else if (payload.succeeded > 0) {
            store.notify({ tone: "success", title: `Wrote ${payload.succeeded} steps` });
          }
        },
      ),
    );

    unlisteners.push(
      await listen<AppError>("ai:error", ({ payload }) => {
        store.generation = null;
        store.reportError(payload, "Writing failed");
      }),
    );

    unlisteners.push(
      await listen<PullProgress>("local:pull", ({ payload }) => {
        store.pull = payload;
        if (!payload.done) return;
        if (payload.error) {
          store.notify({
            tone: "error",
            title: `${payload.model} could not be downloaded`,
            detail: payload.error,
          });
        } else {
          store.notify({
            tone: "success",
            title: `${payload.model} is ready`,
            detail: "It can now write your documentation without a network connection.",
          });
        }
        void store.refreshLocal();
      }),
    );

    unlisteners.push(
      await listen<AppError>("local:error", ({ payload }) => {
        store.pull = null;
        store.reportError(payload, "The download failed");
      }),
    );
  }

  unlisteners.push(
    await listen<string>("recording:shortcut", ({ payload }) => {
      if (!isMain) return;
      if (payload === "stop") void store.endRecording();
      if (payload === "mark") void store.markStep();
      if (payload === "pause") void store.togglePause();
    }),
  );

  return () => unlisteners.forEach((off) => off());
}
