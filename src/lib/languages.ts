/** Languages offered in writing settings. Values are passed to the writing prompt. */
export const DOCUMENT_LANGUAGES = [
	"English",
	"Spanish",
	"French",
	"German",
	"Portuguese",
	"Italian",
	"Dutch",
	"Polish",
	"Japanese",
	"Korean",
	"Lithuanian",
	"Chinese (Simplified)",
	"Chinese (Traditional)",
] as const;

export type DocumentLanguage = (typeof DOCUMENT_LANGUAGES)[number];

export const OTHER_LANGUAGE = "Other";

export function isPresetLanguage(language: string): language is DocumentLanguage {
	return DOCUMENT_LANGUAGES.includes(language.trim() as DocumentLanguage);
}

/** Value bound to the language Select (preset name or {@link OTHER_LANGUAGE}). */
export function languageSelectValue(language: string): string {
	const trimmed = language.trim();
	if (isPresetLanguage(trimmed)) return trimmed;
	return OTHER_LANGUAGE;
}

/** Label shown on the closed Select trigger. */
export function languageSelectLabel(language: string): string {
	const trimmed = language.trim();
	if (isPresetLanguage(trimmed)) return trimmed;
	if (trimmed) return trimmed;
	return OTHER_LANGUAGE;
}

export function languageOptions(): string[] {
	return [...DOCUMENT_LANGUAGES, OTHER_LANGUAGE];
}
