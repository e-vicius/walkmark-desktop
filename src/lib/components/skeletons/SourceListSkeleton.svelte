<script lang="ts">
	import { Skeleton } from '$lib/components/ui/skeleton';

	let { count = 6 }: { count?: number } = $props();
</script>

<div
	class="grid gap-3 pb-2"
	style="grid-template-columns: repeat(auto-fill, minmax(210px, 1fr))"
>
	{#each Array.from({ length: count }) as _, i (i)}
		<div class="overflow-hidden rounded-2xl bg-(--text)/5">
			<Skeleton class="aspect-16/10 w-full rounded-none" />
			<div class="space-y-1.5 px-3 py-2.5">
				<Skeleton class="h-3.5 w-3/4 rounded-lg" />
				<Skeleton class="h-3 w-1/2 rounded-full" />
			</div>
		</div>
	{/each}
</div>
