/** Max lengths for user-editable text. Keep in sync with `src-tauri/src/limits.rs`. */
export const LIMITS = {
	documentTitle: 120,
	documentSummary: 500,
	prerequisite: 200,
	prerequisitesMax: 10,
	stepTitle: 80,
	stepBody: 2000,
	audience: 300,
	language: 50,
	productName: 80,
	vocabularyTerm: 80,
	vocabularyExplanation: 200,
	modelId: 128,
	baseUrl: 2048,
	apiKey: 512,
	searchQuery: 100,
} as const;
