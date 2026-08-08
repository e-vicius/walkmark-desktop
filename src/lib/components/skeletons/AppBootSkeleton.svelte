<script lang="ts">
	import { Skeleton } from '$lib/components/ui/skeleton';
	import LibrarySkeleton from './LibrarySkeleton.svelte';
</script>

<div class="flex h-full flex-col">
	<header
		class="titlebar relative z-30 flex flex-none items-center gap-3 border-b border-(--text)/8 bg-(--bg-elevated) px-3 py-3"
	>
		<Skeleton class="size-8 rounded-2xl" />
		<Skeleton class="h-8 w-36 rounded-xl" />
		<div class="ml-auto flex items-center gap-2">
			<Skeleton class="h-9 w-28 rounded-3xl" />
		</div>
	</header>
	<main class="min-h-0 flex-1 overflow-hidden">
		<LibrarySkeleton count={6} />
	</main>
</div>
