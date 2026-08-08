<script lang="ts">
	import { Skeleton } from '$lib/components/ui/skeleton';

	let { count = 6 }: { count?: number } = $props();
</script>

<div class="mx-auto w-full max-w-[960px] px-8 py-12">
	<Skeleton class="mb-4 h-3 w-28 rounded-full" />
	<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
		{#each Array.from({ length: count }) as _, i (i)}
			<article class="overflow-hidden rounded-3xl bg-(--bg-elevated)">
				<Skeleton class="aspect-video w-full rounded-none" />
				<div class="space-y-2 p-4">
					<Skeleton class="h-4 w-3/4 rounded-lg" />
					<Skeleton class="h-3 w-1/2 rounded-full" />
				</div>
			</article>
		{/each}
	</div>
</div>
