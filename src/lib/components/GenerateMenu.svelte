<script lang="ts">
	import Icon from "@iconify/svelte";
	import { pluralize } from "$lib/format";
	import { store } from "$lib/store.svelte";
	import { Button } from "$lib/components/ui/button";
	import WriteModelSelect from "$lib/components/WriteModelSelect.svelte";

	const project = $derived(store.project);
	const missing = $derived(
		project?.steps.filter((s) => s.include && s.status !== "ready") ?? [],
	);
	const included = $derived(project?.steps.filter((s) => s.include) ?? []);
	const busy = $derived(Boolean(store.generation?.running));
	const canWrite = $derived(store.canWriteWith(store.writeProvider) && Boolean(store.writeModel));
</script>

{#if project}
	<div class="flex items-center gap-1">
		<WriteModelSelect disabled={busy} />
		<Button
			size="sm"
			disabled={included.length === 0 || busy || !canWrite}
			onclick={() => void store.runGeneration(missing.length > 0 ? "missing" : "all")}
		>
			<Icon icon="lucide:sparkles" class="size-3.5" />
			{busy ? "Writing" : missing.length > 0 ? `Write ${pluralize(missing.length, "step")}` : "Rewrite"}
		</Button>
		{#if missing.length > 0}
			<Button
				size="sm"
				variant="ghost"
				disabled={busy || !canWrite}
				onclick={() => void store.runGeneration("all")}
				title="Rewrite every step"
			>
				<Icon icon="lucide:refresh-cw" class="size-3.5" />
			</Button>
		{/if}
	</div>
{/if}
