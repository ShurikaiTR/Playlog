<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import ListCard from '#lib/features/lists/list-card.svelte';
	import { createList, listHref, ownLists } from '#lib/features/lists/user-lists.svelte.js';
	import PlusIcon from '@lucide/svelte/icons/plus';

	const lists = $derived(ownLists());

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

	<div class="mt-8 grid gap-4 sm:grid-cols-2 xl:max-w-3xl">
		<button
			type="button"
			onclick={openDialog}
			class="flex min-h-48 w-full cursor-pointer flex-col items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-border text-muted-foreground transition-colors outline-none hover:border-foreground/30 hover:bg-muted/50 hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/50"
		>
			<PlusIcon class="size-6" aria-hidden="true" />
			<span class="text-callout font-medium">New list</span>
		</button>
		{#each lists as list (list.id)}
			<ListCard
				name={list.name}
				kind={list.kind}
				cover={list.backdrop || list.items[0]?.cover || ''}
				gameCount={list.items.length}
				description={list.description || undefined}
				href={listHref(list)}
			/>
		{/each}
	</div>
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
