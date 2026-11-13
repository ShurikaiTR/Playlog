<script lang="ts">
	import { goto } from '$app/navigation';
	import GameCard from '#lib/features/games/game-card.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Slider } from '#lib/components/ui/slider/index.js';
	import { Switch } from '#lib/components/ui/switch/index.js';
	import ResponsivePanel from '#lib/components/layout/responsive-panel.svelte';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import { sorts } from '#lib/features/games/games.js';
	import { cn } from '#lib/utils.js';
	import ArrowUpDownIcon from '@lucide/svelte/icons/arrow-up-down';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import SlidersHorizontalIcon from '@lucide/svelte/icons/sliders-horizontal';
	import SearchXIcon from '@lucide/svelte/icons/search-x';
	import XIcon from '@lucide/svelte/icons/x';
	import { flip } from 'svelte/animate';
	import { fade } from 'svelte/transition';
	import { MediaQuery } from 'svelte/reactivity';
	import type { Snippet } from 'svelte';

	let { data } = $props();

	let filtersOpen = $state(false);

	type ListKey = 'platform' | 'genre' | 'mode';
	type Filters = typeof data.filters;
	const lists: { key: ListKey; label: string }[] = [
		{ key: 'platform', label: 'Platform' },
		{ key: 'genre', label: 'Genre' },
		{ key: 'mode', label: 'Game mode' }
	];

	/** The address for a set of filters; anything left at its default stays out of it. */
	function addressFor(patch: Partial<Filters> & { sort?: string; page?: number }) {
		const next = { ...data.filters, ...patch };
		// A throwaway builder for the next URL, not reactive state.
		// eslint-disable-next-line svelte/prefer-svelte-reactivity
		const params = new URLSearchParams();
		for (const { key } of lists) for (const value of next[key]) params.append(key, value);
		if (next.from !== data.minYear) params.set('from', String(next.from));
		if (next.to !== data.maxYear) params.set('to', String(next.to));
		if (next.upcoming) params.set('upcoming', '1');
		const sort = patch.sort ?? data.sort;
		if (sort !== 'newest') params.set('sort', sort);
		// Changing a filter or the order starts again from the first page.
		if (patch.page && patch.page > 1) params.set('page', String(patch.page));
		const search = params.toString();
		return search ? `?${search}` : '?';
	}

	function update(patch: Partial<Filters> & { sort?: string }) {
		goto(addressFor(patch), { replace: true, reset: false });
	}

	function toggle(key: ListKey, value: string) {
		const current = data.filters[key];
		update({
			[key]: current.includes(value) ? current.filter((v) => v !== value) : [...current, value]
		});
	}

	const labelOf = (key: ListKey, value: string) =>
		data.options[key].find((option) => option.value === value)?.label ?? value;

	const yearsChanged = $derived(
		data.filters.from !== data.minYear || data.filters.to !== data.maxYear
	);
	const chosen = $derived([
		...lists.flatMap(({ key, label }) =>
			data.filters[key].map((value) => ({
				id: key + value,
				label: labelOf(key, value),
				group: label,
				remove: () => toggle(key, value)
			}))
		),
		...(yearsChanged
			? [
					{
						id: 'years',
						label: `${data.filters.from}–${data.filters.to}`,
						group: 'Release year',
						remove: () => update({ from: data.minYear, to: data.maxYear })
					}
				]
			: []),
		...(data.filters.upcoming
			? [
					{
						id: 'upcoming',
						label: 'Coming soon',
						group: 'Release year',
						remove: () => update({ upcoming: false })
					}
				]
			: [])
	]);
	const filtered = $derived(chosen.length > 0);
	const clearAll = () =>
		update({
			platform: [],
			genre: [],
			mode: [],
			from: data.minYear,
			to: data.maxYear,
			upcoming: false
		});

	const reducedMotion = new MediaQuery('(prefers-reduced-motion: reduce)');
	const sortLabel = $derived(sorts.find((sort) => sort.key === data.sort)?.label ?? '');
	const count = (total: number) =>
		`${total.toLocaleString('en-US')} ${total === 1 ? 'game' : 'games'}`;

	// Platform and genre start open; the rest unfold on demand.
	let openGroups = $state<Record<string, boolean>>({ platform: true, genre: true });
	// Slider values while dragging; the address updates on release.
	let years = $derived([data.filters.from, data.filters.to]);

	const optionClass =
		'focus-visible:ring-ring/50 inline-flex min-h-9 cursor-pointer items-center border px-3.5 text-[0.8125rem] font-medium outline-none focus-visible:ring-[3px]';
</script>

<svelte:head>
	<title>Games | Playthru</title>
	<meta
		name="description"
		content="Browse every game on Playthru. Filter by platform, genre, mode and release year."
	/>
</svelte:head>

{#snippet section(id: string, label: string, count: number, body: Snippet)}
	<details
		class="group/section"
		open={openGroups[id]}
		ontoggle={(event) => (openGroups[id] = event.currentTarget.open)}
	>
		<summary
			class="flex min-h-12 cursor-pointer list-none items-center gap-3 px-4 outline-none marker:hidden hover:bg-muted/50 focus-visible:bg-muted/50 [&::-webkit-details-marker]:hidden"
		>
			<span class="text-sm font-semibold text-primary">{label}</span>
			{#if count > 0}
				<span
					class="inline-flex size-5 shrink-0 items-center justify-center rounded-full bg-primary text-[0.6875rem] font-semibold text-primary-foreground tabular-nums"
				>
					{count}
				</span>
			{/if}
			<ChevronDownIcon
				class="ml-auto size-4 shrink-0 text-muted-foreground transition-transform group-open/section:rotate-180 motion-reduce:transition-none"
				aria-hidden="true"
			/>
		</summary>
		<div class="px-4 pt-1 pb-4">{@render body()}</div>
	</details>
{/snippet}

{#snippet chips(key: ListKey)}
	<div class="flex flex-wrap gap-2">
		{#each data.options[key] as option (option.value)}
			{@const on = data.filters[key].includes(option.value)}
			<button
				type="button"
				data-slot="choice"
				aria-pressed={on}
				onclick={() => toggle(key, option.value)}
				class={cn(
					optionClass,
					on
						? 'border-transparent bg-primary text-primary-foreground'
						: 'text-primary hover:bg-muted'
				)}
			>
				{option.label}
			</button>
		{/each}
	</div>
{/snippet}

{#snippet platformBody()}{@render chips('platform')}{/snippet}
{#snippet genreBody()}{@render chips('genre')}{/snippet}
{#snippet modeBody()}{@render chips('mode')}{/snippet}

{#snippet yearBody()}
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-3">
			<div class="flex justify-between text-[0.8125rem] font-medium text-primary tabular-nums">
				<span>{years[0]}</span><span>{years[1]}</span>
			</div>
			<Slider
				type="multiple"
				bind:value={years}
				min={data.minYear}
				max={data.maxYear}
				step={1}
				onValueCommit={([from, to]) => update({ from, to })}
				aria-label="Release year range"
			/>
		</div>
		<label
			class="flex min-h-9 cursor-pointer items-center justify-between gap-3 text-sm text-primary"
		>
			Coming soon
			<Switch
				checked={data.filters.upcoming}
				onCheckedChange={(checked) => update({ upcoming: checked })}
			/>
		</label>
	</div>
{/snippet}

{#snippet filterSections()}
	<div class="divide-y border-y">
		{@render section('platform', 'Platform', data.filters.platform.length, platformBody)}
		{@render section('genre', 'Genre', data.filters.genre.length, genreBody)}
		{@render section(
			'years',
			'Release year',
			(yearsChanged ? 1 : 0) + (data.filters.upcoming ? 1 : 0),
			yearBody
		)}
		{#if data.options.mode.length}
			{@render section('mode', 'Game mode', data.filters.mode.length, modeBody)}
		{/if}
	</div>
{/snippet}

<div class="min-w-0 px-4 pb-20 md:px-8 md:pb-24">
	<header class="pt-2 pb-6 md:pt-4">
		<h1
			class="text-[2rem] leading-tight font-medium tracking-[-0.025em] text-primary md:text-[2.5rem]"
		>
			Games
		</h1>
		<p class="mt-2 text-sm leading-[1.6] text-muted-foreground" aria-live="polite">
			{count(data.total)}
		</p>
	</header>

	<div class="flex items-center justify-end gap-2">
		<Button
			variant="outline"
			size="lg"
			class="h-11 shrink-0 px-4 text-[0.9375rem]"
			aria-haspopup="dialog"
			onclick={() => (filtersOpen = true)}
		>
			<SlidersHorizontalIcon data-icon="inline-start" aria-hidden="true" />
			<span class="max-sm:sr-only">Filters</span>
			{#if chosen.length > 0}
				<span
					class="inline-flex size-5 items-center justify-center rounded-full bg-primary text-[0.6875rem] font-semibold text-primary-foreground tabular-nums"
				>
					{chosen.length}
				</span>
			{/if}
		</Button>

		<ResponsivePanel
			bind:open={filtersOpen}
			title="Filters"
			description="Narrow down the list."
			sheetClass="p-0 data-[side=right]:w-[88%]"
			titleClass="text-base font-semibold"
			footerClass="flex-row gap-2 border-t"
		>
			<div class="min-h-0 flex-1 overflow-y-auto">{@render filterSections()}</div>
			{#snippet footer()}
				<Button
					variant="ghost"
					size="lg"
					class="h-11 px-5 text-[0.9375rem]"
					disabled={chosen.length === 0}
					onclick={clearAll}>Clear</Button
				>
				<Button
					size="lg"
					class="h-11 flex-1 px-5 text-[0.9375rem]"
					onclick={() => (filtersOpen = false)}>Show {count(data.total)}</Button
				>
			{/snippet}
		</ResponsivePanel>

		<DropdownMenu.Root>
			<DropdownMenu.Trigger>
				{#snippet child({ props })}
					<Button
						{...props}
						variant="outline"
						size="lg"
						class="h-11 shrink-0 px-4 text-[0.9375rem]"
					>
						<ArrowUpDownIcon data-icon="inline-start" aria-hidden="true" />
						<span class="max-sm:sr-only">Sort:</span>
						{sortLabel}
					</Button>
				{/snippet}
			</DropdownMenu.Trigger>
			<DropdownMenu.Content align="end" sideOffset={8} class="w-44 p-2">
				<DropdownMenu.RadioGroup
					value={data.sort}
					onValueChange={(value) => update({ sort: value })}
				>
					{#each sorts as sort (sort.key)}
						<DropdownMenu.RadioItem
							value={sort.key}
							class="h-9 rounded-xl px-2 font-medium"
							closeOnSelect>{sort.label}</DropdownMenu.RadioItem
						>
					{/each}
				</DropdownMenu.RadioGroup>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</div>

	{#if filtered}
		<ul class="mt-4 flex flex-wrap items-center gap-2" aria-label="Active filters">
			{#each chosen as item (item.id)}
				<li>
					<button
						type="button"
						data-slot="choice"
						onclick={item.remove}
						aria-label="Remove filter {item.group}: {item.label}"
						class="inline-flex min-h-9 cursor-pointer items-center gap-1.5 bg-muted pr-2.5 pl-3.5 text-[0.8125rem] font-medium text-primary outline-none hover:bg-muted/70 focus-visible:ring-[3px] focus-visible:ring-ring/50"
					>
						{item.label}
						<XIcon class="size-3.5.5" aria-hidden="true" />
					</button>
				</li>
			{/each}
			<li>
				<Button variant="ghost" size="sm" class="h-9 px-3 text-[0.8125rem]" onclick={clearAll}
					>Clear filters</Button
				>
			</li>
		</ul>
	{/if}

	{#if data.games.length}
		<ul
			class="mt-8 grid grid-cols-2 gap-x-4 gap-y-8 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6"
		>
			{#each data.games as game (game.slug)}
				<li
					animate:flip={{ duration: reducedMotion.current ? 0 : 320 }}
					in:fade={{ duration: reducedMotion.current ? 0 : 200 }}
				>
					<GameCard {game} />
				</li>
			{/each}
		</ul>
		{#if data.pages > 1}
			<nav class="mt-12 flex items-center justify-center gap-3" aria-label="Pages">
				<Button
					variant="outline"
					size="lg"
					class="h-11 px-5"
					href={data.page > 1 ? addressFor({ page: data.page - 1 }) : undefined}
					disabled={data.page <= 1}>Previous</Button
				>
				<p class="text-sm text-muted-foreground tabular-nums" aria-current="page">
					Page {data.page} of {data.pages}
				</p>
				<Button
					variant="outline"
					size="lg"
					class="h-11 px-5"
					href={data.page < data.pages ? addressFor({ page: data.page + 1 }) : undefined}
					disabled={data.page >= data.pages}>Next</Button
				>
			</nav>
		{/if}
	{:else}
		<div class="mt-12 flex min-h-72 flex-col items-center justify-center gap-3 px-6 text-center">
			<span
				class="flex size-14 items-center justify-center rounded-full bg-muted text-muted-foreground"
			>
				<SearchXIcon class="size-5" aria-hidden="true" />
			</span>
			<p class="text-[0.9375rem] font-medium text-primary">
				{data.unavailable
					? 'We can’t reach the catalog right now'
					: filtered
						? 'No games match these filters'
						: 'No games yet'}
			</p>
			<p class="text-sm text-muted-foreground">
				{data.unavailable
					? 'Try again in a moment.'
					: filtered
						? 'Try removing a filter or two.'
						: 'Games will show up here once they are added.'}
			</p>
			{#if filtered}
				<Button variant="outline" size="lg" class="mt-1 h-11 px-5" onclick={clearAll}
					>Clear filters</Button
				>
			{/if}
		</div>
	{/if}
</div>
