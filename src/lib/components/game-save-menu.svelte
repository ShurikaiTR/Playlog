<script lang="ts">
	import { Button, buttonVariants } from "#lib/components/ui/button/index.js";
	import * as Dialog from "#lib/components/ui/dialog/index.js";
	import * as DropdownMenu from "#lib/components/ui/dropdown-menu/index.js";
	import { Input } from "#lib/components/ui/input/index.js";
	import { Label } from "#lib/components/ui/label/index.js";
	import { cn } from "#lib/utils.js";
	import CheckIcon from "@lucide/svelte/icons/check";
	import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
	import HeartIcon from "@lucide/svelte/icons/heart";
	import ListPlusIcon from "@lucide/svelte/icons/list-plus";
	import PlusIcon from "@lucide/svelte/icons/plus";

	export interface GameList {
		id: string;
		name: string;
		selected: boolean;
	}

	let {
		title,
		status = $bindable(""),
		favorite = $bindable(false),
		lists = $bindable([]),
		compact = false,
		class: className,
	}: {
		title: string;
		status?: string;
		favorite?: boolean;
		lists?: GameList[];
		// A round icon-only trigger for lists of games.
		compact?: boolean;
		class?: string;
	} = $props();

	const saved = $derived(Boolean(status) || favorite || lists.some((l) => l.selected));
	const statuses = ["Oynuyorum", "Bitirdim", "Bıraktım", "Oynamak istiyorum"];
	const itemClass = "h-10 gap-2 rounded-xl px-2 font-medium";

	let dialogOpen = $state(false);
	let name = $state("");
	let error = $state("");

	function openDialog() {
		name = "";
		error = "";
		dialogOpen = true;
	}

	function createList(event: SubmitEvent) {
		event.preventDefault();
		const trimmed = name.trim();
		if (!trimmed) {
			error = "Liste için bir ad gir.";
			return;
		}
		lists = [...lists, { id: crypto.randomUUID(), name: trimmed, selected: true }];
		dialogOpen = false;
	}
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger
		class={compact
			? cn(buttonVariants({ variant: "outline", size: "icon-lg" }), "size-10", className)
			: cn(buttonVariants({ variant: "default", size: "lg" }), "h-11 gap-0 px-0 text-[0.9375rem]", className)}
		aria-label="{title} için kütüphane seçenekleri"
	>
		{#if compact}
			{#if saved}
				<CheckIcon aria-hidden="true" />
			{:else}
				<PlusIcon aria-hidden="true" />
			{/if}
		{:else}
			<span class="min-w-0 flex-1 truncate px-5">{status || "Listeme ekle"}</span>
			<span aria-hidden="true" class="bg-primary-foreground/20 h-5 w-px shrink-0"></span>
			<ChevronDownIcon class="mx-3.5 size-4" aria-hidden="true" />
		{/if}
	</DropdownMenu.Trigger>

	<DropdownMenu.Content align="start" sideOffset={8} class="w-64 p-2">
		<DropdownMenu.Label class="px-2 pt-1 pb-2">
			<span class="text-primary block text-xs font-semibold">Kütüphanene ekle</span>
			<span class="text-muted-foreground mt-0.5 block truncate text-[0.6875rem] font-normal">{title}</span>
		</DropdownMenu.Label>

		<DropdownMenu.RadioGroup bind:value={status}>
			{#each statuses as item (item)}
				<DropdownMenu.RadioItem value={item} class={itemClass}>{item}</DropdownMenu.RadioItem>
			{/each}
		</DropdownMenu.RadioGroup>

		<DropdownMenu.Separator class="my-2" />

		<DropdownMenu.CheckboxItem bind:checked={favorite} class={itemClass}>
			<HeartIcon class="size-4" aria-hidden="true" />
			Favoriler
		</DropdownMenu.CheckboxItem>
		{#each lists as list (list.id)}
			<DropdownMenu.CheckboxItem bind:checked={list.selected} class={itemClass}>
				<ListPlusIcon class="size-4" aria-hidden="true" />
				<span class="truncate">{list.name}</span>
			</DropdownMenu.CheckboxItem>
		{/each}

		<DropdownMenu.Separator class="my-2" />
		<DropdownMenu.Item class={itemClass} onclick={openDialog}>
			<ListPlusIcon class="size-4" aria-hidden="true" />
			Yeni liste oluştur
		</DropdownMenu.Item>
	</DropdownMenu.Content>
</DropdownMenu.Root>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-sm">
		<Dialog.Header class="text-left">
			<Dialog.Title>Yeni liste</Dialog.Title>
			<Dialog.Description>Liste oluşturulunca {title} otomatik olarak eklenir.</Dialog.Description>
		</Dialog.Header>

		<form class="mt-2 space-y-4" onsubmit={createList}>
			<div class="space-y-2">
				<Label for="new-list-name">Liste adı</Label>
				<Input id="new-list-name" bind:value={name} placeholder="ör. Hafta sonu oyunları" autocomplete="off" />
			</div>
			{#if error}
				<p class="text-destructive text-xs" role="alert">{error}</p>
			{/if}
			<Dialog.Footer>
				<Button type="submit" class="px-5">Listeyi oluştur</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
