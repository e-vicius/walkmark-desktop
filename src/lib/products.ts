import type { Product, Settings, VocabularyTerm } from "./types";

export function newProductId(): string {
  return crypto.randomUUID().replace(/-/g, "").slice(0, 12);
}

export function newVocabularyTerm(): VocabularyTerm {
  return { id: newProductId(), term: "", explanation: "" };
}

export function newProduct(name: string): Product {
  return { id: newProductId(), name, vocabulary: [] };
}

export function vocabularyPreview(product: Product, max = 2): string | null {
  const terms = product.vocabulary
    .map((entry) => entry.term.trim())
    .filter(Boolean);
  if (terms.length === 0) return null;
  const preview = terms.slice(0, max).join(", ");
  if (terms.length > max) return `${preview}…`;
  return preview;
}

export function resolveProduct(
  settings: Settings,
  productId?: string | null,
): Product | undefined {
  if (!productId) return undefined;
  return settings.products.find((p) => p.id === productId);
}

export function productLabel(settings: Settings, productId?: string | null): string {
  return resolveProduct(settings, productId)?.name ?? "General";
}
