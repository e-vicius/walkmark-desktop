<script lang="ts">
	import Icon from "@iconify/svelte";
	import { store } from "$lib/store.svelte";
	import type { Product } from "$lib/types";
	import { newProduct } from "$lib/products";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import VocabularyEditor from "$lib/components/VocabularyEditor.svelte";
	import { cn } from "$lib/utils";

	let {
		compact = false,
		pickOnly = false,
		selectedId = $bindable<string | null>(null),
	}: {
		compact?: boolean;
		/** Selection list only — for the recording flow. Edit products in Settings. */
		pickOnly?: boolean;
		selectedId?: string | null;
	} = $props();

	const selectable = $derived(pickOnly || compact || selectedId !== null);

	function patchProducts(next: Product[]) {
		void store.updateSettings({ products: next });
	}

	function updateProduct(id: string, patch: Partial<Product>) {
		patchProducts(store.settings.products.map((p) => (p.id === id ? { ...p, ...patch } : p)));
	}

	function addProduct() {
		const product = newProduct(`Product ${store.settings.products.length + 1}`);
		const products = [...store.settings.products, product];
		const defaultProductId = store.settings.defaultProductId ?? product.id;
		selectedId = product.id;
		void store.updateSettings({ products, defaultProductId });
	}

	function removeProduct(id: string) {
		if (store.settings.products.length <= 1) return;
		const products = store.settings.products.filter((p) => p.id !== id);
		const defaultProductId =
			store.settings.defaultProductId === id ? products[0]?.id ?? null : store.settings.defaultProductId;
		if (selectedId === id) selectedId = products[0]?.id ?? null;
		void store.updateSettings({ products, defaultProductId });
	}
</script>

<div class="space-y-3">
	{#if !pickOnly}
		<div class="flex items-center justify-between gap-3">
			{#if !compact}
				<div>
					<p class="text-sm font-medium text-(--text)">Products</p>
					<p class="text-xs text-(--text)/56">
						Each product has its own vocabulary. You pick one when recording.
					</p>
				</div>
			{:else}
				<p class="text-xs text-(--text)/56">Add or edit product vocabulary</p>
			{/if}
			<Button size="sm" variant="ghost" onclick={addProduct}>
				<Icon icon="lucide:plus" class="size-3.5" />
				Add
			</Button>
		</div>
	{/if}

	<div class={cn("space-y-3", pickOnly && "grid gap-2 sm:grid-cols-2")}>
		{#each store.settings.products as product (product.id)}
			<div
				class={cn(
					pickOnly
						? "rounded-2xl p-3 transition-colors"
						: "space-y-2 rounded-2xl p-3 transition-colors",
					selectable && selectedId === product.id
						? "bg-(--text)/8 ring-1 ring-(--text)/12"
						: pickOnly
							? "bg-(--text)/5 hover:bg-(--text)/8"
							: "bg-(--bg)",
				)}
			>
				{#if pickOnly}
					<button
						type="button"
						class="flex w-full items-center gap-2.5 text-left"
						onclick={() => (selectedId = product.id)}
					>
						<span
							class={cn(
								"grid size-5 shrink-0 place-items-center rounded-full border transition-colors",
								selectedId === product.id
									? "border-(--text) bg-(--text) text-(--bg)"
									: "border-(--text)/20",
							)}
						>
							{#if selectedId === product.id}
								<Icon icon="lucide:check" class="size-3" />
							{/if}
						</span>
						<span class="min-w-0 truncate text-sm font-medium">{product.name}</span>
					</button>
				{:else}
				<div class="flex items-center gap-2">
					{#if selectable}
						<button
							type="button"
							class="grid size-8 shrink-0 place-items-center rounded-xl transition-colors {selectedId ===
							product.id
								? 'bg-(--text) text-(--bg)'
								: 'bg-(--text)/5 text-(--text)/40 hover:bg-(--text)/10'}"
							aria-label="Select {product.name}"
							onclick={() => (selectedId = product.id)}
						>
							<Icon icon="lucide:check" class="size-3.5" />
						</button>
					{/if}
					<Input
						value={product.name}
						placeholder="Product name"
						oninput={(e) =>
							updateProduct(product.id, {
								name: (e.currentTarget as HTMLInputElement).value,
							})}
					/>
					{#if store.settings.products.length > 1}
						<button
							type="button"
							class="grid size-8 shrink-0 place-items-center rounded-xl text-(--text)/40 hover:bg-(--text)/5 hover:text-red-500"
							aria-label="Remove {product.name}"
							onclick={() => removeProduct(product.id)}
						>
							<Icon icon="lucide:trash-2" class="size-3.5" />
						</button>
					{/if}
				</div>
				<VocabularyEditor
					compact={compact}
					terms={product.vocabulary}
					onChange={(vocabulary) => updateProduct(product.id, { vocabulary })}
				/>
				{/if}
			</div>
		{/each}
	</div>
</div>
