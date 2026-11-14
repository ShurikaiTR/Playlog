<script lang="ts">
	import * as Card from '#lib/components/ui/card/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import AdminPageHeader from '#lib/features/admin/admin-page-header.svelte';
	import GamepadIcon from '@lucide/svelte/icons/gamepad-2';
	import UsersIcon from '@lucide/svelte/icons/users';
	import MessageSquareIcon from '@lucide/svelte/icons/message-square-text';
	import ListIcon from '@lucide/svelte/icons/list';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import type { Component } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const number = new Intl.NumberFormat('en');
	const cards = $derived<{ label: string; value: number; hint?: string; icon: Component }[]>(
		data.stats
			? [
					{
						label: 'Games',
						value: data.stats.games,
						hint: `${number.format(data.stats.games_published)} published · ${number.format(data.stats.games_draft)} drafts`,
						icon: GamepadIcon
					},
					{ label: 'Users', value: data.stats.users, icon: UsersIcon },
					{ label: 'Reviews', value: data.stats.reviews, icon: MessageSquareIcon },
					{ label: 'Lists', value: data.stats.lists, icon: ListIcon }
				]
			: []
	);
</script>

<AdminPageHeader title="Dashboard" description="How big Playthru is right now.">
	{#snippet actions()}
		<Button href="/admin/games/new" size="lg"><PlusIcon />New game</Button>
	{/snippet}
</AdminPageHeader>

{#if data.stats}
	<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
		{#each cards as card (card.label)}
			<Card.Root>
				<Card.Header>
					<Card.Description class="flex items-center gap-2 text-callout">
						<card.icon class="size-4" aria-hidden="true" />{card.label}
					</Card.Description>
					<Card.Title class="text-large-title numeric">{number.format(card.value)}</Card.Title>
				</Card.Header>
				{#if card.hint}
					<Card.Footer class="text-footnote text-muted-foreground">{card.hint}</Card.Footer>
				{/if}
			</Card.Root>
		{/each}
	</div>
{:else}
	<p class="text-callout text-muted-foreground">The numbers could not be loaded. Try again.</p>
{/if}
