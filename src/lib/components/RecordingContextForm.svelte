<script lang="ts">
	import Icon from "@iconify/svelte";
	import type { Tone } from "$lib/types";
	import { languageOptions } from "$lib/languages";
	import { store } from "$lib/store.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
	} from "$lib/components/ui/select";
	import { ToggleGroup, ToggleGroupItem } from "$lib/components/ui/toggle-group";
	import ProductsEditor from "$lib/components/ProductsEditor.svelte";

	let {
		selectedProduct = $bindable<string | null>(null),
		audience = $bindable(""),
		tone = $bindable<Tone>("neutral"),
		language = $bindable("English"),
	}: {
		selectedProduct?: string | null;
		audience?: string;
		tone?: Tone;
		language?: string;
	} = $props();

	const languages = $derived(languageOptions(language));
</script>

<div class="space-y-5">
	<div class="space-y-2">
		<div class="flex items-start justify-between gap-3">
			<div>
				<Label>What are you documenting?</Label>
				<p class="text-xs text-(--text)/56">
					Pick the product this guide is about. Vocabulary is managed in Settings.
				</p>
			</div>
			<Button
				size="sm"
				variant="ghost"
				class="shrink-0"
				onclick={() => store.openSettings("writing", "sources")}
			>
				<Icon icon="lucide:settings-2" class="size-3.5" />
				Edit vocabulary
			</Button>
		</div>
		<ProductsEditor pickOnly bind:selectedId={selectedProduct} />
	</div>

	<div class="space-y-1.5">
		<Label>Who is this for?</Label>
		<Input bind:value={audience} placeholder="e.g. a new teammate who has never used this app" />
	</div>

	<div class="space-y-1.5">
		<Label>Voice</Label>
		<ToggleGroup type="single" bind:value={tone}>
			<ToggleGroupItem value="neutral">Neutral</ToggleGroupItem>
			<ToggleGroupItem value="friendly">Friendly</ToggleGroupItem>
			<ToggleGroupItem value="formal">Formal</ToggleGroupItem>
			<ToggleGroupItem value="playful">Playful</ToggleGroupItem>
		</ToggleGroup>
	</div>

	<div class="space-y-1.5">
		<Label>Language</Label>
		<Select type="single" bind:value={language}>
			<SelectTrigger class="w-full">
				{language || "Select language"}
			</SelectTrigger>
			<SelectContent>
				{#each languages as option (option)}
					<SelectItem value={option} label={option} />
				{/each}
			</SelectContent>
		</Select>
	</div>
</div>
