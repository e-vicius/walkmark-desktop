<script lang="ts">
	import { onMount } from "svelte";
	import { ModeWatcher } from "mode-watcher";
	import Icon from "@iconify/svelte";

	import { connectEvents } from "$lib/events";
	import { isHudWindow } from "$lib/window";
	import { store } from "$lib/store.svelte";
	import { Toaster } from "$lib/components/ui/sonner";
	import AppBootSkeleton from "$lib/components/skeletons/AppBootSkeleton.svelte";
	import { Button } from "$lib/components/ui/button";

	import TopBar from "$lib/components/TopBar.svelte";
	import Library from "$lib/components/Library.svelte";
	import Editor from "$lib/components/editor/Editor.svelte";
	import SourcePicker from "$lib/components/SourcePicker.svelte";
	import SettingsDialog from "$lib/components/SettingsDialog.svelte";
	import ExportDialog from "$lib/components/ExportDialog.svelte";
	import Onboarding from "$lib/components/Onboarding.svelte";
	import RecordingHud from "$lib/components/RecordingHud.svelte";
	import CountdownOverlay from "$lib/components/CountdownOverlay.svelte";
	import EditorSkeleton from "$lib/components/skeletons/EditorSkeleton.svelte";

	const isHud = isHudWindow();
	const close = () => store.closeDialog();

	let connected = $state(false);
	let bootError = $state<string | null>(null);

	onMount(() => {
		let dispose: (() => void) | undefined;
		let cancelled = false;

		void (async () => {
			try {
				const off = await connectEvents();
				if (cancelled) {
					off();
					return;
				}
				dispose = off;
				connected = true;
				await store.boot();
			} catch (error) {
				if (cancelled) return;
				bootError =
					error instanceof Error ? error.message : "Walkmark could not connect to the app.";
				connected = true;
				store.ready = true;
			}
		})();

		const onKey = (event: KeyboardEvent) => {
			const target = event.target as HTMLElement | null;
			const typing =
				target?.tagName === "INPUT" ||
				target?.tagName === "TEXTAREA" ||
				target?.isContentEditable;
			const mod = event.metaKey || event.ctrlKey;
			if (mod && event.key.toLowerCase() === "r" && !event.shiftKey) {
				event.preventDefault();
				if (!store.recording) store.setDialog("sources");
			} else if (mod && event.key.toLowerCase() === "e") {
				event.preventDefault();
				if (store.project) store.setDialog("export");
			} else if (mod && event.key === ",") {
				event.preventDefault();
				store.setDialog("settings");
			} else if (!typing && event.key === "Escape" && store.selection.length > 0) {
				store.clearSelection();
			}
		};
		window.addEventListener("keydown", onKey);
		return () => {
			cancelled = true;
			dispose?.();
			window.removeEventListener("keydown", onKey);
		};
	});
</script>

<ModeWatcher />

{#if isHud}
	<RecordingHud />
{:else if !store.ready || !connected}
	<AppBootSkeleton />
{:else if bootError}
	<div class="grid h-full place-items-center p-8">
		<div class="max-w-md rounded-3xl bg-(--bg-elevated) p-8 text-center">
			<Icon icon="lucide:triangle-alert" class="mx-auto mb-4 size-8 text-(--text)/40" />
			<h1 class="font-semibold">Walkmark couldn't start</h1>
			<p class="mt-2 text-sm text-(--text)/56">{bootError}</p>
			<Button class="mt-6" onclick={() => location.reload()}>Reload</Button>
		</div>
	</div>
{:else}
	<div class="flex h-full flex-col">
		<TopBar />
		<main class="min-h-0 flex-1 overflow-hidden">
			{#if store.route === "library"}
				<Library />
			{:else if store.projectLoading}
				<EditorSkeleton />
			{:else}
				<Editor />
			{/if}
		</main>

		<SourcePicker open={store.dialog === "sources"} onClose={close} />
		<SettingsDialog open={store.dialog === "settings"} onClose={close} />
		<ExportDialog open={store.dialog === "export"} onClose={close} />
		<Onboarding open={store.dialog === "onboarding"} onClose={close} />

		{#if store.recording?.state === "counting"}
			<CountdownOverlay seconds={store.recording.countdown} />
		{/if}
		<Toaster position="bottom-right" />
	</div>
{/if}
