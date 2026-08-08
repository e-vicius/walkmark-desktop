<script lang="ts" module>
	import type { AnnotationKind, AnnotationStroke } from '$lib/types';

	export type Tool = 'select' | AnnotationKind;

	export const TOOLS: {
		value: Tool;
		label: string;
		icon: string;
		hint: string;
		shortcut: string;
	}[] = [
		{
			value: 'select',
			label: 'Select',
			icon: 'lucide:mouse-pointer-2',
			hint: 'Select, move, or delete a region',
			shortcut: 'V'
		},
		{
			value: 'blur',
			label: 'Blur',
			icon: 'lucide:scan',
			hint: 'Pixelate sensitive details',
			shortcut: 'B'
		},
		{
			value: 'redact',
			label: 'Cover',
			icon: 'lucide:square',
			hint: 'Paint over a region completely',
			shortcut: 'R'
		},
		{
			value: 'highlight',
			label: 'Highlight',
			icon: 'lucide:highlighter',
			hint: 'Draw a coloured box around something',
			shortcut: 'H'
		}
	];
</script>

<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Annotation, Rect } from '$lib/types';
	import { cn } from '$lib/utils';
	import {
		annotationClasses,
		annotationStyle,
		defaultColor,
		draftStyle
	} from './annotation-styles';

	let {
		annotations,
		tool,
		color,
		stroke = 'medium',
		selected = $bindable<string | null>(null),
		onChange,
		class: className = ''
	}: {
		annotations: Annotation[];
		tool: Tool;
		/** Colour applied to newly drawn highlight / cover regions. */
		color?: string;
		stroke?: AnnotationStroke;
		selected?: string | null;
		onChange: (annotations: Annotation[]) => void;
		class?: string;
	} = $props();

	const MIN_SIZE = 0.012;

	let surfaceRef = $state<HTMLDivElement | null>(null);
	let draft = $state<Rect | null>(null);
	let origin = $state<{ x: number; y: number } | null>(null);

	$effect(() => {
		if (tool !== 'select') selected = null;
	});

	$effect(() => {
		if (!selected) return;
		const onKey = (event: KeyboardEvent) => {
			if (event.key !== 'Backspace' && event.key !== 'Delete') return;
			const target = event.target as HTMLElement | null;
			if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA') return;
			event.preventDefault();
			onChange(annotations.filter((a) => a.id !== selected));
			selected = null;
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	function toLocal(event: PointerEvent) {
		const box = surfaceRef!.getBoundingClientRect();
		return {
			x: Math.min(1, Math.max(0, (event.clientX - box.left) / box.width)),
			y: Math.min(1, Math.max(0, (event.clientY - box.top) / box.height))
		};
	}

	function toStyle(rect: Rect) {
		return `left:${rect.x * 100}%;top:${rect.y * 100}%;width:${rect.w * 100}%;height:${rect.h * 100}%`;
	}

	function onPointerDown(event: PointerEvent) {
		if (tool === 'select') {
			selected = null;
			return;
		}
		event.preventDefault();
		(event.target as HTMLElement).setPointerCapture(event.pointerId);
		origin = toLocal(event);
		draft = { ...origin, w: 0, h: 0 };
	}

	function onPointerMove(event: PointerEvent) {
		if (!origin) return;
		const point = toLocal(event);
		draft = {
			x: Math.min(origin.x, point.x),
			y: Math.min(origin.y, point.y),
			w: Math.abs(point.x - origin.x),
			h: Math.abs(point.y - origin.y)
		};
	}

	function onPointerUp() {
		const rect = draft;
		origin = null;
		draft = null;
		if (!rect || tool === 'select') return;
		if (rect.w < MIN_SIZE || rect.h < MIN_SIZE) return;

		const kind = tool;
		const next: Annotation = {
			id: crypto.randomUUID().slice(0, 12),
			kind,
			rect
		};
		const fill = color ?? defaultColor(kind);
		if (fill) next.color = fill;
		if (kind === 'highlight') next.stroke = stroke;

		onChange([...annotations, next]);
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	bind:this={surfaceRef}
	onpointerdown={onPointerDown}
	onpointermove={onPointerMove}
	onpointerup={onPointerUp}
	onpointercancel={onPointerUp}
	class={cn(
		'absolute inset-0',
		tool === 'select' ? 'pointer-events-none' : 'cursor-crosshair',
		className
	)}
>
	{#each annotations as annotation (annotation.id)}
		{@const isSelected = selected === annotation.id}
		{@const interactive = tool === 'select'}
		{#if interactive}
			<div
				role="button"
				tabindex="0"
				onpointerdown={(event) => {
					event.stopPropagation();
					selected = annotation.id;
				}}
				style="{toStyle(annotation.rect)};{annotationStyle(annotation) ?? ''}"
				class={cn(
					annotationClasses(annotation, isSelected),
					'pointer-events-auto cursor-pointer'
				)}
			>
				<button
					type="button"
					onpointerdown={(event) => {
						event.stopPropagation();
						onChange(annotations.filter((a) => a.id !== annotation.id));
						selected = null;
					}}
					aria-label="Remove this annotation"
					class="absolute -top-2 -right-2 grid size-5 place-items-center rounded-full bg-red-500 text-white opacity-0 shadow-md transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
				>
					<Icon icon="lucide:trash-2" class="size-2.5" aria-hidden="true" />
				</button>
			</div>
		{:else}
			<div
				aria-hidden="true"
				style="{toStyle(annotation.rect)};{annotationStyle(annotation) ?? ''}"
				class={cn(annotationClasses(annotation, isSelected), 'pointer-events-none')}
			></div>
		{/if}
	{/each}

	{#if draft && tool !== 'select'}
		<div
			class={cn(
				'pointer-events-none absolute rounded-[3px] border-2',
				tool === 'blur' && 'border-(--text)/60 bg-(--text)/15',
				tool === 'highlight' && 'border-(--text)/60',
				tool === 'redact' && 'border-(--text)/40'
			)}
			style="{toStyle(draft)};{draftStyle(tool, color) ?? ''}"
		></div>
	{/if}
</div>
