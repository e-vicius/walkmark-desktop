<script lang="ts">
	import Icon from "@iconify/svelte";
	import { store } from "$lib/store.svelte";
	import { pluralize } from "$lib/format";
	import { isMacOS } from "$lib/window";
	import { Button } from "$lib/components/ui/button";
	import { Spinner } from "$lib/components/ui/spinner";
	import GenerateMenu from "$lib/components/GenerateMenu.svelte";
	import Logo from "$lib/components/Logo.svelte";
	import { cn } from "$lib/utils";
	import { LIMITS } from "$lib/limits";

	const isMac = isMacOS();

	let editing = $state(false);
	let draft = $state("");

	$effect(() => {
		if (!editing && store.project) draft = store.project.title;
	});

	function commitTitle() {
		editing = false;
		const title = draft.trim();
		if (store.project && title && title !== store.project.title) {
			void store.patchMeta({ title });
		} else if (store.project) {
			draft = store.project.title;
		}
	}
</script>

<header
	data-tauri-drag-region
	class={cn(
		"titlebar relative z-30 flex flex-none items-center gap-3 border-b border-(--text)/8 bg-(--bg-elevated) pr-3",
		!isMac && "h-12",
	)}
>
	{#if !isMac}
		<div class="w-3 shrink-0" data-tauri-drag-region aria-hidden="true"></div>
	{/if}

	<div class="no-drag flex min-w-0 flex-1 items-center gap-3">
		{#if store.route === "editor" && store.project}
			{#if !store.recording}
				<button
					type="button"
					class="grid size-8 place-items-center rounded-2xl text-(--text)/56 transition-colors hover:bg-(--text)/5 hover:text-(--text)"
					aria-label="Back to all documents"
					onclick={() => void store.leaveProject()}
				>
					<Icon icon="lucide:chevron-left" class="size-4" />
				</button>
			{/if}
			<div class="flex min-w-0 items-center gap-2">
				{#if editing}
					<input
						class="w-[320px] rounded-xl bg-(--bg) px-2.5 py-1 text-sm font-medium text-(--text) outline-none ring-1 ring-(--text)/20"
						bind:value={draft}
						maxlength={LIMITS.documentTitle}
						onblur={commitTitle}
						onkeydown={(e) => {
							if (e.key === "Enter") commitTitle();
							if (e.key === "Escape") {
								draft = store.project?.title ?? "";
								editing = false;
							}
						}}
					/>
				{:else}
					<button
						class="max-w-[420px] truncate rounded-xl px-2.5 py-1 text-sm font-medium text-(--text) hover:bg-(--text)/5"
						onclick={() => (editing = true)}
					>
						{store.project.title}
					</button>
				{/if}
				<span class="hidden text-xs text-(--text)/40 sm:inline">
					{pluralize(store.project.steps.filter((s) => s.include).length, "step")}
				</span>
			</div>
		{:else}
			<Logo showName inverted markClass={isMac ? "size-6" : "size-7"} />
		{/if}
	</div>

	<div class="no-drag flex items-center gap-2.5">
		{#if store.recording}
			<div class="flex items-center gap-2 rounded-2xl bg-red-500/10 px-3 py-1.5">
				{#if store.stoppingRecording}
					<Spinner class="size-3 border-red-500/30 border-t-red-500 dark:border-red-400/30 dark:border-t-red-400" />
				{:else}
					<span class="relative flex size-2">
						<span class="absolute inline-flex size-full animate-ping rounded-full bg-red-500 opacity-60"></span>
						<span class="relative inline-flex size-2 rounded-full bg-red-500"></span>
					</span>
				{/if}
				<span class="text-xs font-medium tabular-nums text-red-600 dark:text-red-400">
					{store.stoppingRecording
						? "Finishing…"
						: store.recording.state === "paused"
							? "Paused"
							: pluralize(store.recording.stepCount, "step")}
				</span>
				<Button
					size="sm"
					variant="destructive"
					class="ml-1 h-7"
					disabled={store.stoppingRecording}
					onclick={() => void store.endRecording()}
				>
					{#if store.stoppingRecording}
						<Spinner class="size-3 border-white/30 border-t-white" />
					{:else}
						<Icon icon="lucide:square" class="size-3 fill-current" />
					{/if}
					Stop
				</Button>
			</div>
		{:else if store.route === "editor" && store.project}
			{#if store.generation?.running}
				<div class="flex items-center gap-2 rounded-2xl bg-(--text)/5 px-3 py-1.5">
					<Icon icon="lucide:sparkles" class="size-3.5 animate-pulse text-(--text)/72" />
					<span class="text-xs font-medium tabular-nums text-(--text)/72">
						{store.generation.total > 0
							? `${store.generation.done} of ${store.generation.total}`
							: (store.generation.message ?? "Starting")}
					</span>
					<button
						class="grid size-6 place-items-center rounded-xl text-(--text)/56 hover:bg-(--text)/8"
						aria-label="Stop writing"
						onclick={() => void store.stopGeneration()}
					>
						<Icon icon="lucide:x" class="size-3.5" />
					</button>
				</div>
			{/if}
			<GenerateMenu />
			<Button size="sm" variant="ghost" onclick={() => store.setDialog("export")}>
				<Icon icon="lucide:download" class="size-3.5" />
				Export
			</Button>
		{:else}
			<Button size="sm" onclick={() => store.setDialog("sources")}>
				<Icon icon="lucide:circle-dot" class="size-3.5" />
				New recording
			</Button>
		{/if}

		<button
			type="button"
			class="grid size-8 place-items-center rounded-2xl text-(--text)/56 transition-colors hover:bg-(--text)/5 hover:text-(--text)"
			aria-label="Settings"
			onclick={() => store.setDialog("settings")}
		>
			<Icon icon="lucide:settings-2" class="size-4" />
		</button>
	</div>
</header>
