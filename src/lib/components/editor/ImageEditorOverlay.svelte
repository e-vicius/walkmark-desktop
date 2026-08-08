<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Annotation, AnnotationStroke } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { cn } from '$lib/utils';
	import Annotator, { TOOLS, type Tool } from './Annotator.svelte';
	import { isMacOS } from '$lib/window';
	import {
		DEFAULT_HIGHLIGHT,
		DEFAULT_REDACT,
		HIGHLIGHT_COLORS,
		REDACT_COLORS,
		STROKE_OPTIONS
	} from './annotation-styles';

	let {
		open = false,
		frameUrl,
		title = '',
		annotations,
		tool = $bindable<Tool>('select'),
		onChange,
		onClose
	}: {
		open?: boolean;
		frameUrl: string | null;
		title?: string;
		annotations: Annotation[];
		tool?: Tool;
		onChange: (annotations: Annotation[]) => void;
		onClose: () => void;
	} = $props();

	const isMac = isMacOS();

	let selected = $state<string | null>(null);
	let highlightColor = $state(DEFAULT_HIGHLIGHT);
	let redactColor = $state(DEFAULT_REDACT);
	let stroke = $state<AnnotationStroke>('medium');

	const activeTool = $derived(TOOLS.find((t) => t.value === tool) ?? TOOLS[0]);
	const activeColor = $derived(
		tool === 'highlight' ? highlightColor : tool === 'redact' ? redactColor : undefined
	);
	const showColors = $derived(tool === 'highlight' || tool === 'redact');
	const showStroke = $derived(tool === 'highlight');
	const colorOptions = $derived(tool === 'redact' ? REDACT_COLORS : HIGHLIGHT_COLORS);

	const selectedAnnotation = $derived(
		selected ? annotations.find((a) => a.id === selected) : undefined
	);

	function setAnnotations(next: Annotation[]) {
		onChange(next);
	}

	function undo() {
		if (annotations.length === 0) return;
		setAnnotations(annotations.slice(0, -1));
		selected = null;
	}

	function clearAll() {
		setAnnotations([]);
		selected = null;
	}

	function pickColor(hex: string) {
		if (tool === 'highlight') highlightColor = hex;
		if (tool === 'redact') redactColor = hex;

		if (!selected || !selectedAnnotation) return;
		if (
			(tool === 'highlight' && selectedAnnotation.kind !== 'highlight') ||
			(tool === 'redact' && selectedAnnotation.kind !== 'redact')
		) {
			return;
		}

		setAnnotations(
			annotations.map((a) => (a.id === selected ? { ...a, color: hex } : a))
		);
	}

	function pickStroke(next: AnnotationStroke) {
		stroke = next;
		if (selectedAnnotation?.kind === 'highlight') {
			setAnnotations(
				annotations.map((a) => (a.id === selected ? { ...a, stroke: next } : a))
			);
		}
	}

	function removeSelected() {
		if (!selected) return;
		setAnnotations(annotations.filter((a) => a.id !== selected));
		selected = null;
	}

	$effect(() => {
		if (!open) return;

		const onKey = (event: KeyboardEvent) => {
			const target = event.target as HTMLElement | null;
			if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA') return;

			if (event.key === 'Escape') {
				event.preventDefault();
				onClose();
				return;
			}

			if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'z') {
				event.preventDefault();
				undo();
				return;
			}

			if ((event.key === 'Backspace' || event.key === 'Delete') && selected) {
				event.preventDefault();
				removeSelected();
				return;
			}

			const shortcut = event.key.toUpperCase();
			const match = TOOLS.find((t) => t.shortcut === shortcut);
			if (match && !event.metaKey && !event.ctrlKey && !event.altKey) {
				event.preventDefault();
				tool = match.value;
			}
		};

		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	$effect(() => {
		if (!selectedAnnotation) return;
		if (selectedAnnotation.kind === 'highlight' && selectedAnnotation.color) {
			highlightColor = selectedAnnotation.color;
		}
		if (selectedAnnotation.kind === 'redact' && selectedAnnotation.color) {
			redactColor = selectedAnnotation.color;
		}
		if (selectedAnnotation.kind === 'highlight' && selectedAnnotation.stroke) {
			stroke = selectedAnnotation.stroke;
		}
	});
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 flex flex-col bg-(--bg)"
		role="dialog"
		aria-modal="true"
		aria-label="Edit screenshot"
	>
		<header
			data-tauri-drag-region
			class={cn(
				'titlebar flex shrink-0 items-center gap-3 border-b border-(--text)/8 pr-4 py-3 sm:pr-6',
				!isMac && 'h-12 pl-4 sm:pl-6'
			)}
		>
			<p class="no-drag min-w-0 flex-1 truncate text-sm font-medium text-(--text)">
				{title || 'Edit screenshot'}
			</p>
			<Button size="sm" class="no-drag shrink-0" onclick={onClose}>Done</Button>
		</header>

		<div class="flex min-h-0 flex-1 items-center justify-center overflow-auto p-4 pb-28 sm:p-8 sm:pb-32">
			{#if frameUrl}
				<div class="relative inline-block max-h-full max-w-full shadow-2xl shadow-black/20">
					<img
						src={frameUrl}
						alt={title ? `Screenshot for step: ${title}` : 'Screenshot'}
						draggable={false}
						class="block max-h-[calc(100vh-180px)] max-w-[min(100%,calc(100vw-2rem))] w-auto rounded-xl"
					/>
					<Annotator
						{annotations}
						{tool}
						color={activeColor}
						{stroke}
						bind:selected
						onChange={setAnnotations}
					/>
				</div>
			{:else}
				<Skeleton
					class="aspect-16/10 w-[min(960px,calc(100vw-2rem))] rounded-2xl"
				/>
			{/if}
		</div>

		<footer
			class="pointer-events-none absolute inset-x-0 bottom-0 flex justify-center p-3 sm:p-5"
		>
			<div
				class="pointer-events-auto w-full max-w-3xl rounded-2xl border border-(--text)/10 bg-(--bg-elevated)/95 p-2 shadow-xl shadow-black/15 backdrop-blur-xl sm:p-3"
			>
				<div class="flex flex-wrap items-center gap-2">
					<div class="flex items-center gap-1 rounded-xl bg-(--text)/5 p-1">
						{#each TOOLS as t (t.value)}
							<button
								type="button"
								onclick={() => (tool = t.value)}
								title="{t.label} ({t.shortcut})"
								aria-label={t.label}
								aria-pressed={tool === t.value}
								class={cn(
									'flex items-center gap-1.5 rounded-lg px-2.5 py-2 text-xs font-medium transition-colors sm:px-3',
									tool === t.value
										? 'bg-(--text) text-(--bg)'
										: 'text-(--text)/62 hover:bg-(--text)/8 hover:text-(--text)'
								)}
							>
								<Icon icon={t.icon} class="size-4 shrink-0" aria-hidden="true" />
								<span class="hidden sm:inline">{t.label}</span>
							</button>
						{/each}
					</div>

					{#if showColors}
						<span class="hidden h-8 w-px bg-(--text)/10 sm:block"></span>
						<div
							class="flex items-center gap-1.5"
							role="group"
							aria-label={tool === 'highlight' ? 'Highlight colour' : 'Cover colour'}
						>
							{#each colorOptions as option (option.hex)}
								<button
									type="button"
									title={option.label}
									aria-label={option.label}
									aria-pressed={activeColor === option.hex}
									onclick={() => pickColor(option.hex)}
									class={cn(
										'grid size-7 place-items-center rounded-full border-2 transition-transform hover:scale-105',
										activeColor === option.hex
											? 'border-(--text) ring-2 ring-(--text)/20'
											: 'border-(--text)/12'
									)}
								>
									<span
										class={cn(
											'block size-5 rounded-full',
											option.hex === '#ffffff' && 'border border-(--text)/15'
										)}
										style="background:{option.hex}"
									></span>
								</button>
							{/each}
						</div>
					{/if}

					{#if showStroke}
						<span class="hidden h-8 w-px bg-(--text)/10 sm:block"></span>
						<div
							class="flex items-center gap-1 rounded-xl bg-(--text)/5 p-1"
							role="group"
							aria-label="Outline weight"
						>
							{#each STROKE_OPTIONS as option (option.value)}
								<button
									type="button"
									onclick={() => pickStroke(option.value)}
									aria-pressed={stroke === option.value}
									class={cn(
										'rounded-lg px-2.5 py-1.5 text-xs font-medium transition-colors',
										stroke === option.value
											? 'bg-(--text) text-(--bg)'
											: 'text-(--text)/62 hover:bg-(--text)/8 hover:text-(--text)'
									)}
								>
									{option.label}
								</button>
							{/each}
						</div>
					{/if}

					<div class="ml-auto flex items-center gap-1">
						{#if selected}
							<button
								type="button"
								onclick={removeSelected}
								class="rounded-lg px-2.5 py-1.5 text-xs text-(--text)/56 hover:bg-(--text)/8 hover:text-(--text)"
							>
								Delete
							</button>
						{/if}
						{#if annotations.length > 0}
							<button
								type="button"
								onclick={undo}
								title="Undo (⌘Z)"
								class="rounded-lg px-2.5 py-1.5 text-xs text-(--text)/56 hover:bg-(--text)/8 hover:text-(--text)"
							>
								Undo
							</button>
							<button
								type="button"
								onclick={clearAll}
								class="rounded-lg px-2.5 py-1.5 text-xs text-(--text)/56 hover:bg-red-500/10 hover:text-red-500"
							>
								Clear {annotations.length}
							</button>
						{/if}
					</div>
				</div>

				<p class="mt-2 px-1 text-[11px] text-(--text)/40">
					<span class="text-(--text)/55">{activeTool.hint}.</span>
					<span class="hidden sm:inline">
						Shortcuts: V select · B blur · R cover · H highlight · ⌘Z undo · Delete removes
						selection
					</span>
				</p>
			</div>
		</footer>
	</div>
{/if}
