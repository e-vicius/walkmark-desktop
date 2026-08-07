<script lang="ts">
	import { onMount } from "svelte";
	import Icon from "@iconify/svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { gigabytes } from "$lib/format";
	import type { LocalCatalogEntry } from "$lib/types";
	import { store } from "$lib/store.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Badge } from "$lib/components/ui/badge";
	import { Progress } from "$lib/components/ui/progress";
	import { Spinner } from "$lib/components/ui/spinner";

	let { selected, onSelect }: { selected: string; onSelect: (model: string) => void } = $props();

	const installed = $derived(store.local?.models.filter((m) => m.vision) ?? []);
	const available = $derived(store.local?.catalog.filter((e) => !e.installed) ?? []);

	onMount(() => {
		void store.refreshLocal();
	});
</script>

{#if !store.local}
	<div class="flex items-center gap-2 py-6 text-sm text-(--text)/56">
		<Spinner class="size-4" />
		Looking for Ollama…
	</div>
{:else if !store.local.running}
	<div class="flex flex-col items-center gap-3 rounded-3xl bg-(--bg) px-6 py-8 text-center">
		<p class="text-sm text-(--text)/72">Install Ollama to download models on this Mac.</p>
		<div class="flex gap-2">
			<Button onclick={() => void openUrl(store.local?.downloadUrl ?? "https://ollama.com/download")}>
				Get Ollama
			</Button>
			<Button variant="ghost" onclick={() => void store.refreshLocal()}>I've installed it</Button>
		</div>
	</div>
{:else}
	<div class="space-y-4">
		{#if store.pull && !store.pull.done}
			<div class="space-y-2 rounded-3xl bg-(--bg) p-4">
				<div class="flex items-center justify-between text-sm">
					<span>Downloading {store.pull.model}</span>
					<button class="text-xs text-(--text)/56" onclick={() => void store.cancelPull()}>
						Cancel
					</button>
				</div>
				<Progress
					value={store.pull.total > 0 ? (store.pull.completed / store.pull.total) * 100 : undefined}
				/>
			</div>
		{/if}

		{#if installed.length > 0}
			<div class="space-y-1">
				{#each installed as model (model.id)}
					<div
						class="flex w-full items-center justify-between rounded-2xl px-3 py-2.5 text-sm transition-colors {selected ===
						model.id
							? 'bg-(--text)/8 text-(--text)'
							: 'text-(--text)/72 hover:bg-(--text)/5'}"
					>
						<button
							type="button"
							class="min-w-0 flex-1 text-left font-mono text-xs"
							onclick={() => onSelect(model.id)}
						>
							{model.id}
						</button>
						<button
							type="button"
							class="text-xs text-(--text)/40 hover:text-red-500"
							onclick={() => void store.deleteModel(model.id)}
						>
							Remove
						</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if available.length > 0}
			<div class="space-y-2">
				{#each available as entry (entry.id)}
					{@render catalogRow(entry)}
				{/each}
			</div>
		{:else if installed.length === 0}
			<p class="text-sm text-(--text)/56">All recommended models are installed.</p>
		{/if}
	</div>
{/if}

{#snippet catalogRow(entry: LocalCatalogEntry)}
	<div class="flex items-center gap-3 rounded-2xl bg-(--bg) px-3 py-2.5">
		<div class="min-w-0 flex-1">
			<div class="flex items-center gap-2">
				<span class="text-sm font-medium text-(--text)">{entry.name}</span>
				{#if entry.recommended}<Badge>Recommended</Badge>{/if}
			</div>
			<p class="text-xs text-(--text)/56">{entry.note}</p>
		</div>
		<Button
			size="sm"
			variant="ghost"
			disabled={!entry.fits || (store.pull !== null && !store.pull.done)}
			title={entry.fits ? undefined : "This model needs more memory than this Mac has"}
			onclick={() => {
				onSelect(entry.id);
				void store.pullModel(entry.id);
			}}
		>
			<Icon icon="lucide:download" class="size-3.5" />
			{gigabytes(entry.size)}
		</Button>
	</div>
{/snippet}
