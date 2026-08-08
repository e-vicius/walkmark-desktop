<script lang="ts">
	import Icon from "@iconify/svelte";
	import { store } from "$lib/store.svelte";
	import { Button } from "$lib/components/ui/button";
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuLabel,
		DropdownMenuSeparator,
		DropdownMenuTrigger,
	} from "$lib/components/ui/dropdown-menu";
	import { cn } from "$lib/utils";

	let { disabled = false, class: className = "" }: { disabled?: boolean; class?: string } =
		$props();

	const options = $derived(store.writeOptions);
	const groups = $derived([...new Set(options.map((o) => o.group))]);
	const selected = $derived(
		options.find((o) => o.provider === store.writeProvider && o.model === store.writeModel),
	);
</script>

<DropdownMenu>
	<DropdownMenuTrigger>
		{#snippet child({ props })}
			<Button
				{...props}
				size="sm"
				variant="ghost"
				{disabled}
				class={cn("max-w-[220px] justify-start gap-1.5 px-2.5 font-normal", className)}
				title="Model for writing"
			>
				<Icon icon="lucide:sparkles" class="size-3.5 shrink-0 opacity-60" />
				<span class="truncate text-xs">{store.writeSelectionLabel}</span>
				<Icon icon="lucide:chevron-down" class="ms-auto size-3 shrink-0 opacity-50" />
			</Button>
		{/snippet}
	</DropdownMenuTrigger>
	<DropdownMenuContent align="end" class="max-h-72 w-64 overflow-y-auto">
		{#if options.length === 0}
			<DropdownMenuLabel class="font-normal text-(--text)/56">
				No models ready yet. Add an API key or start Ollama in Settings.
			</DropdownMenuLabel>
		{:else}
			{#each groups as group (group)}
				<DropdownMenuLabel>{group}</DropdownMenuLabel>
				{#each options.filter((o) => o.group === group) as option (`${option.provider}:${option.model}`)}
					<DropdownMenuItem
						class={cn(selected === option && "bg-(--text)/8")}
						onclick={() => store.selectWriteModel(option.provider, option.model)}
					>
						<span class="truncate">{option.label}</span>
					</DropdownMenuItem>
				{/each}
			{/each}
		{/if}
		<DropdownMenuSeparator />
		<DropdownMenuItem onclick={() => store.openSettings("model")}>
			Configure providers…
		</DropdownMenuItem>
	</DropdownMenuContent>
</DropdownMenu>
