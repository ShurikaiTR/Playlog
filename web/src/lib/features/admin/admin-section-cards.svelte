<script lang="ts">
	import * as Card from '#lib/components/ui/card/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import type { AdminStats } from '#lib/server/admin.js';
	import TrendingUpIcon from '@lucide/svelte/icons/trending-up';

	let { stats }: { stats: AdminStats } = $props();

	const number = new Intl.NumberFormat('en');
	const recent = (key: 'users' | 'games') =>
		stats.activity.slice(-30).reduce((sum, day) => sum + day[key], 0);

	const cards = $derived([
		{
			label: 'Games',
			value: stats.games,
			recent: recent('games'),
			hint: `${number.format(stats.games_published)} published · ${number.format(stats.games_draft)} drafts`
		},
		{
			label: 'Users',
			value: stats.users,
			recent: recent('users'),
			hint: 'Signed-up accounts'
		},
		{ label: 'Reviews', value: stats.reviews, recent: null, hint: 'Written by members' },
		{
			label: 'Lists',
			value: stats.lists,
			recent: null,
			hint: 'Including the played and play-later lists'
		}
	]);
</script>

<div
	class="grid grid-cols-1 gap-4 *:data-[slot=card]:bg-gradient-to-t *:data-[slot=card]:from-primary/5 *:data-[slot=card]:to-card *:data-[slot=card]:shadow-xs @xl/main:grid-cols-2 @5xl/main:grid-cols-4 dark:*:data-[slot=card]:bg-card"
>
	{#each cards as card (card.label)}
		<Card.Root class="@container/card">
			<Card.Header>
				<Card.Description>{card.label}</Card.Description>
				<Card.Title class="text-title-1 font-semibold tabular-nums @[250px]/card:text-large-title">
					{number.format(card.value)}
				</Card.Title>
				{#if card.recent !== null}
					<Card.Action>
						<Badge variant="outline"><TrendingUpIcon />+{number.format(card.recent)}</Badge>
					</Card.Action>
				{/if}
			</Card.Header>
			<Card.Footer class="flex-col items-start gap-1 text-footnote">
				{#if card.recent !== null}
					<div class="font-medium">{number.format(card.recent)} new in the last 30 days</div>
				{/if}
				<div class="text-muted-foreground">{card.hint}</div>
			</Card.Footer>
		</Card.Root>
	{/each}
</div>
