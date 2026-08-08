<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import * as api from '$lib/api';
	import { duration, pluralize, shortcutSymbols } from '$lib/format';
	import { store } from '$lib/store.svelte';
	import { Spinner } from '$lib/components/ui/spinner';

	let busy = $state(false);
	let stopping = $state(false);

	const recording = $derived(store.recording);
	const paused = $derived(recording?.state === 'paused');
	const counting = $derived(recording?.state === 'counting');
	const finishing = $derived(stopping || recording?.state === 'stopping');

	onMount(() => {
		void getCurrentWindow().setAlwaysOnTop(true);
	});

	async function run(action: () => Promise<unknown>) {
		if (busy || finishing) return;
		busy = true;
		try {
			await action();
		} catch {
			// Main window owns error reporting.
		} finally {
			busy = false;
		}
	}

	async function stop() {
		if (busy || finishing || counting) return;
		stopping = true;
		try {
			await api.stopRecording();
		} catch {
			stopping = false;
		}
	}
</script>

<style>
	@keyframes steppy-pulse-ring {
		0% {
			opacity: 0.55;
			transform: scale(0.86);
		}
		70% {
			opacity: 0;
			transform: scale(1.5);
		}
		100% {
			opacity: 0;
			transform: scale(1.5);
		}
	}
	.pulse-ring {
		animation: steppy-pulse-ring 1.8s ease-out infinite;
	}
</style>

<div class="flex h-full w-full items-center justify-center p-1.5">
	<div
		class="flex h-full w-full items-center gap-3 rounded-3xl border border-white/10 bg-[#15161c]/95 px-3.5 shadow-[0_8px_32px_-12px_rgba(0,0,0,0.45)] backdrop-blur-xl"
	>
		<div
			data-tauri-drag-region
			class="flex min-w-0 flex-1 items-center gap-2.5"
		>
			<span class="relative flex size-2.5 flex-none">
				{#if finishing}
					<Spinner class="size-2.5 border-white/20 border-t-white" />
				{:else if !paused}
					<span class="pulse-ring absolute inline-flex size-full rounded-full bg-[#f2555a]"></span>
					<span class="relative inline-flex size-2.5 rounded-full bg-[#f2555a]"></span>
				{:else}
					<span class="relative inline-flex size-2.5 rounded-full bg-[#9a9cab]"></span>
				{/if}
			</span>
			<div class="min-w-0 leading-none">
				<div class="font-mono text-[15px] font-medium tabular-nums text-white">
					{finishing
						? 'Finishing…'
						: counting
							? `Starting in ${recording?.countdown ?? 0}`
							: duration(recording?.elapsedMs ?? 0)}
				</div>
				<div class="mt-1 truncate text-[11px] text-white/50">
					{finishing
						? 'Saving captured steps'
						: pluralize(recording?.stepCount ?? 0, 'step') + ' captured'}
				</div>
			</div>
		</div>

		<div class="no-drag flex shrink-0 items-center gap-1.5">
			<button
				type="button"
				disabled={busy || finishing || counting}
				title="Capture this moment · {shortcutSymbols(['shift', 'alt', 'M'])}"
				aria-label="Capture this moment ({shortcutSymbols(['shift', 'alt', 'M'])})"
				onclick={() => void run(api.markStep)}
				class="grid size-9 place-items-center rounded-xl bg-white/8 text-white/85 transition-all duration-150 hover:bg-white/15 hover:text-white active:scale-95 disabled:opacity-35"
			>
				<Icon icon="lucide:camera" class="size-4" />
			</button>

			<button
				type="button"
				disabled={busy || finishing || counting}
				title="{paused ? 'Resume' : 'Pause'} · {shortcutSymbols(['shift', 'alt', 'P'])}"
				aria-label="{paused ? 'Resume' : 'Pause'} ({shortcutSymbols(['shift', 'alt', 'P'])})"
				onclick={() => void run(() => api.pauseRecording(!paused))}
				class="grid size-9 place-items-center rounded-xl bg-white/8 text-white/85 transition-all duration-150 hover:bg-white/15 hover:text-white active:scale-95 disabled:opacity-35"
			>
				<Icon icon={paused ? 'lucide:play' : 'lucide:pause'} class="size-4" />
			</button>

			<button
				type="button"
				disabled={busy || finishing}
				title="Finish recording · {shortcutSymbols(['shift', 'alt', 'S'])}"
				aria-label="Finish recording ({shortcutSymbols(['shift', 'alt', 'S'])})"
				onclick={() => void stop()}
				class="grid size-9 place-items-center rounded-xl bg-[#f2555a] text-white transition-all duration-150 hover:brightness-110 active:scale-95 disabled:opacity-35"
			>
				{#if finishing}
					<Spinner class="size-4 border-white/30 border-t-white" />
				{:else}
					<Icon icon="lucide:square" class="size-3.5 fill-current" />
				{/if}
			</button>
		</div>
	</div>
</div>
