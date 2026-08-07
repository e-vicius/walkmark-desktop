<script lang="ts">
	let { seconds }: { seconds: number } = $props();
</script>

<div
	class="fixed inset-0 z-70 grid animate-in fade-in place-items-center bg-(--bg)/85 backdrop-blur-sm"
	role="status"
	aria-live="polite"
>
	<div class="flex flex-col items-center gap-4">
		{#key seconds}
			<div
				class="grid size-24 animate-in fade-in zoom-in-95 place-items-center rounded-3xl bg-(--bg-elevated) text-[44px] font-semibold tabular-nums text-(--text) shadow-lg"
			>
				{seconds}
			</div>
		{/key}
		<p class="text-sm tracking-[-0.39px] text-(--text)/56">
			Switch to what you want to document
		</p>
	</div>
</div>
