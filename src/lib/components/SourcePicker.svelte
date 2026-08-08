<script lang="ts">
	import Icon from '@iconify/svelte';
	import * as api from '$lib/api';
	import { store } from '$lib/store.svelte';
	import type { CaptureSource } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle
	} from '$lib/components/ui/dialog';
	import { Input } from '$lib/components/ui/input';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import SourceListSkeleton from '$lib/components/skeletons/SourceListSkeleton.svelte';
	import { Spinner } from '$lib/components/ui/spinner';
	import { ToggleGroup, ToggleGroupItem } from '$lib/components/ui/toggle-group';
	import RecordingContextForm from '$lib/components/RecordingContextForm.svelte';
	import { LIMITS } from '$lib/limits';
	import { cn } from '$lib/utils';
	import type { Tone } from '$lib/types';

	let { open = false, onClose }: { open?: boolean; onClose?: () => void } = $props();

	function handleOpenChange(value: boolean) {
		if (!value) onClose?.();
	}

	let sources = $state<CaptureSource[] | null>(null);
	let previewsPending = $state(false);
	let kind = $state<'monitor' | 'window'>('monitor');
	let query = $state('');
	let step = $state<'context' | 'capture'>('context');
	let selected = $state<string | null>(null);
	let selectedProduct = $state<string | null>(null);
	let audience = $state('');
	let tone = $state<Tone>('neutral');
	let language = $state('English');
	let starting = $state(false);

	let requestToken = 0;

	const settings = $derived(store.settings);
	const permission = $derived(store.permission);
	const blocked = $derived(permission.required && !permission.granted);

	const visible = $derived.by(() => {
		if (!sources) return [];
		const needle = query.trim().toLowerCase();
		return sources.filter(
			(source) =>
				source.kind === kind &&
				(needle === '' ||
					source.name.toLowerCase().includes(needle) ||
					source.detail.toLowerCase().includes(needle))
		);
	});

	const counts = $derived({
		monitor: sources?.filter((s) => s.kind === 'monitor').length ?? 0,
		window: sources?.filter((s) => s.kind === 'window').length ?? 0
	});

	$effect(() => {
		if (open) {
			step = 'context';
			selectedProduct = null;
			audience = store.settings.audience;
			tone = store.settings.tone;
			language = store.settings.language;
			void load();
		} else {
			query = '';
		}
	});

	function canContinueContext() {
		return Boolean(audience.trim());
	}

	async function continueToCapture() {
		if (!canContinueContext()) return;
		await store.updateSettings({
			audience: audience.trim(),
			tone,
			language: language.trim() || 'English',
			...(selectedProduct ? { defaultProductId: selectedProduct } : {}),
		});
		step = 'capture';
	}

	async function load() {
		const token = ++requestToken;
		sources = null;
		previewsPending = true;
		try {
			const found = await api.listSources(false);
			if (requestToken !== token) return;
			sources = found;
			selected ??=
				found.find((s) => s.kind === 'monitor' && s.isPrimary)?.id ?? found[0]?.id ?? null;

			const previews = await api.listSources(true);
			if (requestToken !== token) return;
			const byId = new Map(previews.map((s) => [s.id, s.thumbnail] as const));
			sources = (sources ?? []).map((source) => ({
				...source,
				thumbnail: byId.get(source.id) ?? source.thumbnail
			}));
		} catch (error) {
			if (requestToken !== token) return;
			sources = [];
			store.reportError(error, 'Steppy could not list your screens');
		} finally {
			if (requestToken === token) previewsPending = false;
		}
	}

	async function start() {
		if (!selected) return;
		starting = true;
		await store.beginRecording(selected, selectedProduct);
		starting = false;
	}
</script>

<Dialog {open} onOpenChange={handleOpenChange}>
	<DialogContent class="flex h-[min(720px,90vh)] w-full max-w-[1040px] flex-col gap-0 overflow-hidden p-6" showCloseButton={true}>
		<DialogHeader class="shrink-0">
			{#if step === 'context'}
				<DialogTitle>What are you documenting?</DialogTitle>
				<DialogDescription>
					Say who this guide is for. Pick a product if you want its vocabulary applied.
				</DialogDescription>
			{:else}
				<DialogTitle>What should Steppy watch?</DialogTitle>
				<DialogDescription>
					Pick a whole screen, or a single window to keep everything else out of your document.
				</DialogDescription>
			{/if}
		</DialogHeader>

		{#if blocked}
			<div class="grid place-items-center px-8 py-16 text-center">
				<Icon icon="lucide:monitor" class="mb-4 size-7 text-(--text)/40" />
				<h3 class="text-[15px] font-semibold">macOS hasn't granted screen access</h3>
				<p class="mt-1.5 max-w-sm text-[13px] leading-relaxed text-(--text)/56">
					Until you allow it, Steppy can list your screens but cannot capture what is on them.
				</p>
				<Button class="mt-6" onclick={() => store.setDialog('onboarding')}>Show me how</Button>
			</div>
		{:else if step === 'context'}
			<div class="min-h-0 flex-1 overflow-y-auto">
				<RecordingContextForm
					bind:selectedProduct
					bind:audience
					bind:tone
					bind:language
				/>
			</div>

			<DialogFooter class="mt-4 shrink-0">
				<Button variant="ghost" onclick={() => onClose?.()}>Cancel</Button>
				<Button disabled={!canContinueContext()} onclick={() => void continueToCapture()}>
					Continue
					<Icon icon="lucide:arrow-right" class="size-4" />
				</Button>
			</DialogFooter>
		{:else}
			<div class="flex min-h-0 flex-1 flex-col overflow-hidden">
			<div class="mb-4 flex shrink-0 flex-wrap items-center gap-3">
				<ToggleGroup bind:value={kind} type="single" variant="outline" class="rounded-3xl">
					<ToggleGroupItem value="monitor" aria-label="Screens">
						<Icon icon="lucide:monitor" class="size-3.5" />
						Screens{counts.monitor ? ` (${counts.monitor})` : ''}
					</ToggleGroupItem>
					<ToggleGroupItem value="window" aria-label="Windows">
						<Icon icon="lucide:app-window" class="size-3.5" />
						Windows{counts.window ? ` (${counts.window})` : ''}
					</ToggleGroupItem>
				</ToggleGroup>

				<div class="relative min-w-[180px] flex-1">
					<Icon
						icon="lucide:search"
						class="pointer-events-none absolute top-1/2 left-3 size-3.5 -translate-y-1/2 text-(--text)/40"
					/>
					<Input bind:value={query} maxlength={LIMITS.searchQuery} placeholder="Filter by name" class="pl-9" />
				</div>

				<Button variant="ghost" size="icon" aria-label="Refresh the list" onclick={() => void load()}>
					<Icon icon="lucide:refresh-cw" class="size-4" />
				</Button>
			</div>

			{#if sources === null}
				<div class="min-h-0 flex-1 overflow-y-auto py-1">
					<SourceListSkeleton />
				</div>
			{:else if visible.length === 0}
				<div class="grid min-h-0 flex-1 place-items-center px-8 text-center">
					<p class="text-[13px] text-(--text)/56">
						{query
							? `Nothing matches “${query}”.`
							: kind === 'window'
								? 'No open windows are large enough to record. Minimised windows are hidden.'
								: 'No screens were found.'}
					</p>
				</div>
			{:else}
				<div
					class="min-h-0 flex-1 overflow-y-auto"
				>
				<div
					class="grid gap-3 pb-2"
					style="grid-template-columns: repeat(auto-fill, minmax(210px, 1fr))"
				>
					{#each visible as source (source.id)}
						<button
							type="button"
							onclick={() => (selected = source.id)}
							ondblclick={() => void start()}
							aria-pressed={selected === source.id}
							class={cn(
								'group relative overflow-hidden rounded-2xl bg-(--text)/5 text-left transition-all duration-150',
								selected === source.id ? 'ring-2 ring-(--text)/24' : 'hover:bg-(--text)/8'
							)}
						>
							<div class="relative aspect-16/10 overflow-hidden bg-(--text)/8">
								{#if source.thumbnail}
									<img
										src={source.thumbnail}
										alt=""
										class="size-full object-cover object-top"
									/>
								{:else if previewsPending}
									<Skeleton class="size-full rounded-none" />
								{:else}
									<div class="grid size-full place-items-center text-(--text)/40">
										<Icon
											icon={source.kind === 'monitor' ? 'lucide:monitor' : 'lucide:app-window'}
											class="size-5"
										/>
									</div>
								{/if}
								{#if selected === source.id}
									<span
										class="absolute top-2 right-2 grid size-5 place-items-center rounded-full bg-(--text) text-(--bg)"
									>
										<Icon icon="lucide:check" class="size-3" />
									</span>
								{/if}
								{#if source.isPrimary}
									<span
										class="absolute top-2 left-2 rounded bg-black/55 px-1.5 py-0.5 text-[10.5px] font-medium text-white backdrop-blur"
									>
										Main
									</span>
								{/if}
							</div>
							<div class="px-3 py-2.5">
								<p class="truncate text-[12.5px] font-medium">{source.name}</p>
								<p class="truncate text-[11.5px] text-(--text)/56">{source.detail}</p>
							</div>
						</button>
					{/each}
				</div>
				</div>
			{/if}

			<p class="mt-4 shrink-0 border-t border-(--text)/8 pt-3 text-[12px] leading-relaxed text-(--text)/56">
				{settings.capture.hideWindow
					? 'Steppy will step out of the way and leave a small floating controller on screen.'
					: 'Steppy will stay open. Its own window is never offered as a capture source.'}
			</p>

			<DialogFooter class="mt-4 shrink-0 sm:justify-between">
				<Button variant="ghost" class="mr-auto" onclick={() => (step = 'context')}>
					<Icon icon="lucide:arrow-left" class="size-3.5" />
					Back
				</Button>
				<Button variant="ghost" onclick={() => store.openSettings('recording', 'sources')}>
					<Icon icon="lucide:sliders-horizontal" class="size-3.5" />
					Recording preferences
				</Button>
				<Button variant="ghost" onclick={() => onClose?.()}>Cancel</Button>
				<Button disabled={!selected || starting} onclick={() => void start()}>
					{#if starting}
						<Spinner class="size-4" />
					{:else}
						<Icon icon="lucide:circle-dot" class="size-4" />
					{/if}
					{settings.capture.countdownSecs > 0
						? `Start in ${settings.capture.countdownSecs}s`
						: 'Start recording'}
				</Button>
			</DialogFooter>
			</div>
		{/if}
	</DialogContent>
</Dialog>
