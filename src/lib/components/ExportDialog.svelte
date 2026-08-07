<script lang="ts">
	import Icon from '@iconify/svelte';
	import { save } from '@tauri-apps/plugin-dialog';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { openPath } from '@tauri-apps/plugin-opener';
	import * as api from '$lib/api';
	import { fileSize, pluralize } from '$lib/format';
	import { store } from '$lib/store.svelte';
	import type { ExportFormat, ExportOptions } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle
	} from '$lib/components/ui/dialog';
	import { Label } from '$lib/components/ui/label';
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger
	} from '$lib/components/ui/select';
	import { Switch } from '$lib/components/ui/switch';
	import { ToggleGroup, ToggleGroupItem } from '$lib/components/ui/toggle-group';
	import { Spinner } from '$lib/components/ui/spinner';
	import { cn } from '$lib/utils';

	let { open = false, onClose }: { open?: boolean; onClose?: () => void } = $props();

	function handleOpenChange(value: boolean) {
		if (!value && exporting) return;
		if (!value) onClose?.();
	}

	const FORMATS: Record<
		ExportFormat,
		{ label: string; icon: string; extension: string; blurb: string }
	> = {
		markdown: {
			label: 'Markdown',
			icon: 'lucide:file-text',
			extension: 'md',
			blurb:
				'A .md file plus a folder of images beside it. Drops straight into a repo, a wiki or a static site.'
		},
		html: {
			label: 'Web page',
			icon: 'lucide:file-code-2',
			extension: 'html',
			blurb:
				'One self-contained file with the images embedded. Email it, host it, open it offline — it just works.'
		},
		pdf: {
			label: 'PDF',
			icon: 'lucide:file-type-2',
			extension: 'pdf',
			blurb:
				'A print-ready A4 document with page numbers. Best for handbooks and anything that gets signed off.'
		}
	};

	const IMAGE_WIDTHS = [
		{ value: '900', label: 'Compact — 900px' },
		{ value: '1400', label: 'Standard — 1400px' },
		{ value: '1920', label: 'Large — 1920px' }
	];

	let options = $state<ExportOptions>({
		format: 'html',
		includeImages: true,
		imageWidth: 1400,
		includeToc: true,
		includeSummary: true,
		includePrerequisites: true,
		theme: 'auto'
	});
	let exporting = $state(false);

	const project = $derived(store.project);

	$effect(() => {
		if (!open) exporting = false;
	});

	const included = $derived(project?.steps.filter((s) => s.include) ?? []);
	const unwritten = $derived(included.filter((s) => s.status !== 'ready').length);
	const format = $derived(FORMATS[options.format]);
	const imageWidthStr = $derived(String(options.imageWidth));

	function patch(next: Partial<ExportOptions>) {
		options = { ...options, ...next };
	}

	async function run() {
		if (!project || exporting) return;
		try {
			const suggested = await api.suggestExportName(options.format);
			const destination = await save({
				defaultPath: suggested,
				filters: [{ name: format.label, extensions: [format.extension] }]
			});
			if (!destination) return;

			exporting = true;
			const result = await api.exportDocument(options, destination);
			onClose?.();
			store.notify({
				tone: 'success',
				title: `Exported as ${format.label}`,
				detail: `${result.path.split('/').pop()} · ${fileSize(result.bytes)}${
					result.assetsDir ? ' (plus an images folder)' : ''
				}`,
				action: {
					label: 'Open',
					run: () => void openPath(result.path).catch(() => {})
				}
			});
		} catch (error) {
			store.reportError(error, 'The export failed');
		} finally {
			exporting = false;
		}
	}

	async function copy() {
		try {
			await writeText(await api.copyAsMarkdown());
			store.notify({ tone: 'success', title: 'Markdown copied to the clipboard' });
		} catch (error) {
			store.reportError(error, 'Nothing could be copied');
		}
	}
</script>

{#if project}
	<Dialog {open} onOpenChange={handleOpenChange}>
		<DialogContent class="relative max-w-[760px] overflow-hidden">
			{#if exporting}
				<div
					class="absolute inset-0 z-20 flex flex-col items-center justify-center gap-3 bg-(--bg-elevated)/90 backdrop-blur-sm"
					aria-live="polite"
					aria-busy="true"
				>
					<Spinner class="size-5" />
					<p class="text-sm font-medium text-(--text)">Saving {format.label}…</p>
					<p class="text-xs text-(--text)/56">This may take a moment for large documents.</p>
				</div>
			{/if}

			<DialogHeader>
				<DialogTitle>Export this document</DialogTitle>
				<DialogDescription>
					{pluralize(included.length, 'step')} will be included.
				</DialogDescription>
			</DialogHeader>

			{#if unwritten > 0}
				<div class="mb-5 rounded-2xl bg-amber-500/10 px-3.5 py-2.5">
					<p class="text-[12.5px] leading-relaxed text-amber-700 dark:text-amber-400">
						{pluralize(unwritten, 'step has', 'steps have')} no instructions yet. They'll appear
						with a placeholder title and just the screenshot.
					</p>
				</div>
			{/if}

			<div class="grid gap-2 sm:grid-cols-3">
				{#each Object.keys(FORMATS) as key (key)}
					{@const entry = FORMATS[key as ExportFormat]}
					{@const active = options.format === key}
					<button
						type="button"
						onclick={() => patch({ format: key as ExportFormat })}
						aria-pressed={active}
						class={cn(
							'rounded-2xl p-3.5 text-left transition-colors',
							active ? 'bg-(--text)/12' : 'bg-(--text)/5 hover:bg-(--text)/8'
						)}
					>
						<Icon
							icon={entry.icon}
							class={cn('mb-2 size-4', active ? 'text-(--text)' : 'text-(--text)/56')}
						/>
						<p class="text-[13px] font-medium">{entry.label}</p>
						<p class="mt-0.5 text-[11.5px] tracking-wide text-(--text)/40 uppercase">
							.{entry.extension}
						</p>
					</button>
				{/each}
			</div>

			<p class="mt-3 text-[12.5px] leading-relaxed text-(--text)/56">{format.blurb}</p>

			<div class="mt-6 flex flex-col gap-4 border-t border-(--text)/8 pt-5">
				<div class="flex items-center justify-between gap-4">
					<div>
						<Label>Include screenshots</Label>
						<p class="text-xs text-(--text)/56">Turn off for a text-only checklist.</p>
					</div>
					<Switch
						checked={options.includeImages}
						onCheckedChange={(includeImages) => patch({ includeImages })}
					/>
				</div>

				{#if options.includeImages}
					<div class="flex flex-col gap-2">
						<Label>Screenshot size</Label>
						<p class="text-xs text-(--text)/56">
							Larger images keep small UI text readable but make the file bigger.
						</p>
						<Select
							type="single"
							value={imageWidthStr}
							onValueChange={(v) => v && patch({ imageWidth: Number(v) })}
						>
							<SelectTrigger class="w-full">
								{IMAGE_WIDTHS.find((w) => w.value === imageWidthStr)?.label ?? 'Select size'}
							</SelectTrigger>
							<SelectContent>
								{#each IMAGE_WIDTHS as option (option.value)}
									<SelectItem value={option.value} label={option.label} />
								{/each}
							</SelectContent>
						</Select>
					</div>
				{/if}

				<div class="flex items-center justify-between gap-4">
					<Label>Include the summary</Label>
					<Switch
						checked={options.includeSummary}
						onCheckedChange={(includeSummary) => patch({ includeSummary })}
					/>
				</div>

				<div class="flex items-center justify-between gap-4">
					<Label>Include “Before you start”</Label>
					<Switch
						checked={options.includePrerequisites}
						onCheckedChange={(includePrerequisites) => patch({ includePrerequisites })}
					/>
				</div>

				<div class="flex items-center justify-between gap-4">
					<div>
						<Label>Include a table of contents</Label>
						<p class="text-xs text-(--text)/56">Only added when there are more than two steps.</p>
					</div>
					<Switch
						checked={options.includeToc}
						onCheckedChange={(includeToc) => patch({ includeToc })}
					/>
				</div>

				{#if options.format === 'html'}
					<div class="flex flex-col gap-2">
						<Label>Colour scheme</Label>
						<ToggleGroup
							value={options.theme}
							onValueChange={(theme) => theme && patch({ theme: theme as ExportOptions['theme'] })}
							type="single"
							variant="outline"
							class="w-full"
						>
							<ToggleGroupItem value="auto" class="flex-1">Match the reader</ToggleGroupItem>
							<ToggleGroupItem value="light" class="flex-1">Light</ToggleGroupItem>
							<ToggleGroupItem value="dark" class="flex-1">Dark</ToggleGroupItem>
						</ToggleGroup>
					</div>
				{/if}
			</div>

			<DialogFooter class="sm:justify-between">
				<Button variant="ghost" class="mr-auto" disabled={exporting} onclick={() => void copy()}>
					<Icon icon="lucide:copy" class="size-3.5" />
					Copy as Markdown
				</Button>
				<Button variant="ghost" disabled={exporting} onclick={() => onClose?.()}>Cancel</Button>
				<Button disabled={included.length === 0 || exporting} onclick={() => void run()}>
					{#if exporting}
						<Spinner class="size-4" />
						Saving…
					{:else}
						<Icon icon="lucide:download" class="size-4" />
						Choose location…
					{/if}
				</Button>
			</DialogFooter>
		</DialogContent>
	</Dialog>
{/if}
