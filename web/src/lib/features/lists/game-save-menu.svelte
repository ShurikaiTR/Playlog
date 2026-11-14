<script lang="ts">
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import {
		createList,
		customLists,
		hasGame,
		listByKind,
		setGame,
		type UserList
	} from './user-lists.svelte.js';
	import { cn } from '#lib/utils.js';
	import { toast } from 'svelte-sonner';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ClockIcon from '@lucide/svelte/icons/clock-3';
	import HeartIcon from '@lucide/svelte/icons/heart';
	import ListIcon from '@lucide/svelte/icons/list';
	import PlusIcon from '@lucide/svelte/icons/plus';

	let {
		game,
		class: className
	}: {
		game: { slug: string; title: string; cover: string | null; year: string };
		class?: string;
	} = $props();

	const favorites = $derived(listByKind('favorites'));
	const playLater = $derived(listByKind('play_later'));
	const custom = $derived(customLists());
	const saved = $derived(
		hasGame(favorites, game.slug) ||
			hasGame(playLater, game.slug) ||
			custom.some((list) => hasGame(list, game.slug))
	);

	function toggle(list: UserList, on: boolean) {
		setGame(list, game, on);
		toast(on ? `Added to ${list.name}` : `Removed from ${list.name}`, {
			action: { label: 'Undo', onClick: () => setGame(list, game, !on) }
		});
	}

	let dialogOpen = $state(false);
	let name = $state('');
	let error = $state('');

	function openDialog() {
		name = '';
		error = '';
		dialogOpen = true;
	}

	function submitList(event: SubmitEvent) {
		event.preventDefault();
		const trimmed = name.trim();
		if (!trimmed) {
			error = 'Give the list a name.';
			return;
		}
		createList(trimmed, game);
		dialogOpen = false;
		toast(`Created ${trimmed}`);
	}
</script>

<div class="contents">
	<DropdownMenu.Root>
		<DropdownMenu.Trigger
			class={cn(
				buttonVariants({ variant: 'default', size: 'lg' }),
				'h-11 shrink-0 gap-0 px-0 text-sm whitespace-nowrap',
				className
			)}
			aria-label="Save {game.title} to a list"
		>
			<span class="px-4">{saved ? 'Saved' : 'Add to library'}</span>
			<span aria-hidden="true" class="h-5 w-px shrink-0 bg-current opacity-20"></span>
			<ChevronDownIcon class="mx-3 size-4" aria-hidden="true" />
		</DropdownMenu.Trigger>

		<DropdownMenu.Content align="start" sideOffset={8} class="w-60 p-2">
			<DropdownMenu.Label class="px-2 pt-1 pb-2">
				<span class="block text-xs font-semibold text-primary">Add to library</span>
				<span class="mt-0.5 block truncate text-[0.6875rem] font-normal text-muted-foreground"
					>{game.title}</span
				>
			</DropdownMenu.Label>

			{#each [{ list: favorites, icon: HeartIcon }, { list: playLater, icon: ClockIcon }] as { list, icon: Icon } (list.id)}
				<DropdownMenu.CheckboxItem
					checked={hasGame(list, game.slug)}
					onCheckedChange={(on) => toggle(list, on)}
				>
					<Icon class="size-4" aria-hidden="true" />
					<span class="truncate">{list.name}</span>
				</DropdownMenu.CheckboxItem>
			{/each}

			{#if custom.length}
				<DropdownMenu.Separator class="my-2" />
				<DropdownMenu.Label class="px-2 pb-1 text-xs font-medium text-muted-foreground"
					>My lists</DropdownMenu.Label
				>
				<div class={cn(custom.length > 5 && 'max-h-52 overflow-y-auto')}>
					{#each custom as list (list.id)}
						<DropdownMenu.CheckboxItem
							checked={hasGame(list, game.slug)}
							onCheckedChange={(on) => toggle(list, on)}
						>
							<ListIcon class="size-4" aria-hidden="true" />
							<span class="truncate">{list.name}</span>
						</DropdownMenu.CheckboxItem>
					{/each}
				</div>
			{/if}

			<DropdownMenu.Separator class="my-2" />
			<DropdownMenu.Item class="text-muted-foreground" onclick={openDialog}>
				<PlusIcon class="size-4" aria-hidden="true" />
				New list
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>
</div>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-sm">
		<Dialog.Header class="text-left">
			<Dialog.Title>New list</Dialog.Title>
			<Dialog.Description>{game.title} is added to the list once it is created.</Dialog.Description>
		</Dialog.Header>

		<form class="mt-2 space-y-4" onsubmit={submitList}>
			<div class="space-y-2">
				<Label for="new-list-name">List name</Label>
				<Input
					id="new-list-name"
					bind:value={name}
					placeholder="e.g. Weekend games"
					autocomplete="off"
				/>
			</div>
			{#if error}
				<p class="text-xs text-destructive" role="alert">{error}</p>
			{/if}
			<Dialog.Footer>
				<Button type="submit" class="px-5">Create list</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
