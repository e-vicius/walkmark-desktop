<script lang="ts">
	import Icon from "@iconify/svelte";
	import { store } from "$lib/store.svelte";
	import { Button } from "$lib/components/ui/button";
	import ModelPicker from "$lib/components/ModelPicker.svelte";
	import {
		Dialog,
		DialogContent,
		DialogFooter,
		DialogHeader,
		DialogTitle,
		DialogDescription,
	} from "$lib/components/ui/dialog";
	import * as api from "$lib/api";

	let { open = false, onClose }: { open?: boolean; onClose?: () => void } = $props();

	function handleOpenChange(value: boolean) {
		if (!value) onClose?.();
	}

	function finish() {
		if (!store.settings.onboarded) void store.updateSettings({ onboarded: true });
		store.setDialog(null);
	}

	let primed = false;

	$effect(() => {
		if (open && !primed) {
			primed = true;
			if (
				!store.settings.onboarded &&
				store.catalog.configured.length === 0 &&
				store.settings.provider === "ollama"
			) {
				void store.updateSettings({ provider: "gemini" });
			}
		}
		if (!open) primed = false;
	});
</script>

<Dialog {open} onOpenChange={handleOpenChange}>
	<DialogContent class="flex h-[min(680px,86vh)] w-full max-w-3xl flex-col gap-0 overflow-hidden p-0">
		<DialogHeader class="border-b border-(--text)/8 px-6 py-5">
			<DialogTitle>Welcome to Steppy</DialogTitle>
			<DialogDescription>
				Start with Google Gemini, OpenAI, Claude, or OpenRouter — each has several models to
				choose from. Local models need Ollama installed.
			</DialogDescription>
		</DialogHeader>

		<div class="min-h-0 flex-1 space-y-4 overflow-y-auto px-6 py-5">
			{#if store.permission.required && !store.permission.granted}
				<div class="rounded-3xl bg-(--bg) p-4">
					<p class="text-sm font-medium text-(--text)">Screen recording access</p>
					<p class="mt-1 text-xs text-(--text)/56">
						macOS needs permission before Steppy can capture your screen.
					</p>
					<div class="mt-3 flex gap-2">
						<Button
							size="sm"
							onclick={() => void api.requestPermission().then(() => store.refreshPermission())}
						>
							Grant access
						</Button>
						<Button size="sm" variant="ghost" onclick={() => void api.openPrivacySettings()}>
							System Settings
						</Button>
					</div>
				</div>
			{/if}

			<div class="space-y-3">
				<div>
					<p class="text-sm font-medium text-(--text)">Writing model</p>
					<p class="mt-1 text-xs text-(--text)/56">
						{#if store.canWrite}
							Ready with {store.activeProvider?.name ?? "your provider"} · {store.activeConfig.model ||
								store.activeProvider?.defaultModel}.
						{:else if store.settings.provider === "ollama"}
							Ollama is for offline models. Pick Google Gemini, OpenAI, Claude, or OpenRouter
							above for cloud models — Gemini has a generous free tier.
						{:else if store.blockedReason}
							{store.blockedReason}
						{:else}
							Pick a provider, choose a model, then paste your API key below.
						{/if}
					</p>
				</div>

				<ModelPicker />
			</div>
		</div>

		<DialogFooter class="border-t border-(--text)/8 px-6 py-4">
			<Button variant="ghost" onclick={finish}>
				{store.settings.onboarded ? "Close" : "Skip for now"}
			</Button>
			<Button
				onclick={() => {
					if (!store.settings.onboarded) void store.updateSettings({ onboarded: true });
					store.setDialog("sources");
				}}
			>
				<Icon icon="lucide:circle-dot" class="size-4" />
				Record something
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
