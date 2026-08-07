<script lang="ts">
	import Icon from "@iconify/svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import type { ProviderInfo } from "$lib/types";
	import { Button } from "$lib/components/ui/button";

	let { provider }: { provider: ProviderInfo } = $props();
</script>

{#if provider.needsKey && provider.keyGuide.length > 0}
	<div class="space-y-3 rounded-2xl bg-(--bg) p-4">
		<div>
			<p class="text-sm font-medium text-(--text)">How to get an API key</p>
			{#if provider.keyHint}
				<p class="mt-0.5 text-xs text-(--text)/56">The key {provider.keyHint.toLowerCase()}.</p>
			{/if}
		</div>

		<ol class="space-y-2 text-xs leading-relaxed text-(--text)/72">
			{#each provider.keyGuide as step, index (index)}
				<li class="flex gap-2">
					<span
						class="grid size-5 shrink-0 place-items-center rounded-full bg-(--text)/8 text-[11px] font-medium text-(--text)/56"
					>
						{index + 1}
					</span>
					<span class="min-w-0 pt-0.5">{step}</span>
				</li>
			{/each}
		</ol>

		{#if provider.keyUrl}
			<Button size="sm" variant="ghost" onclick={() => void openUrl(provider.keyUrl)}>
				<Icon icon="lucide:external-link" class="size-3.5" />
				Open {provider.name} key page
			</Button>
		{/if}
	</div>
{/if}
