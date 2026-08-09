<script lang="ts">
	import Icon from "@iconify/svelte";
	import { store } from "$lib/store.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuLabel,
		DropdownMenuSeparator,
		DropdownMenuTrigger,
	} from "$lib/components/ui/dropdown-menu";
	import {
		isPresetLanguage,
		languageOptions,
		languageShortLabel,
		OTHER_LANGUAGE,
	} from "$lib/languages";
	import { LIMITS } from "$lib/limits";
	import { cn } from "$lib/utils";

	let { disabled = false, class: className = "" }: { disabled?: boolean; class?: string } =
		$props();

	const options = languageOptions();
	const label = $derived(languageShortLabel(store.writeLanguage));
	const showCustom = $derived(!isPresetLanguage(store.writeLanguage));
	let customDraft = $state("");

	$effect(() => {
		if (showCustom) customDraft = store.writeLanguage;
	});

	function select(language: string) {
		if (language === OTHER_LANGUAGE) {
			void store.selectWriteLanguage(customDraft.trim() || store.settings.language);
			return;
		}
		void store.selectWriteLanguage(language);
	}

	function commitCustom() {
		const value = customDraft.trim();
		if (!value) return;
		void store.selectWriteLanguage(value);
	}
</script>

<DropdownMenu>
	<DropdownMenuTrigger>
		{#snippet child({ props })}
			<Button
				{...props}
				size="sm"
				variant="ghost"
				{disabled}
				class={cn("max-w-[160px] justify-start gap-1.5 px-2.5 font-normal", className)}
				title="Language for writing"
			>
				<Icon icon="lucide:languages" class="size-3.5 shrink-0 opacity-60" />
				<span class="truncate text-xs">{label}</span>
				<Icon icon="lucide:chevron-down" class="ms-auto size-3 shrink-0 opacity-50" />
			</Button>
		{/snippet}
	</DropdownMenuTrigger>
	<DropdownMenuContent align="end" class="max-h-72 w-56 overflow-y-auto">
		<DropdownMenuLabel class="font-normal text-(--text)/56">
			Write in this language
		</DropdownMenuLabel>
		{#each options as option (option)}
			{#if option === OTHER_LANGUAGE}
				<DropdownMenuSeparator />
				<div class="space-y-2 px-2 py-1.5">
					<p class="text-xs text-(--text)/56">Other language</p>
					<div class="flex gap-2">
						<Input
							bind:value={customDraft}
							maxlength={LIMITS.language}
							placeholder="e.g. Norwegian"
							class="h-8 text-xs"
							onkeydown={(e) => {
								if (e.key === "Enter") commitCustom();
							}}
						/>
						<Button size="sm" variant="ghost" class="shrink-0" onclick={commitCustom}>
							Use
						</Button>
					</div>
				</div>
			{:else}
				<DropdownMenuItem
					class={cn(store.writeLanguage === option && "bg-(--text)/8")}
					onclick={() => select(option)}
				>
					<span class="truncate">{option}</span>
				</DropdownMenuItem>
			{/if}
		{/each}
	</DropdownMenuContent>
</DropdownMenu>
