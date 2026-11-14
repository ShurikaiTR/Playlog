<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import * as Card from '#lib/components/ui/card/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';
	import type { AdminStats } from '#lib/server/admin.js';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import GamepadIcon from '@lucide/svelte/icons/gamepad-2';
	import UsersIcon from '@lucide/svelte/icons/users';
	import MessageSquareIcon from '@lucide/svelte/icons/message-square-text';
	import ListIcon from '@lucide/svelte/icons/list';
	import ChangeBadge from './change-badge.svelte';

	let { stats, isAdmin, chart }: { stats: AdminStats; isAdmin: boolean; chart?: Snippet } =
		$props();

	const number = new Intl.NumberFormat('en');

	type Metric = {
		label: string;
		value: number;
		added: number;
		before: number;
		icon: Component;
		/** The admin page that lists them; none for reviews and lists, which have no page yet. */
		href?: string;
	};

	const games = $derived<Metric>({
		label: 'Games',
		value: stats.games,
		added: stats.games_new,
		before: stats.games_prev,
		icon: GamepadIcon,
		href: '/admin/games'
	});
	const users = $derived<Metric>({
		label: 'Users',
		value: stats.users,
		added: stats.users_new,
		before: stats.users_prev,
		icon: UsersIcon,
		// Moderators can't see the user list.
		href: isAdmin ? '/admin/users' : undefined
	});
	const reviews = $derived<Metric>({
		label: 'Reviews',
		value: stats.reviews,
		added: stats.reviews_new,
		before: stats.reviews_prev,
		icon: MessageSquareIcon
	});
	const lists = $derived<Metric>({
		label: 'Lists',
		value: stats.lists,
		added: stats.lists_new,
		before: stats.lists_prev,
		icon: ListIcon
	});
</script>

{#snippet small(metric: Metric)}
	<Card.Root class="col-span-12 sm:col-span-6 xl:col-span-3">
		<Card.Content class="flex items-start justify-between gap-4">
			<div class="flex flex-col justify-between gap-5">
				<div class="flex flex-col gap-1">
					<p class="text-headline font-medium">{metric.label}</p>
					<div class="flex items-center gap-2">
						<p class="text-title-1 font-medium tabular-nums">{number.format(metric.value)}</p>
						<ChangeBadge added={metric.added} before={metric.before} />
					</div>
				</div>
				{#if metric.href}
					<Button href={metric.href} variant="outline" size="lg" class="w-fit">
						View<ArrowRightIcon />
					</Button>
				{/if}
			</div>
			<div class="rounded-full p-3 ring-1 ring-border">
				<metric.icon class="size-5" aria-hidden="true" />
			</div>
		</Card.Content>
	</Card.Root>
{/snippet}

<div class="grid grid-cols-12 gap-4 md:gap-6">
	<Card.Root class="relative col-span-12 xl:col-span-6">
		<Card.Content class="flex h-full flex-col justify-between gap-9">
			<div>
				<p class="text-headline font-medium">Playthru Dashboard</p>
				<p class="text-footnote text-muted-foreground">
					The size of the catalog and the community. Changes compare the last 30 days with the 30
					before.
				</p>
			</div>
			<div class="flex items-center gap-6">
				{#each [users, games] as metric, index (metric.label)}
					{#if index > 0}<Separator orientation="vertical" class="h-12" />{/if}
					<div>
						<p class="text-footnote text-muted-foreground">{metric.label}</p>
						<div class="flex items-center gap-1">
							<p class="text-title-1 font-medium tabular-nums">{number.format(metric.value)}</p>
							<ChangeBadge added={metric.added} before={metric.before} />
						</div>
					</div>
				{/each}
			</div>
		</Card.Content>
		<!-- One drawing per theme; the dark one is swapped in by the site's dark class. -->
		<img
			src="/mascot/happy.svg"
			alt=""
			width="220"
			height="165"
			class="pointer-events-none absolute end-3 -bottom-2 hidden h-auto w-52 sm:block dark:hidden"
		/>
		<img
			src="/mascot/happy-dark.svg"
			alt=""
			width="220"
			height="165"
			class="pointer-events-none absolute end-3 -bottom-2 hidden h-auto w-52 dark:sm:block"
		/>
	</Card.Root>

	{@render small(games)}
	{@render small(users)}
	{@render small(reviews)}
	{@render small(lists)}
	{#if chart}
		<div class="col-span-12 xl:col-span-6">{@render chart()}</div>
	{/if}
</div>
