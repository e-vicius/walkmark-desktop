/** `mm:ss`, or `h:mm:ss` once a recording passes an hour. */
export function duration(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return hours > 0
    ? `${hours}:${pad(minutes)}:${pad(seconds)}`
    : `${minutes}:${pad(seconds)}`;
}

/** "just now", "12 min ago", "yesterday", then a plain date. */
export function relativeTime(iso: string): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";

  const diff = Date.now() - then;
  const minutes = Math.round(diff / 60_000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes} min ago`;

  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} hr ago`;
  if (hours < 48) return "yesterday";

  const days = Math.round(hours / 24);
  if (days < 7) return `${days} days ago`;

  return new Date(then).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: days > 300 ? "numeric" : undefined,
  });
}

export function fileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/**
 * Model downloads and machine memory are gigabyte-scale, where a "7482.3 MB"
 * reading is noise. Rounds to whole gigabytes above 10.
 */
export function gigabytes(bytes: number): string {
  if (bytes <= 0) return "—";
  const gb = bytes / 1e9;
  if (gb < 1) return `${Math.round(bytes / 1e6)} MB`;
  return `${gb >= 10 ? Math.round(gb) : gb.toFixed(1)} GB`;
}

export function pluralize(count: number, singular: string, plural = `${singular}s`) {
  return `${count} ${count === 1 ? singular : plural}`;
}

/** Human label for the sensitivity slider, which is otherwise meaningless. */
export function sensitivityLabel(value: number): string {
  if (value < 0.25) return "Only big changes";
  if (value < 0.45) return "Fewer steps";
  if (value < 0.65) return "Balanced";
  if (value < 0.85) return "More steps";
  return "Every small change";
}

export function shortcutSymbols(keys: string[]): string {
  const isMac = navigator.platform.toLowerCase().includes("mac");
  return keys
    .map((key) => {
      switch (key) {
        case "mod":
          return isMac ? "⌘" : "Ctrl";
        case "shift":
          return isMac ? "⇧" : "Shift";
        case "alt":
          return isMac ? "⌥" : "Alt";
        default:
          return key.toUpperCase();
      }
    })
    .join(isMac ? "" : "+");
}
