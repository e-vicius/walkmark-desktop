<script lang="ts">
	import {
		isPresetLanguage,
		languageOptions,
		languageSelectLabel,
		languageSelectValue,
		OTHER_LANGUAGE,
	} from "$lib/languages";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
	} from "$lib/components/ui/select";
	import { LIMITS } from "$lib/limits";

	let { value = $bindable("") }: { value?: string } = $props();

	const options = languageOptions();
	const selectValue = $derived(languageSelectValue(value));
	const showCustom = $derived(selectValue === OTHER_LANGUAGE);

	function onSelect(next: string | undefined) {
		if (!next) return;
		if (next === OTHER_LANGUAGE) {
			value = isPresetLanguage(value) ? "" : value;
		} else {
			value = next;
		}
	}
</script>

<div class="space-y-1.5">
	<Label>Language</Label>
	<Select type="single" value={selectValue} onValueChange={onSelect}>
		<SelectTrigger class="w-full">
			{languageSelectLabel(value)}
		</SelectTrigger>
		<SelectContent>
			{#each options as option (option)}
				<SelectItem value={option} label={option} />
			{/each}
		</SelectContent>
	</Select>
	{#if showCustom}
		<Input bind:value maxlength={LIMITS.language} placeholder="Enter language name" />
	{/if}
</div>
