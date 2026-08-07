<script lang="ts">
	import * as api from "$lib/api";
	import { store } from "$lib/store.svelte";
	import type { ProviderId } from "$lib/types";
	import { Badge } from "$lib/components/ui/badge";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import LocalModels from "$lib/components/LocalModels.svelte";
	import ApiKeyGuide from "$lib/components/ApiKeyGuide.svelte";

	let { class: className = "" }: { class?: string } = $props();

	let keyDraft = $state("");
	let keyBusy = $state(false);

	const provider = $derived(store.catalog.providers.find((p) => p.id === store.settings.provider));
	const hasKey = $derived(store.catalog.configured.includes(store.settings.provider));
	const selectedModel = $derived(store.activeConfig.model);

	async function saveKey() {
		if (!provider || !keyDraft.trim()) return;
		keyBusy = true;
		try {
			await api.verifyProvider({ provider: provider.id, key: keyDraft.trim() });
			await api.setApiKey(provider.id, keyDraft.trim());
			await store.refreshCredentials();
			keyDraft = "";
			store.notify({ tone: "success", title: "Key saved" });
		} catch (e) {
			store.reportError(e, "That key didn't work");
		} finally {
			keyBusy = false;
		}
	}
</script>

<div class="grid min-h-0 grid-cols-[128px_minmax(0,1fr)] gap-4 {className}">
	<nav class="flex flex-col gap-1 overflow-y-auto">
		{#each store.catalog.providers as p (p.id)}
			<button
				type="button"
				class="rounded-2xl px-3 py-2 text-left text-sm transition-colors {store.settings.provider ===
				p.id
					? 'bg-(--text)/8 text-(--text)'
					: 'text-(--text)/56 hover:bg-(--text)/5'}"
				onclick={() => void store.updateSettings({ provider: p.id as ProviderId })}
			>
				{p.name}
			</button>
		{/each}
	</nav>

	{#if provider}
		<div class="min-h-0 min-w-0 space-y-4">
			<p class="text-sm text-(--text)/56">{provider.blurb}</p>

			{#if provider.local}
				<LocalModels
					selected={selectedModel}
					onSelect={(m) => void store.configureProvider(provider.id, { model: m })}
				/>
			{:else}
				{#if provider.baseUrlEditable}
					<div class="space-y-1.5">
						<Label>Server address</Label>
						<Input
							value={store.settings.providers[provider.id]?.baseUrl ?? ""}
							placeholder={provider.defaultBaseUrl}
							oninput={(e) =>
								void store.configureProvider(provider.id, {
									baseUrl: (e.currentTarget as HTMLInputElement).value,
								})}
						/>
					</div>
				{/if}

				<div class="space-y-1.5">
					<Label>Model</Label>
					{#if provider.models.length > 0}
						<div class="space-y-1">
							{#each provider.models as model (model.id)}
								<button
									type="button"
									class="w-full rounded-2xl px-3 py-2.5 text-left transition-colors {selectedModel ===
									model.id
										? 'bg-(--text)/8 text-(--text)'
										: 'text-(--text)/72 hover:bg-(--text)/5'}"
									onclick={() => void store.configureProvider(provider.id, { model: model.id })}
								>
									<div class="flex items-center gap-2">
										<span class="text-sm font-medium">{model.name}</span>
										{#if model.tier === "recommended"}
											<Badge>Recommended</Badge>
										{/if}
									</div>
									<p class="mt-0.5 text-xs text-(--text)/56">{model.note}</p>
								</button>
							{/each}
						</div>
					{:else}
						<Input
							value={store.settings.providers[provider.id]?.model || provider.defaultModel}
							placeholder="Model name"
							oninput={(e) =>
								void store.configureProvider(provider.id, {
									model: (e.currentTarget as HTMLInputElement).value,
								})}
						/>
					{/if}
				</div>

				{#if provider.needsKey}
					<ApiKeyGuide {provider} />
				{/if}

				{#if provider.needsKey || !hasKey}
					<div class="space-y-1.5">
						<Label>{hasKey ? "Replace API key" : "Paste your API key"}</Label>
						<div class="flex gap-2">
							<Input
								type="password"
								bind:value={keyDraft}
								class="font-mono text-xs"
								placeholder="Paste your key"
							/>
							<Button disabled={!keyDraft.trim() || keyBusy} onclick={saveKey}>Save</Button>
						</div>
					</div>
				{/if}
			{/if}
		</div>
	{/if}
</div>
