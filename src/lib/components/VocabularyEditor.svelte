<script lang="ts">
	import Icon from "@iconify/svelte";
	import type { VocabularyTerm } from "$lib/types";
	import { newVocabularyTerm } from "$lib/products";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import { LIMITS } from "$lib/limits";

	let {
		terms,
		compact = false,
		onChange,
	}: {
		terms: VocabularyTerm[];
		compact?: boolean;
		onChange: (terms: VocabularyTerm[]) => void;
	} = $props();

	function patch(next: VocabularyTerm[]) {
		onChange(next);
	}

	function addTerm() {
		patch([...terms, newVocabularyTerm()]);
	}

	function updateTerm(id: string, update: Partial<VocabularyTerm>) {
		patch(terms.map((entry) => (entry.id === id ? { ...entry, ...update } : entry)));
	}

	function removeTerm(id: string) {
		patch(terms.filter((entry) => entry.id !== id));
	}
</script>

<div class="space-y-2">
	<Label class="text-xs text-(--text)/56">Vocabulary</Label>

	{#if terms.length === 0}
		<p class="text-xs text-(--text)/40">Add terms the model should prefer when writing steps.</p>
	{/if}

	{#each terms as entry (entry.id)}
		<div class="flex items-start gap-2">
			<div class="grid min-w-0 flex-1 gap-2 sm:grid-cols-2">
				<Input
					value={entry.term}
					maxlength={LIMITS.vocabularyTerm}
					placeholder="Term"
					oninput={(e) =>
						updateTerm(entry.id, { term: (e.currentTarget as HTMLInputElement).value })}
				/>
				<Input
					value={entry.explanation}
					maxlength={LIMITS.vocabularyExplanation}
					placeholder="What it is"
					oninput={(e) =>
						updateTerm(entry.id, {
							explanation: (e.currentTarget as HTMLInputElement).value,
						})}
				/>
			</div>
			<button
				type="button"
				class="mt-1 grid size-8 shrink-0 place-items-center rounded-xl text-(--text)/40 hover:bg-(--text)/5 hover:text-red-500"
				aria-label="Remove term"
				onclick={() => removeTerm(entry.id)}
			>
				<Icon icon="lucide:trash-2" class="size-3.5" />
			</button>
		</div>
	{/each}

	<Button size="sm" variant="ghost" class={compact ? "" : "mt-1"} onclick={addTerm}>
		<Icon icon="lucide:plus" class="size-3.5" />
		Add term
	</Button>
</div>
