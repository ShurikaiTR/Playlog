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
		setStatus,
		statusOf,
		type UserList
	} from './user-lists.svelte.js';
	import { cn } from '#lib/utils.js';
	import { toast } from 'svelte-sonner';
	import BanIcon from '@lucide/svelte/icons/ban';
	import CheckIcon from '@lucide/svelte/icons/check';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ClockIcon from '@lucide/svelte/icons/clock-3';
	import GamepadIcon from '@lucide/svelte/icons/gamepad-2';
	import HeartIcon from '@lucide/svelte/icons/heart';
	import ListPlusIcon from '@lucide/svelte/icons/list-plus';
	import PauseCircleIcon from '@lucide/svelte/icons/circle-pause';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	let {
		game,
		compact = false,
		class: className
	}: {
		game: { slug: string; title: string; cover: string | null; year: string };
		// A round icon-only trigger for lists of games.
		compact?: boolean;
		class?: string;
	} = $props();

	const statuses = [
		{ value: 'Playing', icon: GamepadIcon, message: 'Marked as playing' },
		{ value: 'Backlog', icon: ClockIcon, message: 'Added to your backlog' },
		{ value: 'Paused', icon: PauseCircleIcon, message: 'Marked as paused' },
		{ value: 'Completed', icon: CircleCheckIcon, message: 'Marked as completed' },
		{ value: 'Dropped', icon: BanIcon, message: 'Marked as dropped' }
	];
	const favorites = $derived(listByKind('favorites'));
	const custom = $derived(customLists());
	const status = $derived(statusOf(game.slug));
	const saved = $derived(
		Boolean(status) ||
			hasGame(favorites, game.slug) ||
			custom.some((list) => hasGame(list, game.slug))
	);
	const isFavorite = $derived(hasGame(favorites, game.slug));
	const itemClass = 'h-10 gap-2 rounded-xl px-2 font-medium';

	function changeStatus(next: string) {
		const previous = status;
		if (next === previous) return;
		setStatus(game, next);
		toast(statuses.find((item) => item.value === next)?.message ?? 'Removed from your library', {
			action: { label: 'Undo', onClick: () => setStatus(game, previous) }
		});
	}

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

<div class={compact ? 'contents' : cn('flex items-center gap-2', className)}>
	<DropdownMenu.Root>
		<DropdownMenu.Trigger
			class={compact
				? cn(buttonVariants({ variant: 'outline', size: 'icon-lg' }), 'size-10', className)
				: cn(
						buttonVariants({ variant: 'default', size: 'lg' }),
						'h-11 min-w-0 flex-1 gap-0 px-0 text-sm'
					)}
			aria-label="Save options for {game.title}"
		>
			{#if compact}
				{#if saved}
					<CheckIcon aria-hidden="true" />
				{:else}
					<PlusIcon aria-hidden="true" />
				{/if}
			{:else}
				<span class="flex min-w-0 flex-1 items-center gap-2 px-4">
					<span class="truncate"
						>{#if status}{status}{:else}Add<span class="max-sm:hidden">&nbsp;to library</span
							>{/if}</span
					>
				</span>
				<span aria-hidden="true" class="h-5 w-px shrink-0 bg-current opacity-20"></span>
				<ChevronDownIcon class="mx-3 size-4" aria-hidden="true" />
			{/if}
		</DropdownMenu.Trigger>

		<DropdownMenu.Content align="start" sideOffset={8} class="w-56 p-2">
			<DropdownMenu.Label class="px-2 pt-1 pb-2">
				<span class="block text-xs font-semibold text-primary">Add to library</span>
				<span class="mt-0.5 block truncate text-[0.6875rem] font-normal text-muted-foreground"
					>{game.title}</span
				>
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
				<DropdownMenu.Label class="px-2 pb-1 text-xs font-medium text-muted-foreground"
					>My lists</DropdownMenu.Label
				>
				<div class={cn(custom.length > 5 && 'max-h-52 overflow-y-auto')}>
					{#each custom as list (list.id)}
						<DropdownMenu.CheckboxItem
							checked={hasGame(list, game.slug)}
							onCheckedChange={(on) => toggle(list, on)}
							class={itemClass}
						>
							<ListPlusIcon class="size-4" aria-hidden="true" />
							<span class="truncate">{list.name}</span>
							<DropdownMenu.Shortcut>{list.items.length}</DropdownMenu.Shortcut>
						</DropdownMenu.CheckboxItem>
					{/each}
				</div>
			{/if}

			<DropdownMenu.Separator class="my-2" />
			<DropdownMenu.Item class={cn(itemClass, 'text-muted-foreground')} onclick={openDialog}>
				<PlusIcon class="size-4" aria-hidden="true" />
				New list
			</DropdownMenu.Item>

			{#if status}
				<DropdownMenu.Separator class="my-2" />
				<DropdownMenu.Item variant="destructive" class={itemClass} onclick={() => changeStatus('')}>
					<Trash2Icon class="size-4" aria-hidden="true" />
					Remove from library
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
			aria-label={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
			onclick={() => toggle(favorites, !isFavorite)}
		>
			<HeartIcon class={cn(isFavorite && 'fill-current')} aria-hidden="true" />
		</Button>
	{/if}
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
