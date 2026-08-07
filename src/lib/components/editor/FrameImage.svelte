<script lang="ts">
	import * as api from '$lib/api';
	import { cn } from '$lib/utils';

	let {
		projectId,
		frame,
		class: className = '',
		alt = '',
		onload
	}: {
		projectId: string;
		frame: string;
		class?: string;
		alt?: string;
		onload?: (event: Event) => void;
	} = $props();

	let url = $state<string | null>(null);
	let loaded = $state(false);

	$effect(() => {
		if (!frame) {
			url = null;
			return;
		}
		let active = true;
		loaded = false;
		void api
			.frameUrl(projectId, frame)
			.then((resolved) => {
				if (active) url = resolved;
			})
			.catch(() => {
				if (active) url = null;
			});
		return () => {
			active = false;
		};
	});

	$effect(() => {
		url;
		loaded = false;
	});
</script>

{#if !url}
	<div class={cn('animate-pulse bg-(--text)/8', className)} aria-hidden="true"></div>
{:else}
	<img
		src={url}
		{alt}
		draggable={false}
		onload={(event) => {
			loaded = true;
			onload?.(event);
		}}
		class={cn('transition-opacity duration-200', loaded ? 'opacity-100' : 'opacity-0', className)}
	/>
{/if}
