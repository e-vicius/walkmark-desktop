<script lang="ts">
	import Icon from '@iconify/svelte';
	import { pluralize } from '$lib/format';
	import { store } from '$lib/store.svelte';
	import { Button } from '$lib/components/ui/button';
	import DocumentHeader from './DocumentHeader.svelte';
	import StepCard from './StepCard.svelte';
	import StepRail from './StepRail.svelte';

	const project = $derived(store.project);
	const focused = $derived(store.focused);
	const recording = $derived(store.recording);
	const selection = $derived(store.selection);

	let scrolledTo = $state<string | null>(null);
	const lastStepId = $derived(project?.steps.at(-1)?.id ?? null);

	$effect(() => {
		if (!focused || scrolledTo === focused) return;
		scrolledTo = focused;
		document.getElementById(`step-${focused}`)?.scrollIntoView({ block: 'center', behavior: 'smooth' });
	});

	$effect(() => {
		if (!recording || !lastStepId) return;
		document.getElementById(`step-${lastStepId}`)?.scrollIntoView({ block: 'end', behavior: 'smooth' });
	});
</script>

{#if project}
	<div class="flex h-full">
		<StepRail />

		<div class="min-w-0 flex-1 overflow-y-auto">
			<div class="mx-auto w-full max-w-[840px] px-10 pt-8 pb-32">
				<DocumentHeader />

				{#if project.steps.length === 0}
					{#if recording}
						<div
							class="mt-10 flex flex-col items-center gap-3 rounded-3xl border border-dashed border-(--text)/12 px-8 py-14 text-center"
						>
							<span class="relative flex size-3">
								<span
									class="absolute inline-flex size-full animate-ping rounded-full bg-red-500 opacity-60"
								></span>
								<span class="relative inline-flex size-3 rounded-full bg-red-500"></span>
							</span>
							<p class="text-[13.5px] font-medium">Watching your screen</p>
							<p class="max-w-xs text-[12.5px] leading-relaxed text-(--text)/56">
								Steps will appear here as you work. Press the camera button in the floating
								controller to capture a moment yourself.
							</p>
						</div>
					{:else}
						<div class="mt-10 flex flex-col items-center gap-4 rounded-3xl bg-(--text)/5 px-8 py-14 text-center">
							<Icon icon="lucide:circle-dot" class="size-6 text-(--text)/40" />
							<div>
								<p class="text-[15px] font-semibold">This document has no steps</p>
								<p class="mt-1.5 max-w-sm text-[13px] leading-relaxed text-(--text)/56">
									Record the workflow again — Walkmark captures a step each time the screen
									changes meaningfully.
								</p>
							</div>
							<Button onclick={() => store.setDialog('sources')}>Record steps</Button>
						</div>
					{/if}
				{:else}
					<ol class="mt-8 flex flex-col gap-3">
						{#each project.steps as step, index (step.id)}
							<StepCard {step} {index} />
						{/each}
					</ol>
				{/if}
			</div>
		</div>

		{#if selection.length > 0}
			<div
				class="fixed bottom-6 left-1/2 z-40 flex -translate-x-1/2 animate-in fade-in slide-in-from-bottom-2 items-center gap-1.5 rounded-3xl bg-(--bg-elevated) p-1.5 pl-4 shadow-lg"
			>
				<span class="mr-2 text-[12.5px] font-medium">
					{pluralize(selection.length, 'step')} selected
				</span>
				<Button
					size="sm"
					variant="ghost"
					disabled={selection.length < 2}
					onclick={() => void store.mergeSelection()}
				>
					<Icon icon="lucide:combine" class="size-3.5" />
					Merge
				</Button>
				<Button
					size="sm"
					variant="ghost"
					onclick={() => {
						selection.forEach((id) => void store.patchStep(id, { include: false }));
						store.clearSelection();
					}}
				>
					<Icon icon="lucide:eye-off" class="size-3.5" />
					Exclude
				</Button>
				<Button
					size="sm"
					variant="ghost"
					class="hover:text-red-500"
					onclick={() => void store.removeSteps(selection)}
				>
					<Icon icon="lucide:trash-2" class="size-3.5" />
					Delete
				</Button>
				<div class="mx-1 h-5 w-px bg-(--text)/12"></div>
				<Button size="sm" variant="ghost" onclick={() => store.clearSelection()}>
					<Icon icon="lucide:x" class="size-3.5" />
					Clear
				</Button>
			</div>
		{/if}
	</div>
{/if}
