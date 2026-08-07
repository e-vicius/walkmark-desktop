<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Step } from '$lib/types';
	import { duration } from '$lib/format';
	import { store } from '$lib/store.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';
	import Annotator, { TOOLS, type Tool } from './Annotator.svelte';
	import FrameImage from './FrameImage.svelte';
	import * as api from '$lib/api';

	let { step, index }: { step: Step; index: number } = $props();

	const project = $derived(store.project!);
	const active = $derived(store.focused === step.id);
	const selected = $derived(store.selection.includes(step.id));
	const writing = $derived(step.status === 'queued' || step.status === 'generating');
	const ready = $derived(store.canWrite);

	const number = $derived(
		project.steps.slice(0, index + 1).filter((s) => s.include).length
	);

	let tool = $state<Tool>('select');
	let showFrames = $state(false);

	// Title draft
	let titleDraft = $state('');
	let titleEditing = $state(false);

	// Body draft
	let bodyDraft = $state('');
	let bodyEditing = $state(false);
	let bodyRef = $state<HTMLTextAreaElement | null>(null);

	$effect(() => {
		if (!titleEditing) titleDraft = step.title;
	});
	$effect(() => {
		if (!bodyEditing) bodyDraft = step.body;
	});
	$effect(() => {
		const node = bodyRef;
		if (!node) return;
		node.style.height = 'auto';
		node.style.height = `${node.scrollHeight}px`;
	});

	let frameUrl = $state<string | null>(null);
	$effect(() => {
		let activeEffect = true;
		void api.frameUrl(project.id, step.frame).then((url) => {
			if (activeEffect) frameUrl = url;
		});
		return () => {
			activeEffect = false;
		};
	});

	function patchStep(patch: Parameters<typeof store.patchStep>[1]) {
		void store.patchStep(step.id, patch);
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<li
	id="step-{step.id}"
	onpointerdown={(event) => {
		if (event.defaultPrevented) return;
		if (!selected) {
			store.select(step.id, event.metaKey || event.ctrlKey ? 'toggle' : 'replace');
		}
	}}
	class={cn(
		'group/step scroll-mt-6 rounded-3xl bg-(--bg-elevated) transition-[background-color] duration-150',
		selected && 'ring-2 ring-(--text)/24',
		active && !selected && 'ring-1 ring-(--text)/12',
		!step.include && 'opacity-60'
	)}
>
	<div class="flex items-start gap-3 p-4 pb-2">
		<span
			class={cn(
				'mt-0.5 grid size-7 flex-none place-items-center rounded-xl text-[12.5px] font-semibold tabular-nums',
				step.include ? 'bg-(--text) text-(--bg)' : 'bg-(--text)/5 text-(--text)/40'
			)}
		>
			{step.include ? number : '–'}
		</span>

		<div class="min-w-0 flex-1">
			{#if writing}
				<div class="flex flex-col gap-2 py-1">
					<div class="h-4 w-2/3 animate-pulse rounded bg-(--text)/8"></div>
					<div class="h-3 w-full animate-pulse rounded bg-(--text)/8"></div>
					<div class="h-3 w-4/5 animate-pulse rounded bg-(--text)/8"></div>
				</div>
			{:else}
				<input
					bind:value={titleDraft}
					placeholder="Name this step"
					onfocus={() => (titleEditing = true)}
					onblur={() => {
						titleEditing = false;
						if (titleDraft.trim() !== step.title) patchStep({ title: titleDraft.trim() });
					}}
					onkeydown={(e) => {
						if (e.key === 'Enter') e.currentTarget.blur();
						if (e.key === 'Escape') {
							titleDraft = step.title;
							e.currentTarget.blur();
						}
					}}
					class="w-full rounded-xl bg-transparent px-1 py-0.5 -mx-1 text-base font-semibold leading-snug tracking-[-0.011em] outline-none placeholder:font-normal placeholder:text-(--text)/40 hover:bg-(--text)/5 focus:bg-(--text)/8 focus:ring-1 focus:ring-(--text)/24"
				/>
				<textarea
					bind:this={bodyRef}
					rows={1}
					bind:value={bodyDraft}
					placeholder={ready
						? 'Describe what the reader should do — or have it written for you.'
						: 'Describe what the reader should do here.'}
					onfocus={() => (bodyEditing = true)}
					onblur={() => {
						bodyEditing = false;
						if (bodyDraft.trim() !== step.body) patchStep({ body: bodyDraft.trim() });
					}}
					onkeydown={(e) => {
						if (e.key === 'Escape') {
							bodyDraft = step.body;
							e.currentTarget.blur();
						}
					}}
					class="mt-1 -mx-1 w-full resize-none overflow-hidden rounded-xl bg-transparent px-1 py-0.5 text-[13.5px] leading-relaxed text-(--text)/72 outline-none placeholder:text-(--text)/40 hover:bg-(--text)/5 focus:bg-(--text)/8 focus:text-(--text) focus:ring-1 focus:ring-(--text)/24"
				></textarea>
			{/if}
		</div>

		<div
			class="flex flex-none items-center gap-1 opacity-0 transition-opacity group-hover/step:opacity-100 focus-within:opacity-100"
		>
			{#if step.locked}
				<span title="You edited this. 'Rewrite everything' will leave it alone.">
					<Badge>
						<Icon icon="lucide:lock" class="size-2.5" aria-hidden="true" /> Edited
					</Badge>
				</span>
			{/if}
			<Button
				size="icon"
				variant="ghost"
				aria-label={step.include ? 'Exclude from the document' : 'Include in the document'}
				onclick={() => patchStep({ include: !step.include })}
			>
				<Icon icon={step.include ? 'lucide:eye' : 'lucide:eye-off'} class="size-3.5" />
			</Button>
			<Button
				size="icon"
				variant="ghost"
				aria-label="Rewrite this step"
				disabled={writing || !ready}
				onclick={() => void store.runGeneration('only', [step.id])}
			>
				<Icon icon="lucide:sparkles" class="size-3.5" />
			</Button>
			<Button
				size="icon"
				variant="ghost"
				aria-label="Delete this step"
				class="hover:text-red-500"
				onclick={() => void store.removeSteps([step.id])}
			>
				<Icon icon="lucide:trash-2" class="size-3.5" />
			</Button>
		</div>
	</div>

	{#if step.status === 'failed' && step.error}
		<div class="mx-4 mb-3 flex items-start gap-2 rounded-2xl bg-red-500/10 px-3 py-2.5">
			<Icon icon="lucide:alert-circle" class="mt-0.5 size-3.5 flex-none text-red-500" />
			<p class="min-w-0 flex-1 text-[12.5px] leading-relaxed text-red-600 dark:text-red-400">
				{step.error}
			</p>
			<Button
				size="sm"
				variant="ghost"
				onclick={() => void store.runGeneration('only', [step.id])}
			>
				<Icon icon="lucide:refresh-cw" class="size-3" />
				Retry
			</Button>
		</div>
	{/if}

	<div class="px-4 pb-4">
		<div class="relative">
			<div class="relative overflow-hidden rounded-2xl bg-(--text)/5">
				{#if frameUrl}
					<img
						src={frameUrl}
						alt="Screenshot for step: {step.title || 'untitled'}"
						draggable={false}
						class="block w-full"
					/>
				{:else}
					<div class="aspect-16/10 w-full animate-pulse bg-(--text)/8" aria-hidden="true"></div>
				{/if}

				<Annotator
					annotations={step.annotations}
					{tool}
					onChange={(annotations) => patchStep({ annotations })}
				/>
			</div>

			<div
				class="absolute top-2 left-2 flex items-center gap-1 rounded-xl border border-white/10 bg-black/65 p-1 opacity-0 shadow-md backdrop-blur transition-opacity duration-150 group-hover/step:opacity-100 focus-within:opacity-100"
			>
				{#each TOOLS as t (t.value)}
					<button
						type="button"
						onpointerdown={(event) => event.stopPropagation()}
						onclick={() => (tool = t.value)}
						title="{t.label} — {t.hint}"
						aria-label={t.label}
						aria-pressed={tool === t.value}
						class={cn(
							'grid size-7 place-items-center rounded-lg transition-colors',
							tool === t.value
								? 'bg-white text-black'
								: 'text-white/70 hover:bg-white/15 hover:text-white'
						)}
					>
						<Icon icon={t.icon} class="size-3.5" aria-hidden="true" />
					</button>
				{/each}

				{#if step.annotations.length > 0}
					<span class="mx-0.5 h-4 w-px bg-white/15"></span>
					<button
						type="button"
						onpointerdown={(event) => event.stopPropagation()}
						onclick={() => patchStep({ annotations: [] })}
						class="rounded-lg px-2 py-1 text-[11.5px] text-white/70 hover:bg-white/15 hover:text-white"
					>
						Clear {step.annotations.length}
					</button>
				{/if}
			</div>

			<div
				class="absolute top-2 right-2 flex items-center gap-1 opacity-0 transition-opacity group-hover/step:opacity-100 focus-within:opacity-100"
			>
				<span
					class="rounded-lg bg-black/65 px-2 py-1 text-[11px] tabular-nums text-white/70 backdrop-blur"
				>
					{duration(step.offsetMs)}
				</span>
				{#if step.alternates.length > 0}
					<button
						type="button"
						onpointerdown={(event) => event.stopPropagation()}
						onclick={() => (showFrames = !showFrames)}
						class={cn(
							'flex items-center gap-1.5 rounded-lg px-2 py-1 text-[11.5px] backdrop-blur transition-colors',
							showFrames
								? 'bg-white text-black'
								: 'bg-black/65 text-white/80 hover:bg-black/80 hover:text-white'
						)}
					>
						<Icon icon="lucide:images" class="size-3" aria-hidden="true" />
						Different moment
					</button>
				{/if}
			</div>

			{#if showFrames}
				{@const options = [step.frame, ...step.alternates].sort()}
				<div
					onpointerdown={(event) => event.stopPropagation()}
					class="absolute inset-x-2 bottom-2 animate-in fade-in slide-in-from-bottom-2 rounded-2xl border border-white/10 bg-black/80 p-2 shadow-lg backdrop-blur-xl"
				>
					<div class="mb-2 flex items-center justify-between px-1">
						<span class="text-[11.5px] font-medium text-white/80">
							Pick the screenshot that shows the step best
						</span>
						<button
							type="button"
							onclick={() => (showFrames = false)}
							class="text-[11.5px] text-white/60 hover:text-white"
						>
							Done
						</button>
					</div>
					<div class="flex gap-2 overflow-x-auto pb-1">
						{#each options as frame (frame)}
							{@const current = frame === step.frame}
							<button
								type="button"
								onclick={() => {
									if (!current) {
										patchStep({ frame });
										showFrames = false;
									}
								}}
								class={cn(
									'relative w-[132px] flex-none overflow-hidden rounded-xl border-2 transition-colors',
									current ? 'border-white' : 'border-transparent hover:border-white/40'
								)}
							>
								<FrameImage
									projectId={project.id}
									{frame}
									class="aspect-16/10 w-full object-cover object-top"
								/>
								{#if current}
									<span
										class="absolute top-1 right-1 grid size-4 place-items-center rounded-full bg-white"
									>
										<Icon icon="lucide:check" class="size-2.5 text-black" />
									</span>
								{/if}
							</button>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	</div>
</li>
