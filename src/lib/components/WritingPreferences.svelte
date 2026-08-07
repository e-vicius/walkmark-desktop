<script lang="ts">
	import type { Tone } from "$lib/types";
	import { languageOptions } from "$lib/languages";
	import { store } from "$lib/store.svelte";
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

	const settings = $derived(store.settings);
	const languages = $derived(languageOptions(settings.language));

	function patch(next: Partial<typeof settings>) {
		void store.updateSettings(next);
	}
</script>

<div class="space-y-6">
	<p class="text-sm text-(--text)/56">
		Default voice and audience for new recordings. Vocabulary is grouped by product — Steppy
		uses it when writing step instructions.
	</p>

	<div class="space-y-1.5">
		<Label>Who is this for?</Label>
		<Input
			value={settings.audience}
			placeholder="e.g. a new teammate who has never used this app"
			oninput={(e) => patch({ audience: (e.currentTarget as HTMLInputElement).value })}
		/>
	</div>

	<div class="space-y-1.5">
		<Label>Voice</Label>
		<ToggleGroup
			type="single"
			value={settings.tone}
			onValueChange={(tone) => tone && patch({ tone: tone as Tone })}
		>
			<ToggleGroupItem value="neutral">Neutral</ToggleGroupItem>
			<ToggleGroupItem value="friendly">Friendly</ToggleGroupItem>
			<ToggleGroupItem value="formal">Formal</ToggleGroupItem>
			<ToggleGroupItem value="playful">Playful</ToggleGroupItem>
		</ToggleGroup>
	</div>

	<div class="space-y-1.5">
		<Label>Language</Label>
		<Select
			type="single"
			value={settings.language}
			onValueChange={(language) => language && patch({ language })}
		>
			<SelectTrigger class="w-full">
				{settings.language || "Select language"}
			</SelectTrigger>
			<SelectContent>
				{#each languages as option (option)}
					<SelectItem value={option} label={option} />
				{/each}
			</SelectContent>
		</Select>
	</div>

	<div class="space-y-1.5 border-t border-(--text)/8 pt-5">
		<Label>Default product</Label>
		<p class="text-xs text-(--text)/56">
			Pre-selected when you start a new recording. You can still pick a different one each time.
		</p>
		<Select
			type="single"
			value={settings.defaultProductId ?? settings.products[0]?.id ?? ""}
			onValueChange={(id) => id && patch({ defaultProductId: id })}
		>
			<SelectTrigger class="w-full">
				{settings.products.find((p) => p.id === settings.defaultProductId)?.name ??
					settings.products[0]?.name ??
					"Select product"}
			</SelectTrigger>
			<SelectContent>
				{#each settings.products as product (product.id)}
					<SelectItem value={product.id} label={product.name} />
				{/each}
			</SelectContent>
		</Select>
	</div>

	<div class="border-t border-(--text)/8 pt-5">
		<ProductsEditor />
	</div>
</div>
