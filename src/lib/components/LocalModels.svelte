<script lang="ts">
	import { onMount } from "svelte";
	import Icon from "@iconify/svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { gigabytes } from "$lib/format";
	import type { LocalCatalogEntry } from "$lib/types";
	import { store } from "$lib/store.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Badge } from "$lib/components/ui/badge";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import { Progress } from "$lib/components/ui/progress";
	import { Skeleton } from "$lib/components/ui/skeleton";
	import { LIMITS } from "$lib/limits";

	let { selected, onSelect }: { selected: string; onSelect: (model: string) => void } = $props();

	let customModel = $state("");

	const installed = $derived(store.local?.models.filter((m) => m.vision) ?? []);
	const available = $derived(store.local?.catalog.filter((e) => !e.installed) ?? []);
	const catalogIds = $derived(new Set(store.local?.catalog.map((e) => e.id) ?? []));
	const pulling = $derived(store.pull !== null && !store.pull.done);
	const ollamaRunning = $derived(store.local?.running ?? false);

	$effect(() => {
		if (selected && !catalogIds.has(selected) && !installed.some((m) => m.id === selected)) {
			customModel = selected;
		}
	});

	function useCustom() {
		const id = customModel.trim();
		if (!id) return;
		onSelect(id);
	}

	function pullCustom() {
		const id = customModel.trim();
		if (!id) return;
		onSelect(id);
		void store.pullModel(id);
	}

	onMount(() => {
		void store.refreshLocal();
	});

	function useCompatibleEndpoint() {
		void store.updateSettings({ provider: "compatible" });
	}
</script>

{#if !store.local}
	<div class="space-y-3 py-4">
		<Skeleton class="h-4 w-40 rounded-lg" />
		<Skeleton class="h-16 w-full rounded-3xl" />
		<Skeleton class="h-16 w-full rounded-3xl" />
	</div>
{:else}
	<div class="space-y-4">
		{#if !ollamaRunning}
			<div class="flex flex-col gap-3 rounded-3xl bg-(--bg) px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
				<p class="text-sm text-(--text)/72">
					Install Ollama to download models on this Mac.
				</p>
				<div class="flex shrink-0 gap-2">
					<Button
						size="sm"
						onclick={() => void openUrl(store.local?.downloadUrl ?? "https://ollama.com/download")}
					>
						Get Ollama
					</Button>
					<Button size="sm" variant="ghost" onclick={() => void store.refreshLocal()}>
						I've installed it
					</Button>
				</div>
			</div>
		{/if}

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
				<p class="px-1 text-xs font-medium text-(--text)/56">Installed</p>
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
				<p class="px-1 text-xs font-medium text-(--text)/56">Recommended models</p>
				{#each available as entry (entry.id)}
					{@render catalogRow(entry)}
				{/each}
			</div>
		{:else if installed.length > 0}
			<p class="text-sm text-(--text)/56">All recommended models are installed.</p>
		{/if}

		<div class="space-y-2 border-t border-(--text)/8 pt-4">
			<Label class="text-xs text-(--text)/56">Or enter a model name</Label>
			<div class="flex flex-wrap gap-2">
				<Input
					bind:value={customModel}
					maxlength={LIMITS.modelId}
					placeholder="e.g. qwen3-vl:8b"
					class="min-w-[12rem] flex-1 font-mono text-xs"
					onkeydown={(e) => {
						if (e.key === "Enter") useCustom();
					}}
				/>
				<Button size="sm" variant="ghost" disabled={!customModel.trim()} onclick={useCustom}>
					Use
				</Button>
				<Button
					size="sm"
					disabled={!customModel.trim() || pulling || !ollamaRunning}
					title={ollamaRunning ? undefined : "Install and start Ollama first"}
					onclick={pullCustom}
				>
					<Icon icon="lucide:download" class="size-3.5" />
					Pull
				</Button>
			</div>
			<p class="text-xs text-(--text)/40">
				Any model from the Ollama library. Vision models work best for screenshots.
			</p>
		</div>

		<div class="rounded-2xl bg-(--text)/5 px-4 py-3 text-xs leading-relaxed text-(--text)/56">
			<p class="font-medium text-(--text)/72">Don't want Ollama?</p>
			<p class="mt-1">
				Steppy doesn't ship its own inference engine — Ollama handles downloads and GPU setup for
				the models above. If you already run Qwen, Gemma, or similar in
				<strong class="font-medium text-(--text)/72">LM Studio</strong>,
				<strong class="font-medium text-(--text)/72">llama.cpp</strong>, or
				<strong class="font-medium text-(--text)/72">MLX</strong>, switch to
				<button
					type="button"
					class="text-(--text) underline decoration-(--text)/24 underline-offset-2 hover:decoration-(--text)/56"
					onclick={useCompatibleEndpoint}
				>
					Custom endpoint
				</button>
				and paste your server's address (for example <code class="font-mono">http://localhost:1234/v1</code>).
			</p>
		</div>
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
			disabled={!entry.fits || (store.pull !== null && !store.pull.done) || !ollamaRunning}
			title={!ollamaRunning
				? "Install and start Ollama first"
				: entry.fits
					? undefined
					: "This model needs more memory than this Mac has"}
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
