<script lang="ts">
	import { Skeleton } from '$lib/components/ui/skeleton';
</script>

<div class="flex h-full">
	<aside class="flex w-[248px] flex-none flex-col border-r border-(--text)/8 bg-(--bg-elevated)">
		<div class="flex items-center justify-between px-4 py-3">
			<Skeleton class="h-3 w-10 rounded-full" />
			<Skeleton class="h-3 w-4 rounded-full" />
		</div>
		<div class="space-y-1 px-2 pb-4">
			{#each Array.from({ length: 5 }) as _, i (i)}
				<div class="flex items-center gap-2 rounded-2xl p-2">
					<Skeleton class="size-7 shrink-0 rounded-xl" />
					<Skeleton class="h-10 flex-1 rounded-xl" />
				</div>
			{/each}
		</div>
	</aside>

	<div class="min-w-0 flex-1 overflow-y-auto">
		<div class="mx-auto w-full max-w-[840px] px-10 pt-8 pb-32">
			<header class="border-b border-(--text)/8 pb-7">
				<div class="mb-3 flex gap-2">
					<Skeleton class="h-3 w-16 rounded-full" />
					<Skeleton class="h-3 w-24 rounded-full" />
				</div>
				<Skeleton class="h-8 w-2/3 max-w-md rounded-xl" />
				<Skeleton class="mt-3 h-4 w-full max-w-lg rounded-lg" />
				<Skeleton class="mt-2 h-4 w-4/5 max-w-md rounded-lg" />
			</header>

			<ol class="mt-8 flex flex-col gap-3">
				{#each Array.from({ length: 3 }) as _, i (i)}
					<li class="rounded-3xl bg-(--bg-elevated) p-4">
						<div class="flex items-start gap-3 pb-2">
							<Skeleton class="size-7 shrink-0 rounded-xl" />
							<div class="min-w-0 flex-1 space-y-2">
								<Skeleton class="h-4 w-1/2 rounded-lg" />
								<Skeleton class="h-3 w-full rounded-lg" />
								<Skeleton class="h-3 w-5/6 rounded-lg" />
							</div>
						</div>
						<Skeleton class="aspect-16/10 w-full rounded-2xl" />
					</li>
				{/each}
			</ol>
		</div>
	</div>
</div>
