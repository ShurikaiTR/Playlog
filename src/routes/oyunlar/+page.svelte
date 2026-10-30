<script lang="ts">
	import { goto } from '$app/navigation';
	import GameCard from '#lib/components/game-card.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Slider } from '#lib/components/ui/slider/index.js';
	import { Switch } from '#lib/components/ui/switch/index.js';
	import * as Sheet from '#lib/components/ui/sheet/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import { sorts } from '#lib/games.js';
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

	type ListKey = 'platform' | 'tur' | 'mod';
	type Filters = typeof data.filters;
	const lists: { key: ListKey; label: string }[] = [
		{ key: 'platform', label: 'Platform' },
		{ key: 'tur', label: 'Tür' },
		{ key: 'mod', label: 'Oyun modu' }
	];

	function update(patch: Partial<Filters> & { sirala?: string }) {
		const next = { ...data.filters, ...patch };
		const params = new URLSearchParams();
		for (const { key } of lists) for (const value of next[key]) params.append(key, value);
		if (next.yilmin !== data.minYear) params.set('yilmin', String(next.yilmin));
		if (next.yilmax !== data.maxYear) params.set('yilmax', String(next.yilmax));
		if (next.yakinda) params.set('yakinda', '1');
		if (next.puan > 0) params.set('puan', String(next.puan));
		const sirala = patch.sirala ?? data.sort;
		if (sirala !== 'populer') params.set('sirala', sirala);
		const search = params.toString();
		goto(search ? `?${search}` : '?', { replace: true, reset: false });
	}

	function toggle(key: ListKey, value: string) {
		const current = data.filters[key];
		update({
			[key]: current.includes(value) ? current.filter((v) => v !== value) : [...current, value]
		});
	}

	const yearsChanged = $derived(
		data.filters.yilmin !== data.minYear || data.filters.yilmax !== data.maxYear
	);
	const chosen = $derived([
		...lists.flatMap(({ key, label }) =>
			data.filters[key].map((value) => ({ id: key + value, label: value, group: label, remove: () => toggle(key, value) }))
		),
		...(yearsChanged
			? [{ id: 'yil', label: `${data.filters.yilmin}–${data.filters.yilmax}`, group: 'Çıkış yılı', remove: () => update({ yilmin: data.minYear, yilmax: data.maxYear }) }]
			: []),
		...(data.filters.yakinda
			? [{ id: 'yakinda', label: 'Yakında çıkacaklar', group: 'Çıkış yılı', remove: () => update({ yakinda: false }) }]
			: []),
		...(data.filters.puan > 0
			? [{ id: 'puan', label: `Puan ${data.filters.puan}+`, group: 'Topluluk puanı', remove: () => update({ puan: 0 }) }]
			: [])
	]);
	const filtered = $derived(chosen.length > 0);
	const clearAll = () =>
		update({ platform: [], tur: [], mod: [], yilmin: data.minYear, yilmax: data.maxYear, yakinda: false, puan: 0 });

	const reducedMotion = new MediaQuery('(prefers-reduced-motion: reduce)');
	// Each card shows its first genre and score under the title.
	const cards = $derived(
		data.games.map((game) => ({ ...game, meta: `${game.genres[0]} · ★ ${game.score.toFixed(1)}` }))
	);
	const sortLabel = $derived(sorts.find((sort) => sort.key === data.sort)?.label ?? '');

	// Platform and Tür start open; the rest unfold on demand.
	let openGroups = $state<Record<string, boolean>>({ platform: true, tur: true });
	// Slider values while dragging; the address updates on release.
	let years = $derived([data.filters.yilmin, data.filters.yilmax]);
	let puan = $derived(data.filters.puan);

	const optionClass =
		'focus-visible:ring-ring/50 inline-flex min-h-9 cursor-pointer items-center border px-3.5 text-[0.8125rem] font-medium outline-none focus-visible:ring-[3px]';
</script>

<svelte:head>
	<title>Tüm oyunlar | Playlog</title>
	<meta
		name="description"
		content="Playlog'daki tüm oyunlara göz at: platforma, türe, temaya ve puana göre süz ve sırala."
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
		{#each data.options[key] as option (option)}
			{@const on = data.filters[key].includes(option)}
			<button
				type="button"
				data-slot="choice"
				aria-pressed={on}
				onclick={() => toggle(key, option)}
				class={cn(
					optionClass,
					on ? 'border-transparent bg-primary text-primary-foreground' : 'text-primary hover:bg-muted'
				)}
			>
				{option}
			</button>
		{/each}
	</div>
{/snippet}

{#snippet platformBody()}{@render chips('platform')}{/snippet}
{#snippet genreBody()}{@render chips('tur')}{/snippet}
{#snippet modeBody()}{@render chips('mod')}{/snippet}

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
				onValueCommit={([from, to]) => update({ yilmin: from, yilmax: to })}
				aria-label="Çıkış yılı aralığı"
			/>
		</div>
		<label class="flex min-h-9 cursor-pointer items-center justify-between gap-3 text-sm text-primary">
			Yakında çıkacaklar
			<Switch
				checked={data.filters.yakinda}
				onCheckedChange={(checked) => update({ yakinda: checked })}
			/>
		</label>
	</div>
{/snippet}

{#snippet scoreBody()}
	<div class="flex flex-col gap-3">
		<div class="flex justify-between text-[0.8125rem] text-muted-foreground">
			<span>En az</span><span class="font-medium text-primary tabular-nums">{puan}</span>
		</div>
		<Slider
			type="single"
			bind:value={puan}
			min={0}
			max={10}
			step={0.5}
			onValueCommit={(value) => update({ puan: value })}
			aria-label="En düşük topluluk puanı"
		/>
	</div>
{/snippet}

{#snippet filterSections()}
	<div class="divide-y border-y">
		{@render section('platform', 'Platform', data.filters.platform.length, platformBody)}
		{@render section('tur', 'Tür', data.filters.tur.length, genreBody)}
		{@render section('yil', 'Çıkış yılı', (yearsChanged ? 1 : 0) + (data.filters.yakinda ? 1 : 0), yearBody)}
		{@render section('puan', 'Topluluk puanı', data.filters.puan > 0 ? 1 : 0, scoreBody)}
		{@render section('mod', 'Oyun modu', data.filters.mod.length, modeBody)}
	</div>
{/snippet}


<div lang="tr" class="min-w-0 px-4 pb-20 md:px-8 md:pb-24">
	<header class="pt-2 pb-6 md:pt-4">
		<h1
			class="text-[2rem] leading-tight font-medium tracking-[-0.025em] text-primary md:text-[2.5rem]"
		>
			Tüm oyunlar
		</h1>
		<p class="mt-2 text-sm leading-[1.6] text-muted-foreground" aria-live="polite">
			{#if filtered}{data.games.length} / {data.total} oyun{:else}{data.total} oyun{/if}
		</p>
	</header>

	<div class="flex items-center justify-end gap-2">
		<Sheet.Root>
			<Sheet.Trigger>
				{#snippet child({ props })}
					<Button
						{...props}
						variant="outline"
						size="lg"
						class="h-11 shrink-0 px-4 text-[0.9375rem]"
					>
						<SlidersHorizontalIcon data-icon="inline-start" aria-hidden="true" />
						<span class="max-sm:sr-only">Filtrele</span>
						{#if chosen.length > 0}
							<span
								class="inline-flex size-5 items-center justify-center rounded-full bg-primary text-[0.6875rem] font-semibold text-primary-foreground tabular-nums"
							>
								{chosen.length}
							</span>
						{/if}
					</Button>
				{/snippet}
			</Sheet.Trigger>

			<Sheet.Content side="right" class="gap-0 p-0 data-[side=right]:w-[88%]">
				<Sheet.Header class="pr-12">
					<Sheet.Title class="text-base font-semibold">Filtrele</Sheet.Title>
					<Sheet.Description>Oyunları süz.</Sheet.Description>
				</Sheet.Header>
				<div class="min-h-0 flex-1 overflow-y-auto">{@render filterSections()}</div>
				<Sheet.Footer class="flex-row gap-2 border-t">
					<Button
						variant="ghost"
						size="lg"
						class="h-11 px-5 text-[0.9375rem]"
						disabled={chosen.length === 0}
						onclick={clearAll}>Temizle</Button
					>
					<Sheet.Close>
						{#snippet child({ props })}
							<Button {...props} size="lg" class="h-11 flex-1 px-5 text-[0.9375rem]"
								>{data.games.length} oyunu göster</Button
							>
						{/snippet}
					</Sheet.Close>
				</Sheet.Footer>
			</Sheet.Content>
		</Sheet.Root>

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
						<span class="max-sm:sr-only">Sırala:</span>
						{sortLabel}
					</Button>
				{/snippet}
			</DropdownMenu.Trigger>
			<DropdownMenu.Content align="end" sideOffset={8} class="w-44 p-2">
				<DropdownMenu.RadioGroup
					value={data.sort}
					onValueChange={(value) => update({ sirala: value })}
				>
					{#each sorts as sort (sort.key)}
						<DropdownMenu.RadioItem value={sort.key} class="h-9 rounded-xl px-2 font-medium"
							>{sort.label}</DropdownMenu.RadioItem
						>
					{/each}
				</DropdownMenu.RadioGroup>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</div>

	{#if filtered}
		<ul class="mt-4 flex flex-wrap items-center gap-2" aria-label="Seçili filtreler">
			{#each chosen as item (item.id)}
				<li>
					<button
						type="button"
						data-slot="choice"
						onclick={item.remove}
						aria-label="{item.group}: {item.label} filtresini kaldır"
						class="inline-flex min-h-9 cursor-pointer items-center gap-1.5 bg-muted pr-2.5 pl-3.5 text-[0.8125rem] font-medium text-primary outline-none hover:bg-muted/70 focus-visible:ring-[3px] focus-visible:ring-ring/50"
					>
						{item.label}
						<XIcon class="size-3.5" aria-hidden="true" />
					</button>
				</li>
			{/each}
			<li>
				<Button variant="ghost" size="sm" class="h-9 px-3 text-[0.8125rem]" onclick={clearAll}
					>Filtreleri temizle</Button
				>
			</li>
		</ul>
	{/if}

	{#if cards.length}
		<ul
			class="mt-8 grid grid-cols-2 gap-x-4 gap-y-8 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6"
		>
			{#each cards as game (game.slug)}
				<li
					animate:flip={{ duration: reducedMotion.current ? 0 : 320 }}
					in:fade={{ duration: reducedMotion.current ? 0 : 200 }}
				>
					<GameCard {game} />
				</li>
			{/each}
		</ul>
	{:else}
		<div class="mt-12 flex min-h-72 flex-col items-center justify-center gap-3 px-6 text-center">
			<span
				class="flex size-14 items-center justify-center rounded-full bg-muted text-muted-foreground"
			>
				<SearchXIcon class="size-6" aria-hidden="true" />
			</span>
			<p class="text-[0.9375rem] font-medium text-primary">Bu filtrelere uyan oyun bulamadık</p>
			<p class="text-sm text-muted-foreground">Birkaç filtreyi kaldırmayı dene.</p>
			<Button variant="outline" size="lg" class="mt-1 h-11 px-5" onclick={clearAll}
				>Filtreleri temizle</Button
			>
		</div>
	{/if}
</div>
