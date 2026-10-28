<script lang="ts">
	import { goto } from '$app/navigation';
	import GameCard from '#lib/components/game-card.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Sheet from '#lib/components/ui/sheet/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { sorts } from '#lib/games.js';
	import { cn } from '#lib/utils.js';
	import ArrowUpDownIcon from '@lucide/svelte/icons/arrow-up-down';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import SearchIcon from '@lucide/svelte/icons/search';
	import SlidersHorizontalIcon from '@lucide/svelte/icons/sliders-horizontal';
	import SearchXIcon from '@lucide/svelte/icons/search-x';
	import XIcon from '@lucide/svelte/icons/x';
	import { flip } from 'svelte/animate';
	import { fade } from 'svelte/transition';
	import { MediaQuery } from 'svelte/reactivity';

	let { data } = $props();

	type Key = 'platform' | 'mod' | 'tur' | 'tema' | 'sure' | 'yil' | 'yas' | 'puan';
	const playtimeHints: Record<string, string> = {
		Kısa: 'Kısa · 10 saate kadar',
		Orta: 'Orta · 10–30 saat',
		Uzun: 'Uzun · 30+ saat'
	};
	const groups: {
		key: Key;
		label: string;
		single?: boolean;
		format?: (value: string) => string;
	}[] = [
		{ key: 'platform', label: 'Platform' },
		{ key: 'mod', label: 'Oyun modu' },
		{ key: 'tur', label: 'Türler' },
		{ key: 'tema', label: 'Tema' },
		{ key: 'sure', label: 'Oynama süresi', format: (value) => playtimeHints[value] },
		{ key: 'yil', label: 'Çıkış yılı' },
		{ key: 'yas', label: 'Yaş sınırı' },
		{ key: 'puan', label: 'En düşük puan', single: true, format: (value) => `${value}+` }
	];
	const none: Record<Key, string[]> = {
		platform: [],
		mod: [],
		tur: [],
		tema: [],
		sure: [],
		yil: [],
		yas: [],
		puan: []
	};

	function update(patch: Partial<Record<Key, string[]>> & { q?: string; sirala?: string }) {
		const params = new URLSearchParams();
		const q = patch.q ?? data.q;
		if (q) params.set('q', q);
		for (const { key } of groups)
			for (const value of patch[key] ?? data.filters[key]) params.append(key, value);
		const sirala = patch.sirala ?? data.sort;
		if (sirala !== 'populer') params.set('sirala', sirala);
		const search = params.toString();
		goto(search ? `?${search}` : '?', { replace: true, reset: false });
	}

	function toggle(key: Key, value: string, single = false) {
		const current = data.filters[key];
		update({
			[key]: current.includes(value)
				? current.filter((v) => v !== value)
				: single
					? [value]
					: [...current, value]
		});
	}

	// The search box is local state that follows the address (back button) and writes to it after a pause.
	let query = $state('');
	$effect(() => {
		query = data.q;
	});
	let timer: ReturnType<typeof setTimeout>;
	function onInput() {
		clearTimeout(timer);
		timer = setTimeout(() => update({ q: query.trim() }), 250);
	}

	const chosen = $derived(
		groups.flatMap((group) =>
			data.filters[group.key].map((value) => ({
				key: group.key,
				value,
				label: group.format?.(value) ?? value,
				group: group.label
			}))
		)
	);
	const filtered = $derived(chosen.length > 0 || Boolean(data.q));
	const clearAll = () => {
		query = '';
		update({ ...none, q: '' });
	};

	const reducedMotion = new MediaQuery('(prefers-reduced-motion: reduce)');
	// Each card shows its first genre and score under the title.
	const cards = $derived(
		data.games.map((game) => ({ ...game, meta: `${game.genres[0]} · ★ ${game.score.toFixed(1)}` }))
	);
	const sortLabel = $derived(sorts.find((sort) => sort.key === data.sort)?.label ?? '');

	// Only Platform starts open; the other sections unfold on demand.
	let openGroups = $state<Record<Key, boolean>>({
		platform: true,
		mod: false,
		tur: false,
		tema: false,
		sure: false,
		yil: false,
		yas: false,
		puan: false
	});

	const optionClass =
		'focus-visible:ring-ring/50 inline-flex min-h-9 cursor-pointer items-center border px-3.5 text-[0.8125rem] font-medium outline-none focus-visible:ring-[3px]';
</script>

<svelte:head>
	<title>Tüm oyunlar | Playlog</title>
	<meta
		name="description"
		content="Playlog'daki tüm oyunlara göz at: ada, platforma, türe, temaya ve puana göre ara, süz ve sırala."
	/>
</svelte:head>

{#snippet filterSections()}
	<div class="divide-y border-y">
		{#each groups as group (group.key)}
			{@const picked = data.filters[group.key]}
			<details
				class="group/section"
				open={openGroups[group.key]}
				ontoggle={(event) => (openGroups[group.key] = event.currentTarget.open)}
			>
				<summary
					class="flex min-h-12 cursor-pointer list-none items-center gap-3 px-4 outline-none marker:hidden hover:bg-muted/50 focus-visible:bg-muted/50 [&::-webkit-details-marker]:hidden"
				>
					<span class="text-sm font-semibold text-primary">{group.label}</span>
					{#if picked.length > 0}
						<span
							class="inline-flex size-5 shrink-0 items-center justify-center rounded-full bg-primary text-[0.6875rem] font-semibold text-primary-foreground tabular-nums"
						>
							{picked.length}
						</span>
					{/if}
					<ChevronDownIcon
						class="ml-auto size-4 shrink-0 text-muted-foreground transition-transform group-open/section:rotate-180 motion-reduce:transition-none"
						aria-hidden="true"
					/>
				</summary>
				<div class="flex flex-wrap gap-2 px-4 pt-1 pb-4">
					{#each data.options[group.key] as option (option)}
						{@const on = picked.includes(option)}
						<button
							type="button"
							data-slot="choice"
							aria-pressed={on}
							onclick={() => toggle(group.key, option, group.single)}
							class={cn(
								optionClass,
								on
									? 'border-transparent bg-primary text-primary-foreground'
									: 'text-primary hover:bg-muted'
							)}
						>
							{group.format?.(option) ?? option}
						</button>
					{/each}
				</div>
			</details>
		{/each}
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

	<!-- One glass toolbar that follows the scroll: search, filters, order and a rail of quick genre chips. -->
	<div
		class="sticky top-3 z-20 rounded-3xl bg-background/80 p-2 shadow-sm ring-1 ring-black/8 backdrop-blur-xl dark:ring-white/10"
	>
		<div class="flex items-center gap-2">
			<div class="relative min-w-0 flex-1">
				<SearchIcon
					class="pointer-events-none absolute top-1/2 left-4 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden="true"
				/>
				<Input
					type="search"
					bind:value={query}
					oninput={onInput}
					placeholder="Oyun veya geliştirici ara"
					aria-label="Oyun veya geliştirici ara"
					autocomplete="off"
					class="h-11 rounded-full border-transparent bg-muted/60 pr-4 pl-11 text-[0.9375rem]"
				/>
			</div>

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
							onclick={() => update(none)}>Temizle</Button
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

		<div
			class="mt-2 flex [scrollbar-width:none] gap-1.5 overflow-x-auto px-1 pb-1 [&::-webkit-scrollbar]:hidden"
			role="group"
			aria-label="Türler"
		>
			{#each data.options.tur as genre (genre)}
				{@const on = data.filters.tur.includes(genre)}
				<button
					type="button"
					data-slot="choice"
					aria-pressed={on}
					onclick={() => toggle('tur', genre)}
					class={cn(
						'inline-flex min-h-9 shrink-0 cursor-pointer items-center px-3.5 text-[0.8125rem] font-medium outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50',
						on ? 'bg-primary text-primary-foreground' : 'text-primary hover:bg-muted'
					)}
				>
					{genre}
				</button>
			{/each}
		</div>
	</div>

	{#if filtered}
		<ul class="mt-4 flex flex-wrap items-center gap-2" aria-label="Seçili filtreler">
			{#each chosen as item (item.key + item.value)}
				<li>
					<button
						type="button"
						data-slot="choice"
						onclick={() => toggle(item.key, item.value)}
						aria-label="{item.group}: {item.label} filtresini kaldır"
						class="inline-flex min-h-9 cursor-pointer items-center gap-1.5 bg-muted pr-2.5 pl-3.5 text-[0.8125rem] font-medium text-primary outline-none hover:bg-muted/70 focus-visible:ring-[3px] focus-visible:ring-ring/50"
					>
						{item.key === 'puan' ? `Puan ${item.label}` : item.label}
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
			<p class="text-[0.9375rem] font-medium text-primary">Aramana uyan oyun bulamadık</p>
			<p class="text-sm text-muted-foreground">Başka bir ad dene ya da birkaç filtreyi kaldır.</p>
			<Button variant="outline" size="lg" class="mt-1 h-11 px-5" onclick={clearAll}
				>Filtreleri temizle</Button
			>
		</div>
	{/if}
</div>
