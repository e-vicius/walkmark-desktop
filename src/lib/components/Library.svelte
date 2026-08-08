<script lang="ts">
	import { onMount } from "svelte";
	import Icon from "@iconify/svelte";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { store } from "$lib/store.svelte";
	import { pluralize, relativeTime } from "$lib/format";
	import { Button } from "$lib/components/ui/button";
	import { Badge } from "$lib/components/ui/badge";
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle,
	} from "$lib/components/ui/dialog";
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuTrigger,
	} from "$lib/components/ui/dropdown-menu";
	import Logo from "$lib/components/Logo.svelte";
	import LibrarySkeleton from "$lib/components/skeletons/LibrarySkeleton.svelte";
	import type { ProjectSummary } from "$lib/types";

	let deleteTarget = $state<ProjectSummary | null>(null);
	let deleting = $state(false);

	onMount(() => {
		void store.refreshProjects();
	});

	async function confirmDelete() {
		if (!deleteTarget || deleting) return;
		deleting = true;
		try {
			await store.removeProject(deleteTarget.id);
			deleteTarget = null;
		} finally {
			deleting = false;
		}
	}
</script>

<div class="h-full overflow-y-auto">
	{#if store.projectsLoading && store.projects.length === 0}
		<LibrarySkeleton />
	{:else}
	<div class="mx-auto w-full max-w-[960px] px-8 py-12">
		{#if store.projects.length === 0}
			<section class="flex flex-col items-center px-6 py-20 text-center">
				<Logo markClass="size-14" />
				<h1 class="mt-6 text-2xl font-semibold tracking-tight text-(--text)">
					Document what you just did
				</h1>
				<p class="mt-2 max-w-md text-sm leading-relaxed text-(--text)/56">
					Steppy watches a screen or window, captures each meaningful step, and writes
					the instructions for you.
				</p>
				<Button class="mt-8" onclick={() => store.setDialog("sources")}>
					<Icon icon="lucide:circle-dot" class="size-4" />
					Record your first guide
				</Button>
			</section>
			{@render notices("mt-6")}
		{:else}
			{@render notices("mb-8")}
			<section>
				<h2 class="mb-4 text-xs font-medium uppercase tracking-wider text-(--text)/40">
					Your documents
				</h2>
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
					{#each store.projects as project (project.id)}
						{@render card(project)}
					{/each}
				</div>
			</section>
		{/if}
	</div>
	{/if}
</div>

<Dialog open={deleteTarget !== null} onOpenChange={(open) => !open && (deleteTarget = null)}>
	<DialogContent class="max-w-md">
		<DialogHeader>
			<DialogTitle>Delete this document?</DialogTitle>
			<DialogDescription>
				{#if deleteTarget}
					“{deleteTarget.title}” and all of its screenshots will be removed permanently.
				{/if}
			</DialogDescription>
		</DialogHeader>
		<DialogFooter>
			<Button variant="ghost" disabled={deleting} onclick={() => (deleteTarget = null)}>
				Cancel
			</Button>
			<Button variant="destructive" disabled={deleting} onclick={() => void confirmDelete()}>
				{deleting ? "Deleting…" : "Delete"}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>

{#snippet notices(className: string)}
	{#if store.blockedReason || (!store.permission.granted && store.permission.required)}
		<div class="flex flex-col gap-2 {className}">
			{#if !store.permission.granted && store.permission.required}
				<div class="flex items-center gap-3 rounded-3xl bg-(--bg-elevated) px-4 py-3">
					<Icon icon="lucide:monitor-play" class="size-4 flex-none text-red-500" />
					<div class="min-w-0 flex-1">
						<p class="text-sm font-medium text-(--text)">Screen recording access is off</p>
						<p class="text-xs text-(--text)/56">macOS needs permission before Steppy can capture.</p>
					</div>
					<Button size="sm" variant="ghost" onclick={() => store.setDialog("onboarding")}>
						Fix this
					</Button>
				</div>
			{/if}
			{#if store.blockedReason}
				<div class="flex items-center gap-3 rounded-3xl bg-(--bg-elevated) px-4 py-3">
					<Icon icon="lucide:key-round" class="size-4 flex-none text-(--text)/56" />
					<div class="min-w-0 flex-1">
						<p class="text-sm font-medium text-(--text)">Select an AI model</p>
						<p class="text-xs text-(--text)/56">{store.blockedReason}</p>
					</div>
					<Button size="sm" variant="ghost" onclick={() => store.setDialog("settings")}>
						Set up
					</Button>
				</div>
			{/if}
		</div>
	{/if}
{/snippet}

{#snippet card(project: ProjectSummary)}
	{@const done = project.stepCount > 0 && project.readyCount === project.stepCount}
	<article
		class="group relative overflow-hidden rounded-3xl bg-(--bg-elevated) transition-transform hover:-translate-y-0.5"
	>
		<button
			type="button"
			class="block w-full text-left"
			onclick={() => void store.openProject(project.id)}
		>
			<div class="relative aspect-video overflow-hidden bg-(--bg)">
				{#if project.cover}
					<img
						src={convertFileSrc(project.cover)}
						alt=""
						draggable="false"
						class="size-full object-cover object-top transition-transform duration-500 group-hover:scale-[1.02]"
						loading="lazy"
					/>
				{:else}
					<div class="grid size-full place-items-center text-(--text)/24">
						<Icon icon="lucide:file-text" class="size-6" />
					</div>
				{/if}
			</div>
			<div class="p-4">
				<h3 class="truncate text-sm font-medium text-(--text)">{project.title}</h3>
				<p class="mt-1 flex flex-wrap items-center gap-1.5 text-xs text-(--text)/56">
					<span>{pluralize(project.stepCount, "step")}</span>
					<span>·</span>
					<span>{relativeTime(project.updatedAt)}</span>
					{#if !done && project.stepCount > 0}
						<span>·</span>
						<Badge>{project.stepCount - project.readyCount} unwritten</Badge>
					{/if}
				</p>
			</div>
		</button>

		<div class="absolute top-3 right-3 z-10 opacity-0 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100">
			<DropdownMenu>
				<DropdownMenuTrigger>
					{#snippet child({ props })}
						<button
							{...props}
							type="button"
							class="grid size-8 place-items-center rounded-xl bg-(--bg-elevated)/95 text-(--text)/72 shadow-sm ring-1 ring-(--text)/10 backdrop-blur-sm transition-colors hover:bg-(--bg-elevated) hover:text-(--text)"
							aria-label="Document options for {project.title}"
							onclick={(event) => event.stopPropagation()}
						>
							<Icon icon="lucide:ellipsis" class="size-4" />
						</button>
					{/snippet}
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end" class="w-40">
					<DropdownMenuItem
						variant="destructive"
						onclick={() => (deleteTarget = project)}
					>
						<Icon icon="lucide:trash-2" />
						Delete
					</DropdownMenuItem>
				</DropdownMenuContent>
			</DropdownMenu>
		</div>
	</article>
{/snippet}
