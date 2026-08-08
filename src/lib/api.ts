import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import type {
  Annotation,
  AppError,
  CaptureSource,
  ExportFormat,
  ExportOptions,
  ExportResult,
  LocalOverview,
  PermissionStatus,
  Project,
  ProjectSummary,
  ProviderCatalog,
  ProviderId,
  RecordingStatus,
  Settings,
  Step,
} from "./types";

/**
 * Normalises anything thrown across the IPC bridge into our error shape, so
 * callers never have to guess whether they caught a string, an Error or the
 * struct the backend serialises.
 */
export function toAppError(error: unknown): AppError {
  if (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    "message" in error
  ) {
    return error as AppError;
  }
  if (error instanceof Error) return { kind: "other", message: error.message };
  return { kind: "other", message: String(error) };
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toAppError(error);
  }
}

// --- Settings & credentials -------------------------------------------------

export const getSettings = () => call<Settings>("get_settings");
export const saveSettings = (settings: Settings) =>
  call<Settings>("save_settings", { settings });

export const permissionStatus = () => call<PermissionStatus>("permission_status");
export const requestPermission = () => call<PermissionStatus>("request_permission");
export const openPrivacySettings = () => call<void>("open_privacy_settings");
export const openAccessibilitySettings = () => call<void>("open_accessibility_settings");

export const providerCatalog = () => call<ProviderCatalog>("provider_catalog");
export const setApiKey = (provider: ProviderId, key: string) =>
  call<void>("set_api_key", { provider, key });
export const clearApiKey = (provider: ProviderId) =>
  call<void>("clear_api_key", { provider });

/**
 * Round-trips a trivial request. Anything left out falls back to what is
 * already saved, so the dialog can verify a key the user just typed without
 * committing it first.
 */
export const verifyProvider = (args: {
  provider: ProviderId;
  model?: string;
  baseUrl?: string;
  key?: string | null;
}) => call<void>("verify_provider", { key: null, ...args });

// --- Local models -----------------------------------------------------------

export const localStatus = () => call<LocalOverview>("local_status");
export const downloadModel = (model: string) =>
  call<void>("download_model", { model });
export const cancelDownload = () => call<void>("cancel_download");
export const removeModel = (model: string) =>
  call<void>("remove_model", { model });

// --- Capture ----------------------------------------------------------------

export const listSources = (withThumbnails: boolean) =>
  call<CaptureSource[]>("list_sources", { withThumbnails });

export const recordingStatus = () => call<RecordingStatus>("recording_status");
export const startRecording = (sourceId: string, productId?: string | null) =>
  call<Project>("start_recording", { sourceId, productId: productId ?? null });
export const pauseRecording = (paused: boolean) =>
  call<void>("pause_recording", { paused });
export const markStep = () => call<void>("mark_step");
export const stopRecording = () => call<Project>("stop_recording");
export const appendStep = (step: Step) => call<void>("append_step", { step });
export const attachAlternate = (stepId: string, frame: string) =>
  call<void>("attach_alternate", { stepId, frame });

// --- Projects ---------------------------------------------------------------

export const listProjects = () => call<ProjectSummary[]>("list_projects");
export const openProject = (id: string) => call<Project>("open_project", { id });
export const currentProject = () => call<Project | null>("current_project");
export const closeProject = () => call<void>("close_project");
export const deleteProject = (id: string) => call<void>("delete_project", { id });

export const updateProjectMeta = (meta: {
  title?: string;
  summary?: string;
  prerequisites?: string[];
}) => call<Project>("update_project_meta", { meta });

const framePathCache = new Map<string, string>();

/**
 * Frames are served over the asset protocol rather than base64 over IPC —
 * screenshots are megabytes each and would otherwise be copied twice per render.
 */
export async function frameUrl(projectId: string, frame: string): Promise<string> {
  const key = `${projectId}/${frame}`;
  const cached = framePathCache.get(key);
  if (cached) return cached;

  const path = await call<string>("frame_path", { projectId, frame });
  const url = convertFileSrc(path);
  framePathCache.set(key, url);
  return url;
}

// --- Steps ------------------------------------------------------------------

export interface StepPatch {
  title?: string;
  body?: string;
  include?: boolean;
  locked?: boolean;
  frame?: string;
  annotations?: Annotation[];
}

export const updateStep = (id: string, patch: StepPatch) =>
  call<Step>("update_step", { id, patch });
export const reorderSteps = (order: string[]) =>
  call<Project>("reorder_steps", { order });
export const deleteSteps = (ids: string[]) => call<Project>("delete_steps", { ids });
export const mergeSteps = (ids: string[]) => call<Project>("merge_steps", { ids });

// --- Generation -------------------------------------------------------------

export type GenerateScope = "missing" | "all" | "only";

export interface GenerationRequest {
  provider?: ProviderId;
  model?: string;
}

export const generate = (
  scope: GenerateScope,
  ids: string[] = [],
  request: GenerationRequest = {},
) =>
  call<void>("generate", {
    scope,
    ids,
    provider: request.provider ?? null,
    model: request.model ?? null,
  });
export const cancelGeneration = () => call<void>("cancel_generation");

// --- Export -----------------------------------------------------------------

export const suggestExportName = (format: ExportFormat) =>
  call<string>("suggest_export_name", { format });
export const exportDocument = (options: ExportOptions, destination: string) =>
  call<ExportResult>("export_document", { options, destination });
export const copyAsMarkdown = () => call<string>("copy_as_markdown");
export const revealInFolder = (path: string) =>
  call<void>("reveal_in_folder", { path });
