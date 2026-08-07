/** Languages offered when starting a recording. Values are passed to the writing prompt. */
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
	"Chinese (Simplified)",
	"Chinese (Traditional)",
] as const;

export function languageOptions(current: string): string[] {
	const trimmed = current.trim();
	if (!trimmed || DOCUMENT_LANGUAGES.includes(trimmed as (typeof DOCUMENT_LANGUAGES)[number])) {
		return [...DOCUMENT_LANGUAGES];
	}
	return [trimmed, ...DOCUMENT_LANGUAGES];
}
