<script lang="ts">
	import { Switch as SwitchPrimitive } from 'bits-ui';
	import { cn, type WithoutChildrenOrChild } from '$lib/utils.js';

	let {
		ref = $bindable(null),
		class: className,
		checked = $bindable(false),
		...restProps
	}: WithoutChildrenOrChild<SwitchPrimitive.RootProps> = $props();
</script>

<SwitchPrimitive.Root
	bind:ref
	bind:checked
	data-slot="switch"
	class={cn(
		'peer relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full p-0.5 outline-none transition-colors duration-150 data-[state=checked]:bg-(--text) data-[state=unchecked]:bg-(--text)/16 data-disabled:cursor-not-allowed data-disabled:opacity-50',
		className
	)}
	{...restProps}
>
	<SwitchPrimitive.Thumb
		data-slot="switch-thumb"
		class="pointer-events-none absolute top-0.5 left-0.5 block size-5 rounded-full bg-(--bg-elevated) shadow-sm transition-[left] duration-150 data-[state=checked]:left-[1.375rem] data-[state=unchecked]:left-0.5"
	/>
</SwitchPrimitive.Root>
