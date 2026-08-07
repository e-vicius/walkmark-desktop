<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import * as api from '$lib/api';
	import { duration, pluralize, shortcutSymbols } from '$lib/format';
	import { store } from '$lib/store.svelte';
	import { cn } from '$lib/utils';

	let busy = $state(false);

	const recording = $derived(store.recording);
	const paused = $derived(recording?.state === 'paused');
	const counting = $derived(recording?.state === 'counting');

	onMount(() => {
		void getCurrentWindow().setAlwaysOnTop(true);
	});

	async function run(action: () => Promise<unknown>) {
		busy = true;
		try {
			await action();
		} catch {
			// Main window owns error reporting.
		} finally {
			busy = false;
		}
	}

	function activityBars(value: number, muted: boolean) {
		const bars = 5;
		return muted ? 0 : Math.min(bars, Math.round(value * 22) + (value > 0.002 ? 1 : 0));
	}

	const activeBars = $derived(activityBars(recording?.activity ?? 0, paused || counting));
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

<div class="flex h-full w-full items-center justify-center bg-transparent p-1.5">
	<div
		data-tauri-drag-region
		class="flex h-full w-full items-center gap-3 rounded-3xl border border-white/10 bg-[#15161c]/95 px-3.5 shadow-[0_10px_40px_-8px_rgba(0,0,0,0.7)] backdrop-blur-xl"
	>
		<div class="pointer-events-none flex items-center gap-2.5">
			<span class="relative flex size-2.5 flex-none">
				{#if !paused}
					<span class="pulse-ring absolute inline-flex size-full rounded-full bg-[#f2555a]"></span>
				{/if}
				<span
					class={cn(
						'relative inline-flex size-2.5 rounded-full',
						paused ? 'bg-[#9a9cab]' : 'bg-[#f2555a]'
					)}
				></span>
			</span>
			<div class="leading-none">
				<div class="font-mono text-[15px] font-medium tabular-nums text-white">
					{counting
						? `Starting in ${recording?.countdown ?? 0}`
						: duration(recording?.elapsedMs ?? 0)}
				</div>
				<div class="mt-1 text-[11px] text-white/50">
					{pluralize(recording?.stepCount ?? 0, 'step')} captured
				</div>
			</div>
		</div>

		<div class="pointer-events-none flex h-6 items-end gap-[3px]" aria-label="Screen activity">
			{#each Array.from({ length: 5 }) as _, i (i)}
				<span
					class={cn(
						'w-[3px] rounded-full transition-all duration-200',
						i < activeBars ? 'bg-[#6366f1]' : 'bg-white/12'
					)}
					style="height: {8 + i * 3}px"
				></span>
			{/each}
		</div>

		<div class="ml-auto flex items-center gap-1.5">
			<button
				type="button"
				disabled={busy || counting}
				title="Capture this moment · {shortcutSymbols(['shift', 'alt', 'M'])}"
				aria-label="Capture this moment ({shortcutSymbols(['shift', 'alt', 'M'])})"
				onclick={() => void run(api.markStep)}
				class="grid size-9 place-items-center rounded-xl bg-white/8 text-white/85 transition-all duration-150 hover:bg-white/15 hover:text-white active:scale-95 disabled:opacity-35"
			>
				<Icon icon="lucide:camera" class="size-4" />
			</button>

			<button
				type="button"
				disabled={busy || counting}
				title="{paused ? 'Resume' : 'Pause'} · {shortcutSymbols(['shift', 'alt', 'P'])}"
				aria-label="{paused ? 'Resume' : 'Pause'} ({shortcutSymbols(['shift', 'alt', 'P'])})"
				onclick={() => void run(() => api.pauseRecording(!paused))}
				class="grid size-9 place-items-center rounded-xl bg-white/8 text-white/85 transition-all duration-150 hover:bg-white/15 hover:text-white active:scale-95 disabled:opacity-35"
			>
				<Icon icon={paused ? 'lucide:play' : 'lucide:pause'} class="size-4" />
			</button>

			<button
				type="button"
				disabled={busy}
				title="Finish recording · {shortcutSymbols(['shift', 'alt', 'S'])}"
				aria-label="Finish recording ({shortcutSymbols(['shift', 'alt', 'S'])})"
				onclick={() => void run(api.stopRecording)}
				class="grid size-9 place-items-center rounded-xl bg-[#f2555a] text-white transition-all duration-150 hover:brightness-110 active:scale-95 disabled:opacity-35"
			>
				<Icon icon="lucide:square" class="size-3.5 fill-current" />
			</button>
		</div>
	</div>
</div>
