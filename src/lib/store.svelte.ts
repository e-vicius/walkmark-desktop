import { toast } from "svelte-sonner";
import { setMode } from "mode-watcher";

import * as api from "./api";
import { newProduct } from "./products";
import type {
  AppError,
  GenerationProgress,
  LocalOverview,
  PermissionStatus,
  Project,
  ProjectSummary,
  ProviderCatalog,
  ProviderConfig,
  ProviderId,
  ProviderInfo,
  PullProgress,
  RecordingTick,
  Settings,
} from "./types";

export type Route = "library" | "editor";
export type DialogName = "settings" | "export" | "sources" | "onboarding" | null;

const defaultProduct = newProduct("General");

const DEFAULT_SETTINGS: Settings = {
  provider: "gemini",
  providers: {},
  audience: "a colleague who has never used this tool before",
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
  theme: "system",
  onboarded: false,
  products: [defaultProduct],
  defaultProductId: defaultProduct.id,
};

class AppStore {
  ready = $state(false);
  settings = $state<Settings>({ ...DEFAULT_SETTINGS });
  permission = $state<PermissionStatus>({ granted: true, required: false });
  catalog = $state<ProviderCatalog>({ providers: [], configured: [] });
  local = $state<LocalOverview | null>(null);
  pull = $state<PullProgress | null>(null);
  projects = $state<ProjectSummary[]>([]);
  project = $state<Project | null>(null);
  selection = $state<string[]>([]);
  focused = $state<string | null>(null);
  recording = $state<RecordingTick | null>(null);
  generation = $state<GenerationProgress | null>(null);
  route = $state<Route>("library");
  dialog = $state<DialogName>(null);
  /** Which settings sidebar tab to open. */
  settingsTab = $state("model");
  /** Return here after closing settings, when opened from another dialog. */
  dialogReturn = $state<DialogName | null>(null);

  get activeProvider(): ProviderInfo | undefined {
    return this.catalog.providers.find((p) => p.id === this.settings.provider);
  }

  get activeConfig(): ProviderConfig {
    const info = this.activeProvider;
    const saved = this.settings.providers[this.settings.provider];
    return {
      model: saved?.model?.trim() || info?.defaultModel || "",
      baseUrl: saved?.baseUrl?.trim() || info?.defaultBaseUrl || "",
    };
  }

  get canWrite(): boolean {
    const info = this.activeProvider;
    if (!info) return true;
    if (info.needsKey) return this.catalog.configured.includes(info.id);
    if (info.local && this.local) {
      return this.local.running && this.local.models.some((m) => m.vision);
    }
    return true;
  }

  get blockedReason(): string | null {
    if (this.canWrite) return null;
    const info = this.activeProvider;
    if (!info) return null;
    if (info.needsKey) return `Add your ${info.name} API key to start writing.`;
    if (!this.local?.running) {
      return "Ollama isn't running, so local models can't be used yet.";
    }
    return "No local model that can read screenshots is downloaded yet.";
  }

  async boot() {
    try {
      const [settings, permission, catalog, projects, project, status] =
        await Promise.all([
          api.getSettings(),
          api.permissionStatus(),
          api.providerCatalog(),
          api.listProjects(),
          api.currentProject(),
          api.recordingStatus(),
        ]);

      this.settings = settings;
      this.permission = permission;
      this.catalog = catalog;
      this.projects = projects;
      this.project = project;
      this.route = project ? "editor" : "library";
      this.focused = project?.steps[0]?.id ?? null;
      this.recording = status.active
        ? {
            state: status.state,
            elapsedMs: 0,
            stepCount: status.stepCount,
            activity: 0,
            countdown: 0,
          }
        : null;
      this.dialog = settings.onboarded ? null : "onboarding";
      this.ready = true;
      this.applyTheme();
    } catch (error) {
      this.ready = true;
      this.reportError(error, "Steppy could not start up cleanly");
    }
  }

  applyTheme() {
    setMode(this.settings.theme);
  }

  notify(opts: {
    tone: "info" | "success" | "error";
    title: string;
    detail?: string;
    action?: { label: string; run: () => void };
  }) {
    const common = {
      description: opts.detail,
      action: opts.action
        ? { label: opts.action.label, onClick: opts.action.run }
        : undefined,
    };
    if (opts.tone === "error") toast.error(opts.title, common);
    else if (opts.tone === "success") toast.success(opts.title, common);
    else toast(opts.title, common);
  }

  reportError(error: unknown, title?: string) {
    const appError = api.toAppError(error) as AppError;
    if (appError.kind === "cancelled") return;
    this.notify({
      tone: "error",
      title: title ?? "Something went wrong",
      detail: appError.message,
      action:
        appError.kind === "missing_api_key"
          ? { label: "Add key", run: () => this.setDialog("settings") }
          : undefined,
    });
  }

  setDialog(dialog: DialogName) {
    this.dialog = dialog;
    if (dialog !== "settings") this.dialogReturn = null;
  }

  openSettings(tab = "model", returnTo: DialogName | null = null) {
    this.settingsTab = tab;
    this.dialogReturn = returnTo;
    this.dialog = "settings";
  }

  closeDialog() {
    if (this.dialogReturn) {
      this.dialog = this.dialogReturn;
      this.dialogReturn = null;
    } else {
      this.dialog = null;
    }
  }

  setRoute(route: Route) {
    this.route = route;
  }

  async updateSettings(patch: Partial<Settings>) {
    const next = { ...this.settings, ...patch };
    this.settings = next;
    try {
      this.settings = await api.saveSettings(next);
      this.applyTheme();
    } catch (error) {
      this.reportError(error, "Your settings could not be saved");
    }
  }

  async configureProvider(provider: ProviderId, patch: Partial<ProviderConfig>) {
    await this.updateSettings({
      providers: {
        ...this.settings.providers,
        [provider]: { ...this.settings.providers[provider], ...patch },
      },
    });
  }

  async refreshCredentials() {
    this.catalog = await api.providerCatalog();
  }

  async refreshLocal() {
    try {
      const local = await api.localStatus();
      this.local = local;
      if (!local.downloading && this.pull?.done !== false) {
        this.pull = null;
      }
    } catch (error) {
      this.reportError(error, "Could not check for local models");
    }
  }

  async pullModel(model: string) {
    try {
      this.pull = {
        model,
        status: "starting",
        completed: 0,
        total: 0,
        done: false,
        error: null,
      };
      await api.downloadModel(model);
    } catch (error) {
      this.pull = null;
      this.reportError(error, "The download could not start");
    }
  }

  async cancelPull() {
    await api.cancelDownload();
    this.pull = null;
    await this.refreshLocal();
  }

  async deleteModel(model: string) {
    try {
      await api.removeModel(model);
      this.notify({ tone: "info", title: `Removed ${model}` });
      await this.refreshLocal();
    } catch (error) {
      this.reportError(error, "That model could not be removed");
    }
  }

  async refreshPermission() {
    this.permission = await api.permissionStatus();
  }

  async refreshProjects() {
    try {
      this.projects = await api.listProjects();
    } catch (error) {
      this.reportError(error);
    }
  }

  async openProject(id: string) {
    try {
      const project = await api.openProject(id);
      this.project = project;
      this.route = "editor";
      this.focused = project.steps[0]?.id ?? null;
      this.selection = [];
    } catch (error) {
      this.reportError(error, "That document could not be opened");
    }
  }

  async leaveProject() {
    await api.closeProject();
    this.project = null;
    this.route = "library";
    this.selection = [];
    this.focused = null;
    await this.refreshProjects();
  }

  async removeProject(id: string) {
    try {
      await api.deleteProject(id);
      this.projects = this.projects.filter((p) => p.id !== id);
      if (this.project?.id === id) {
        this.project = null;
        this.route = "library";
      }
      this.notify({ tone: "success", title: "Document deleted" });
    } catch (error) {
      this.reportError(error);
    }
  }

  async patchMeta(meta: {
    title?: string;
    summary?: string;
    prerequisites?: string[];
  }) {
    const current = this.project;
    if (!current) return;
    this.project = { ...current, ...meta };
    try {
      this.project = await api.updateProjectMeta(meta);
    } catch (error) {
      this.reportError(error);
    }
  }

  async beginRecording(sourceId: string, productId: string) {
    try {
      const project = await api.startRecording(sourceId, productId);
      void this.updateSettings({ defaultProductId: productId });
      this.project = project;
      this.route = "editor";
      this.dialog = null;
      this.selection = [];
      this.focused = null;
      this.recording = {
        state: "counting",
        elapsedMs: 0,
        stepCount: 0,
        activity: 0,
        countdown: this.settings.capture.countdownSecs,
      };
    } catch (error) {
      const appError = api.toAppError(error);
      if (appError.kind === "permission_denied") {
        this.dialog = "onboarding";
        await this.refreshPermission();
      }
      this.reportError(error, "Recording could not start");
    }
  }

  async togglePause() {
    if (!this.recording) return;
    try {
      await api.pauseRecording(this.recording.state !== "paused");
    } catch (error) {
      this.reportError(error);
    }
  }

  async markStep() {
    try {
      await api.markStep();
    } catch (error) {
      this.reportError(error);
    }
  }

  async endRecording() {
    try {
      const project = await api.stopRecording();
      this.project = project;
      this.recording = null;
      this.route = "editor";
      this.focused = project.steps[0]?.id ?? null;
      await this.refreshProjects();

      if (project.steps.length === 0) {
        this.notify({
          tone: "info",
          title: "No steps were captured",
          detail:
            "Nothing on screen changed enough to register. Try raising the sensitivity, or press the mark button while recording.",
        });
        return;
      }
      if (this.canWrite) {
        this.notify({
          tone: "success",
          title: `Captured ${project.steps.length} steps`,
          detail: "Review the screenshots, then have the instructions written.",
          action: {
            label: "Write now",
            run: () => void this.runGeneration("missing"),
          },
        });
      }
    } catch (error) {
      this.recording = null;
      this.reportError(error, "The recording could not be finished");
    }
  }

  select(id: string, mode: "replace" | "toggle" | "range" = "replace") {
    if (mode === "toggle") {
      this.selection = this.selection.includes(id)
        ? this.selection.filter((s) => s !== id)
        : [...this.selection, id];
      this.focused = id;
      return;
    }
    if (mode === "range" && this.project && this.focused) {
      const ids = this.project.steps.map((s) => s.id);
      const from = ids.indexOf(this.focused);
      const to = ids.indexOf(id);
      if (from >= 0 && to >= 0) {
        const [lo, hi] = from < to ? [from, to] : [to, from];
        this.selection = ids.slice(lo, hi + 1);
        this.focused = id;
        return;
      }
    }
    this.selection = [id];
    this.focused = id;
  }

  clearSelection() {
    this.selection = [];
  }

  async patchStep(id: string, patch: api.StepPatch) {
    const project = this.project;
    if (!project) return;
    this.project = {
      ...project,
      steps: project.steps.map((s) => (s.id === id ? { ...s, ...patch } : s)),
    };
    try {
      const updated = await api.updateStep(id, patch);
      if (this.project) {
        this.project = {
          ...this.project,
          steps: this.project.steps.map((step) => (step.id === id ? updated : step)),
        };
      }
    } catch (error) {
      this.reportError(error);
    }
  }

  async removeSteps(ids: string[]) {
    if (ids.length === 0) return;
    try {
      const project = await api.deleteSteps(ids);
      this.project = project;
      this.selection = [];
      this.focused = project.steps[0]?.id ?? null;
    } catch (error) {
      this.reportError(error);
    }
  }

  async mergeSelection() {
    if (this.selection.length < 2) return;
    const count = this.selection.length;
    const first = this.selection[0];
    try {
      const project = await api.mergeSteps(this.selection);
      this.project = project;
      this.selection = [];
      this.focused = first;
      this.notify({ tone: "success", title: `Merged ${count} steps` });
    } catch (error) {
      this.reportError(error);
    }
  }

  async reorder(order: string[]) {
    const project = this.project;
    if (!project) return;
    const byId = new Map(project.steps.map((s) => [s.id, s]));
    this.project = {
      ...project,
      steps: order.map((id) => byId.get(id)!).filter(Boolean),
    };
    try {
      this.project = await api.reorderSteps(order);
    } catch (error) {
      this.reportError(error);
    }
  }

  moveStep(id: string, direction: -1 | 1) {
    const project = this.project;
    if (!project) return;
    const idx = project.steps.findIndex((s) => s.id === id);
    const next = idx + direction;
    if (idx < 0 || next < 0 || next >= project.steps.length) return;
    const order = project.steps.map((s) => s.id);
    [order[idx], order[next]] = [order[next], order[idx]];
    void this.reorder(order);
  }

  async runGeneration(scope: api.GenerateScope, ids: string[] = []) {
    const reason = this.blockedReason;
    if (reason) {
      this.dialog = "settings";
      this.notify({ tone: "info", title: "Finish setting up a model", detail: reason });
      return;
    }
    try {
      this.generation = { done: 0, total: 0, running: true };
      await api.generate(scope, ids);
    } catch (error) {
      this.generation = null;
      this.reportError(error, "Writing could not start");
    }
  }

  async stopGeneration() {
    await api.cancelGeneration();
  }
}

export const store = new AppStore();
