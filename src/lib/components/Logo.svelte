<script lang="ts">
	import { cn } from "$lib/utils";
	import { APP_NAME } from "$lib/brand";

	let {
		class: className,
		markClass = "size-7",
		showName = false,
		inverted = false,
	}: {
		class?: string;
		markClass?: string;
		showName?: boolean;
		inverted?: boolean;
	} = $props();
</script>

<div class={cn("flex items-center gap-2.5", className)}>
	<div
		class={cn(
			"grid shrink-0 place-items-center rounded-xl",
			markClass,
			inverted ? "bg-(--text) text-(--bg)" : "bg-(--text)/8 text-(--text)",
		)}
		aria-hidden="true"
	>
		<svg viewBox="0 0 24 24" class="size-[58%]" fill="currentColor">
			<!-- Three ascending treads — the mark for "steps". -->
			<rect x="3" y="15.5" width="8.5" height="3.5" rx="1.75" opacity="0.42" />
			<rect x="7.25" y="10.25" width="8.5" height="3.5" rx="1.75" opacity="0.72" />
			<rect x="11.5" y="5" width="8.5" height="3.5" rx="1.75" />
		</svg>
	</div>
	{#if showName}
		<span class="text-sm font-semibold tracking-tight text-(--text)">{APP_NAME}</span>
	{/if}
</div>
