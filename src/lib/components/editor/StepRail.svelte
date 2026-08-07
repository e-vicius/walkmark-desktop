<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Step } from '$lib/types';
	import { store } from '$lib/store.svelte';
	import { cn } from '$lib/utils';
	import FrameImage from './FrameImage.svelte';

	const project = $derived(store.project);
	const isMac = typeof navigator !== 'undefined' && navigator.userAgent.includes('Mac');
</script>

{#if project}
	<aside class="flex w-[248px] flex-none flex-col border-r border-(--text)/8 bg-(--bg-elevated)">
		<div class="flex items-center justify-between px-4 py-3">
			<span class="text-[11.5px] font-semibold tracking-wider text-(--text)/40 uppercase">
				Steps
			</span>
			<span class="text-[11.5px] tabular-nums text-(--text)/40">
				{project.steps.length}
			</span>
		</div>

		<div class="min-h-0 flex-1 overflow-y-auto px-2 pb-4">
			<ol class="flex flex-col gap-1">
				{#each project.steps as step, index (step.id)}
					{@render railItem({ step, index, projectId: project.id, total: project.steps.length })}
				{/each}
			</ol>

			{#if project.steps.length > 1}
				<p class="mt-4 px-2 text-[11.5px] leading-relaxed text-(--text)/40">
					Use arrows to reorder. Hold {isMac ? '⌘' : 'Ctrl'} to select several.
				</p>
			{/if}
		</div>
	</aside>
{/if}

{#snippet railItem({
	step,
	index,
	projectId,
	total
}: {
	step: Step;
	index: number;
	projectId: string;
	total: number;
})}
	{@const active = store.focused === step.id}
	{@const selected = store.selection.includes(step.id)}

	<li class="relative">
		<div
			class={cn(
				'group flex w-full items-center gap-1 rounded-xl p-1 text-left transition-colors',
				selected ? 'bg-(--text)/12' : active ? 'bg-(--text)/8' : 'hover:bg-(--text)/5'
			)}
		>
			<div class="flex flex-none flex-col opacity-0 transition-opacity group-hover:opacity-100">
				<button
					type="button"
					aria-label="Move step up"
					disabled={index === 0}
					onclick={() => store.moveStep(step.id, -1)}
					class="grid size-4 place-items-center rounded text-(--text)/40 hover:bg-(--text)/8 hover:text-(--text) disabled:opacity-30"
				>
					<Icon icon="lucide:chevron-up" class="size-3" />
				</button>
				<button
					type="button"
					aria-label="Move step down"
					disabled={index === total - 1}
					onclick={() => store.moveStep(step.id, 1)}
					class="grid size-4 place-items-center rounded text-(--text)/40 hover:bg-(--text)/8 hover:text-(--text) disabled:opacity-30"
				>
					<Icon icon="lucide:chevron-down" class="size-3" />
				</button>
			</div>

			<button
				type="button"
				onclick={(event) =>
					store.select(
						step.id,
						event.metaKey || event.ctrlKey
							? 'toggle'
							: event.shiftKey
								? 'range'
								: 'replace'
					)}
				class="flex min-w-0 flex-1 items-center gap-2 rounded-lg p-0.5 text-left"
			>
				<span
					class={cn(
						'w-4 flex-none text-center text-[11px] font-medium tabular-nums',
						active || selected ? 'text-(--text)' : 'text-(--text)/40'
					)}
				>
					{index + 1}
				</span>

				<div
					class={cn(
						'relative aspect-16/10 w-[60px] flex-none overflow-hidden rounded-lg bg-(--text)/5',
						active && 'ring-2 ring-(--text)/24',
						!step.include && 'opacity-40 grayscale'
					)}
				>
					<FrameImage
						{projectId}
						frame={step.frame}
						class="size-full object-cover object-top"
					/>
				</div>

				<div class="min-w-0 flex-1">
					<p
						class={cn(
							'line-clamp-2 text-[12px] leading-[1.35]',
							step.title ? 'text-(--text)' : 'text-(--text)/40 italic'
						)}
					>
						{step.title || 'Not written yet'}
					</p>
					{@render statusDot({ step })}
				</div>
			</button>
		</div>
	</li>
{/snippet}

{#snippet statusDot({ step }: { step: Step })}
	{#if !step.include}
		<span class="mt-1 flex items-center gap-1 text-[10.5px] text-(--text)/40">
			<Icon icon="lucide:eye-off" class="size-2.5" aria-hidden="true" /> Excluded
		</span>
	{:else if step.status === 'failed'}
		<span class="mt-1 flex items-center gap-1 text-[10.5px] text-red-500">
			<Icon icon="lucide:alert-circle" class="size-2.5" aria-hidden="true" /> Failed
		</span>
	{:else if step.status === 'queued' || step.status === 'generating'}
		<span
			class="mt-1.5 block h-1.5 w-10 animate-pulse rounded-full bg-(--text)/12"
			aria-label="Writing"
		></span>
	{:else if step.manual}
		<span class="mt-1 flex items-center gap-1 text-[10.5px] text-(--text)/40">
			<Icon icon="lucide:pin" class="size-2.5" aria-hidden="true" /> Pinned
		</span>
	{/if}
{/snippet}
