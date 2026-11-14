<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Card from '#lib/components/ui/card/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { createList, listHref, ownLists } from '#lib/features/lists/user-lists.svelte.js';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import ClockIcon from '@lucide/svelte/icons/clock-3';
	import GamepadIcon from '@lucide/svelte/icons/gamepad-2';
	import LockIcon from '@lucide/svelte/icons/lock';
	import PlusIcon from '@lucide/svelte/icons/plus';

	const lists = $derived(ownLists());
	const icons = { played: CircleCheckIcon, play_later: ClockIcon, custom: GamepadIcon };

	let dialogOpen = $state(false);
	let name = $state('');
	let error = $state('');
	let creating = $state(false);

	function openDialog() {
		name = '';
		error = '';
		dialogOpen = true;
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const trimmed = name.trim();
		if (!trimmed) {
			error = 'Give the list a name.';
			return;
		}
		creating = true;
		const created = await createList(trimmed);
		creating = false;
		if (!created) return;
		dialogOpen = false;
		await goto(listHref(created));
	}
</script>

<svelte:head>
	<title>Library | Playlog</title>
</svelte:head>

<div class="min-w-0 px-4 pt-24 pb-16 md:px-8">
	<h1 class="text-large-title text-primary">Library</h1>
	<p class="mt-1 text-callout text-muted-foreground">
		Your played games, your play later pile and the lists you make.
	</p>

	<ul class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
		<li>
			<button
				type="button"
				onclick={openDialog}
				class="flex h-full min-h-44 w-full cursor-pointer flex-col items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-border text-muted-foreground transition-colors outline-none hover:border-foreground/30 hover:bg-muted/50 hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/50"
			>
				<PlusIcon class="size-6" aria-hidden="true" />
				<span class="text-callout font-medium">New list</span>
			</button>
		</li>
		{#each lists as list (list.id)}
			{@const Icon = icons[list.kind]}
			{@const cover = list.backdrop || list.items[0]?.cover}
			<li>
				<Card.Root
					class="group/list h-full gap-0 overflow-hidden pt-0 pb-0 transition-transform duration-300 hover:-translate-y-1"
				>
					<a
						href={listHref(list)}
						class="flex h-full flex-col rounded-2xl outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
					>
						<div class="relative aspect-video bg-muted">
							{#if cover}
								<img src={cover} alt="" loading="lazy" class="size-full object-cover" />
							{:else}
								<div class="flex size-full items-center justify-center">
									<Icon class="size-8 text-muted-foreground" aria-hidden="true" />
								</div>
							{/if}
						</div>
						<Card.Header class="py-4">
							<Card.Title class="flex items-center gap-1.5 truncate">
								<span class="truncate">{list.name}</span>
								{#if list.visibility === 'private'}
									<LockIcon class="size-3.5 shrink-0 text-muted-foreground" aria-label="Private" />
								{/if}
							</Card.Title>
							<Card.Description>
								{list.items.length}
								{list.items.length === 1 ? 'game' : 'games'}
							</Card.Description>
						</Card.Header>
					</a>
				</Card.Root>
			</li>
		{/each}
	</ul>
</div>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-sm">
		<Dialog.Header class="text-left">
			<Dialog.Title>New list</Dialog.Title>
			<Dialog.Description>Give it a name, then add games from their pages.</Dialog.Description>
		</Dialog.Header>
		<form class="mt-2 space-y-4" onsubmit={submit}>
			<div class="space-y-2">
				<Label for="library-list-name">List name</Label>
				<Input
					id="library-list-name"
					bind:value={name}
					placeholder="e.g. Weekend games"
					autocomplete="off"
				/>
			</div>
			{#if error}
				<p class="text-footnote text-destructive" role="alert">{error}</p>
			{/if}
			<Dialog.Footer>
				<Button type="submit" class="px-5" disabled={creating}>Create list</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
