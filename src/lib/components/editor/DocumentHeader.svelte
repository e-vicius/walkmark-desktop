<script lang="ts">
	import Icon from '@iconify/svelte';
	import { pluralize, relativeTime } from '$lib/format';
	import { productLabel } from '$lib/products';
	import { store } from '$lib/store.svelte';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';
	import { LIMITS } from '$lib/limits';

	const project = $derived(store.project!);
	const recording = $derived(store.recording);

	const included = $derived(project.steps.filter((s) => s.include).length);
	const written = $derived(
		project.steps.filter((s) => s.include && s.status === 'ready').length
	);
	const product = $derived(productLabel(store.settings, project.productId));

	// Title field
	let titleDraft = $state('');
	let titleEditing = $state(false);
	let titleRef = $state<HTMLTextAreaElement | null>(null);

	// Summary field
	let summaryDraft = $state('');
	let summaryEditing = $state(false);
	let summaryRef = $state<HTMLTextAreaElement | null>(null);

	// Prerequisites
	let addingPrereq = $state(false);
	let prereqDraft = $state('');
	let prereqInputRef = $state<HTMLInputElement | null>(null);

	$effect(() => {
		if (!titleEditing) titleDraft = project.title;
	});
	$effect(() => {
		if (!summaryEditing) summaryDraft = project.summary;
	});
	$effect(() => {
		const node = titleRef;
		if (!node) return;
		node.style.height = 'auto';
		node.style.height = `${node.scrollHeight}px`;
	});
	$effect(() => {
		const node = summaryRef;
		if (!node) return;
		node.style.height = 'auto';
		node.style.height = `${node.scrollHeight}px`;
	});
	$effect(() => {
		if (addingPrereq) prereqInputRef?.focus();
	});

	function patchMeta(meta: { title?: string; summary?: string; prerequisites?: string[] }) {
		void store.patchMeta(meta);
	}

	function commitPrereq() {
		const value = prereqDraft.trim();
		if (value && project.prerequisites.length < LIMITS.prerequisitesMax) {
			patchMeta({ prerequisites: [...project.prerequisites, value] });
		}
		prereqDraft = '';
		addingPrereq = false;
	}
</script>

<header class="border-b border-(--text)/8 pb-7">
	<div class="mb-3 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-(--text)/40">
		<span>{product}</span>
		<span aria-hidden="true">·</span>
		{#if project.sourceLabel}
			<span class="truncate">Recorded from {project.sourceLabel}</span>
			<span aria-hidden="true">·</span>
		{/if}
		<span>{relativeTime(project.updatedAt)}</span>
		{#if included > 0 && !recording}
			<span aria-hidden="true">·</span>
			<span class={cn(written < included && 'text-amber-600 dark:text-amber-400')}>
				{written === included
					? 'All steps written'
					: `${included - written} of ${pluralize(included, 'step')} unwritten`}
			</span>
		{/if}
	</div>

	<textarea
		bind:this={titleRef}
		rows={1}
		bind:value={titleDraft}
		placeholder="Untitled guide"
		maxlength={LIMITS.documentTitle}
		onfocus={() => (titleEditing = true)}
		onblur={() => {
			titleEditing = false;
			if (titleDraft.trim() !== project.title) patchMeta({ title: titleDraft.trim() });
		}}
		onkeydown={(e) => {
			if (e.key === 'Escape') {
				titleDraft = project.title;
				e.currentTarget.blur();
			}
		}}
		class="-mx-1.5 w-full resize-none overflow-hidden rounded-xl bg-transparent px-1.5 py-0.5 text-[27px] font-semibold leading-[1.2] tracking-[-0.02em] outline-none placeholder:text-(--text)/40 hover:bg-(--text)/5 focus:bg-(--text)/8 focus:ring-1 focus:ring-(--text)/24"
	></textarea>

	<textarea
		bind:this={summaryRef}
		rows={1}
		bind:value={summaryDraft}
		placeholder="Add a sentence about what this guide covers and when to use it."
		maxlength={LIMITS.documentSummary}
		onfocus={() => (summaryEditing = true)}
		onblur={() => {
			summaryEditing = false;
			if (summaryDraft.trim() !== project.summary) patchMeta({ summary: summaryDraft.trim() });
		}}
		onkeydown={(e) => {
			if (e.key === 'Escape') {
				summaryDraft = project.summary;
				e.currentTarget.blur();
			}
		}}
		class="mt-2.5 -mx-1.5 w-full resize-none overflow-hidden rounded-xl bg-transparent px-1.5 py-0.5 text-[14.5px] leading-relaxed text-(--text)/72 outline-none placeholder:text-(--text)/40 hover:bg-(--text)/5 focus:bg-(--text)/8 focus:ring-1 focus:ring-(--text)/24"
	></textarea>

	{#if project.prerequisites.length === 0 && !addingPrereq}
		<button
			type="button"
			onclick={() => (addingPrereq = true)}
			class="-ml-1.5 mt-4 inline-flex items-center gap-1.5 rounded-xl px-1.5 py-1 text-[12.5px] text-(--text)/40 transition-colors hover:bg-(--text)/5 hover:text-(--text)/72"
		>
			<Icon icon="lucide:list-checks" class="size-3.5" aria-hidden="true" />
			Add what the reader needs before starting
		</button>
	{:else}
		<div class="mt-5 rounded-3xl bg-(--text)/5 p-3.5">
			<p class="mb-2 text-[11.5px] font-semibold tracking-wider text-(--text)/40 uppercase">
				Before you start
			</p>
			<ul class="flex flex-col gap-1">
				{#each project.prerequisites as item, index (item + index)}
					<li class="group flex items-center gap-2">
						<span class="size-1 flex-none rounded-full bg-(--text)/40" aria-hidden="true"></span>
						<span class="min-w-0 flex-1 truncate text-[13px] text-(--text)/72">{item}</span>
						<button
							type="button"
							aria-label="Remove “{item}”"
							onclick={() =>
								patchMeta({
									prerequisites: project.prerequisites.filter((_, i) => i !== index)
								})}
							class="rounded p-0.5 text-(--text)/40 opacity-0 transition-opacity hover:text-red-500 group-hover:opacity-100 focus-visible:opacity-100"
						>
							<Icon icon="lucide:x" class="size-3" aria-hidden="true" />
						</button>
					</li>
				{/each}
			</ul>

			{#if addingPrereq}
				<input
					bind:this={prereqInputRef}
					bind:value={prereqDraft}
					maxlength={LIMITS.prerequisite}
					placeholder="e.g. An admin account"
					onblur={commitPrereq}
					onkeydown={(e) => {
						if (e.key === 'Enter') commitPrereq();
						if (e.key === 'Escape') {
							prereqDraft = '';
							addingPrereq = false;
						}
					}}
					class="mt-2 w-full rounded-3xl border border-(--text)/12 bg-(--bg-elevated) px-3 py-1.5 text-[13px] outline-none"
				/>
			{:else}
				<Button
					size="sm"
					variant="ghost"
					class="-ml-2 mt-1.5"
					disabled={project.prerequisites.length >= LIMITS.prerequisitesMax}
					onclick={() => (addingPrereq = true)}
				>
					<Icon icon="lucide:plus" class="size-3.5" />
					Add item
				</Button>
			{/if}
		</div>
	{/if}
</header>
