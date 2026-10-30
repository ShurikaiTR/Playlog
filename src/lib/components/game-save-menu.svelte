<script lang="ts">
	import { Button, buttonVariants } from "#lib/components/ui/button/index.js";
	import * as Dialog from "#lib/components/ui/dialog/index.js";
	import * as DropdownMenu from "#lib/components/ui/dropdown-menu/index.js";
	import { Input } from "#lib/components/ui/input/index.js";
	import { Label } from "#lib/components/ui/label/index.js";
	import { createList, customLists, hasGame, listByKind, setGame, setStatus, statusOf, type UserList } from "#lib/user-lists.svelte.js";
	import { cn } from "#lib/utils.js";
	import { toast } from "svelte-sonner";
	import BanIcon from "@lucide/svelte/icons/ban";
	import CheckIcon from "@lucide/svelte/icons/check";
	import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
	import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
	import ClockIcon from "@lucide/svelte/icons/clock-3";
	import GamepadIcon from "@lucide/svelte/icons/gamepad-2";
	import HeartIcon from "@lucide/svelte/icons/heart";
	import ListPlusIcon from "@lucide/svelte/icons/list-plus";
	import PauseCircleIcon from "@lucide/svelte/icons/circle-pause";
	import PlusIcon from "@lucide/svelte/icons/plus";
	import Trash2Icon from "@lucide/svelte/icons/trash-2";

	let {
		game,
		compact = false,
		class: className,
	}: {
		game: { slug: string; title: string; cover: string; score: number; year: string };
		// A round icon-only trigger for lists of games.
		compact?: boolean;
		class?: string;
	} = $props();

	const statuses = [
		{ value: "Oynuyorum", icon: GamepadIcon, message: "Oynuyorum olarak işaretlendi" },
		{ value: "Oynamayı planlıyorum", short: "Planlanan", icon: ClockIcon, message: "Planlananlara eklendi" },
		{ value: "Ara verdim", icon: PauseCircleIcon, message: "Ara verdim olarak işaretlendi" },
		{ value: "Bitirdim", icon: CircleCheckIcon, message: "Bitirdim olarak işaretlendi" },
		{ value: "Bıraktım", icon: BanIcon, message: "Bıraktım olarak işaretlendi" },
	];
	const favorites = $derived(listByKind("favorites"));
	const custom = $derived(customLists());
	const status = $derived(statusOf(game.slug));
	const current = $derived(statuses.find((item) => item.value === status));
	const saved = $derived(Boolean(status) || hasGame(favorites, game.slug) || custom.some((list) => hasGame(list, game.slug)));
	const isFavorite = $derived(hasGame(favorites, game.slug));
	const itemClass = "h-10 gap-2 rounded-xl px-2 font-medium";

	function changeStatus(next: string) {
		const previous = status;
		if (next === previous) return;
		setStatus(game, next);
		toast(statuses.find((item) => item.value === next)?.message ?? "Kütüphaneden çıkarıldı", {
			action: { label: "Geri al", onClick: () => setStatus(game, previous) },
		});
	}

	function toggle(list: UserList, on: boolean) {
		setGame(list, game, on);
		toast(on ? `${list.name} listesine eklendi` : `${list.name} listesinden çıkarıldı`, {
			action: { label: "Geri al", onClick: () => setGame(list, game, !on) },
		});
	}

	let dialogOpen = $state(false);
	let name = $state("");
	let error = $state("");

	function openDialog() {
		name = "";
		error = "";
		dialogOpen = true;
	}

	function submitList(event: SubmitEvent) {
		event.preventDefault();
		const trimmed = name.trim();
		if (!trimmed) {
			error = "Liste için bir ad gir.";
			return;
		}
		createList(trimmed, game);
		dialogOpen = false;
		toast(`${trimmed} listesi oluşturuldu`);
	}
</script>

<div class={compact ? "contents" : cn("flex items-center gap-2", className)}>
<DropdownMenu.Root>
	<DropdownMenu.Trigger
		class={compact
			? cn(buttonVariants({ variant: "outline", size: "icon-lg" }), "size-10", className)
			: cn(buttonVariants({ variant: "default", size: "lg" }), "h-11 min-w-0 flex-1 gap-0 px-0 text-sm")}
		aria-label="{game.title} için kaydetme seçenekleri"
	>
		{#if compact}
			{#if saved}
				<CheckIcon aria-hidden="true" />
			{:else}
				<PlusIcon aria-hidden="true" />
			{/if}
		{:else}
			<span class="flex min-w-0 flex-1 items-center gap-2 px-4">
				<span class="truncate">{(current && "short" in current ? current.short : status) || "Listeme ekle"}</span>
			</span>
			<span aria-hidden="true" class="h-5 w-px shrink-0 bg-current opacity-20"></span>
			<ChevronDownIcon class="mx-3 size-4" aria-hidden="true" />
		{/if}
	</DropdownMenu.Trigger>

	<DropdownMenu.Content align="start" sideOffset={8} class="w-56 p-2">
		<DropdownMenu.Label class="px-2 pt-1 pb-2">
			<span class="text-primary block text-xs font-semibold">Kütüphanene ekle</span>
			<span class="text-muted-foreground mt-0.5 block truncate text-[0.6875rem] font-normal">{game.title}</span>
		</DropdownMenu.Label>

		<DropdownMenu.RadioGroup value={status} onValueChange={changeStatus}>
			{#each statuses as item (item.value)}
				<DropdownMenu.RadioItem value={item.value} class={itemClass} closeOnSelect>
					<item.icon class="size-4" aria-hidden="true" />
					{item.value}
				</DropdownMenu.RadioItem>
			{/each}
		</DropdownMenu.RadioGroup>

		{#if custom.length}
			<DropdownMenu.Separator class="my-2" />
			<DropdownMenu.Label class="text-muted-foreground px-2 pb-1 text-xs font-medium">Listelerim</DropdownMenu.Label>
			<div class={cn(custom.length > 5 && "max-h-52 overflow-y-auto")}>
				{#each custom as list (list.id)}
					<DropdownMenu.CheckboxItem checked={hasGame(list, game.slug)} onCheckedChange={(on) => toggle(list, on)} class={itemClass}>
						<ListPlusIcon class="size-4" aria-hidden="true" />
						<span class="truncate">{list.name}</span>
						<DropdownMenu.Shortcut>{list.items.length}</DropdownMenu.Shortcut>
					</DropdownMenu.CheckboxItem>
				{/each}
			</div>
		{/if}

		<DropdownMenu.Separator class="my-2" />
		<DropdownMenu.Item class={cn(itemClass, "text-muted-foreground")} onclick={openDialog}>
			<PlusIcon class="size-4" aria-hidden="true" />
			Yeni liste oluştur
		</DropdownMenu.Item>

		{#if status}
			<DropdownMenu.Separator class="my-2" />
			<DropdownMenu.Item variant="destructive" class={itemClass} onclick={() => changeStatus("")}>
				<Trash2Icon class="size-4" aria-hidden="true" />
				Kütüphaneden çıkar
			</DropdownMenu.Item>
		{/if}
	</DropdownMenu.Content>
</DropdownMenu.Root>

{#if !compact}
	<Button
		variant="outline"
		size="icon-lg"
		class="size-11 shrink-0 cursor-pointer"
		aria-pressed={isFavorite}
		aria-label={isFavorite ? "Favorilerden çıkar" : "Favorilere ekle"}
		onclick={() => toggle(favorites, !isFavorite)}
	>
		<HeartIcon class={cn(isFavorite && "fill-current")} aria-hidden="true" />
	</Button>
{/if}
</div>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-sm">
		<Dialog.Header class="text-left">
			<Dialog.Title>Yeni liste</Dialog.Title>
			<Dialog.Description>Liste oluşturulunca {game.title} otomatik olarak eklenir.</Dialog.Description>
		</Dialog.Header>

		<form class="mt-2 space-y-4" onsubmit={submitList}>
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
