<script lang="ts">
	import Icon from "@iconify/svelte";
	import { store } from "$lib/store.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Label } from "$lib/components/ui/label";
	import { Slider } from "$lib/components/ui/slider";
	import { Tabs, TabsContent, TabsList, TabsTrigger } from "$lib/components/ui/tabs";
	import {
		Dialog,
		DialogContent,
		DialogFooter,
		DialogHeader,
		DialogTitle,
	} from "$lib/components/ui/dialog";
	import { ToggleGroup, ToggleGroupItem } from "$lib/components/ui/toggle-group";
	import ModelPicker from "$lib/components/ModelPicker.svelte";
	import RecordingPreferences from "$lib/components/RecordingPreferences.svelte";
	import WritingPreferences from "$lib/components/WritingPreferences.svelte";

	let { open = false, onClose }: { open?: boolean; onClose?: () => void } = $props();

	function handleOpenChange(value: boolean) {
		if (!value) onClose?.();
	}

	let tab = $state("model");

	$effect(() => {
		if (open) tab = store.settingsTab;
	});
</script>

<Dialog {open} onOpenChange={handleOpenChange}>
	<DialogContent class="flex h-[min(680px,86vh)] w-full max-w-3xl flex-col gap-0 overflow-hidden p-0">
		<DialogHeader class="shrink-0 border-b border-(--text)/8 px-6 py-5">
			<DialogTitle>Settings</DialogTitle>
		</DialogHeader>

		<Tabs bind:value={tab} class="flex min-h-0 flex-1 flex-row overflow-hidden">
			<TabsList
				class="h-full w-[168px] shrink-0 flex-col items-stretch gap-0.5 overflow-y-auto rounded-none border-r border-(--text)/8 bg-transparent p-3"
			>
				<TabsTrigger value="model" class="w-full justify-start gap-2 rounded-2xl px-3 py-2">
					<Icon icon="lucide:sparkles" class="size-4 shrink-0 opacity-60" />
					Model
				</TabsTrigger>
				<TabsTrigger value="writing" class="w-full justify-start gap-2 rounded-2xl px-3 py-2">
					<Icon icon="lucide:pen-line" class="size-4 shrink-0 opacity-60" />
					Writing
				</TabsTrigger>
				<TabsTrigger value="recording" class="w-full justify-start gap-2 rounded-2xl px-3 py-2">
					<Icon icon="lucide:circle-dot" class="size-4 shrink-0 opacity-60" />
					Recording
				</TabsTrigger>
				<TabsTrigger value="appearance" class="w-full justify-start gap-2 rounded-2xl px-3 py-2">
					<Icon icon="lucide:palette" class="size-4 shrink-0 opacity-60" />
					Appearance
				</TabsTrigger>
			</TabsList>

			<div class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
				<div class="min-h-0 flex-1 overflow-y-auto px-6 py-5">
					<TabsContent value="model" class="mt-0 space-y-4 pb-2">
						<ModelPicker />
						<div class="space-y-1.5 border-t border-(--text)/8 pt-4">
							<Label>Concurrency — {store.settings.concurrency}</Label>
							<p class="text-xs text-(--text)/56">
								How many steps the model writes at once. Lower this if requests time out.
							</p>
							<Slider
								type="single"
								value={store.settings.concurrency}
								min={1}
								max={8}
								step={1}
								disabled={store.settings.provider === "ollama"}
								onValueChange={(v: number) => void store.updateSettings({ concurrency: v })}
							/>
						</div>
					</TabsContent>

					<TabsContent value="writing" class="mt-0 pb-4">
						<WritingPreferences />
					</TabsContent>

					<TabsContent value="recording" class="mt-0 pb-4">
						<RecordingPreferences />
					</TabsContent>

					<TabsContent value="appearance" class="mt-0 space-y-4 pb-4">
						<div class="space-y-1.5">
							<Label>Theme</Label>
							<ToggleGroup
								type="single"
								value={store.settings.theme}
								onValueChange={(v) =>
									v && void store.updateSettings({ theme: v as typeof store.settings.theme })}
							>
								<ToggleGroupItem value="system">System</ToggleGroupItem>
								<ToggleGroupItem value="light">Light</ToggleGroupItem>
								<ToggleGroupItem value="dark">Dark</ToggleGroupItem>
							</ToggleGroup>
						</div>
					</TabsContent>
				</div>
			</div>
		</Tabs>

		<DialogFooter class="shrink-0 border-t border-(--text)/8 px-6 py-4">
			<Button onclick={() => onClose?.()}>Done</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
