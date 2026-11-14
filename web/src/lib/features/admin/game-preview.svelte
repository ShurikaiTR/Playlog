<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { formatRelease, type DatePrecision } from '#lib/features/games/games.js';
	import type { GameFormValues } from './game-form.js';
	import type { PlatformOption } from '#lib/server/admin.js';
	import StatusBadge from './status-badge.svelte';
	import ClockIcon from '@lucide/svelte/icons/clock';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import ImageOffIcon from '@lucide/svelte/icons/image-off';

	// A live look at the game as visitors will see it: the page header (backdrop, logo, poster)
	// and the grid card. It reads the form's values directly, so it follows every keystroke.
	let { values, platforms }: { values: GameFormValues; platforms: PlatformOption[] } = $props();

	// An address that fails to load is shown as missing instead of as a broken image.
	let failed = $state<string[]>([]);
	const usable = (url: string) =>
		url.startsWith('https://') && !failed.includes(url) ? url : null;
	const fail = (url: string) => (failed = [...failed, url]);

	const cover = $derived(usable(values.coverUrl));
	const backdrop = $derived(usable(values.backdropUrl));
	const logo = $derived(usable(values.logoUrl));

	const title = $derived(values.title.trim() || 'Untitled game');
	const list = (text: string) =>
		text
			.split(',')
			.map((item) => item.trim())
			.filter(Boolean);
	const genres = $derived(list(values.genres).slice(0, 4));
	const developers = $derived(list(values.developers));
	const platformNames = $derived(
		values.platforms.map(
			(slug) => platforms.find((platform) => platform.slug === slug)?.name ?? slug
		)
	);
	const release = $derived(
		values.releaseDate
			? formatRelease(values.releaseDate, values.releasePrecision as DatePrecision)
			: 'TBA'
	);
	const hours = $derived(
		values.timeToBeat && Number(values.timeToBeat) > 0
			? Math.max(1, Math.round(Number(values.timeToBeat) / 60))
			: null
	);
	const meta = $derived(
		[genres[0], values.releaseDate ? values.releaseDate.slice(0, 4) : null]
			.filter(Boolean)
			.join(' · ')
	);
</script>

{#snippet poster(extra: string)}
	<div
		class="relative aspect-2/3 overflow-hidden rounded-xl bg-muted shadow-[0_6px_18px_-10px_rgb(0_0_0/0.5)] ring-1 ring-black/8 dark:ring-white/10 {extra}"
	>
		{#if cover}
			<img
				src={cover}
				alt=""
				class="size-full object-cover"
				onerror={() => fail(cover)}
				decoding="async"
			/>
		{:else}
			<div class="flex size-full flex-col justify-end gap-2 p-3 text-muted-foreground">
				<ImageOffIcon class="size-5" aria-hidden="true" />
				<span class="line-clamp-3 text-footnote font-medium">{title}</span>
			</div>
		{/if}
	</div>
{/snippet}

<div class="grid gap-4">
	<p class="text-footnote font-medium tracking-caps text-muted-foreground uppercase">Preview</p>

	<!-- The game page header -->
	<div class="overflow-hidden rounded-2xl bg-card ring-1 ring-foreground/10">
		<div class="relative aspect-video bg-muted">
			{#if backdrop}
				<img
					src={backdrop}
					alt=""
					class="absolute inset-0 size-full object-cover"
					onerror={() => fail(backdrop)}
					decoding="async"
				/>
			{:else}
				<div
					class="absolute inset-0 bg-gradient-to-br from-muted to-accent"
					aria-hidden="true"
				></div>
			{/if}
			<div
				class="absolute inset-0 bg-gradient-to-t from-black/70 via-black/10 to-transparent"
				aria-hidden="true"
			></div>
			<div class="absolute end-3 top-3"><StatusBadge status={values.status} /></div>
			{#if logo}
				<img
					src={logo}
					alt=""
					class="absolute start-4 bottom-4 max-h-14 max-w-[55%] object-contain drop-shadow"
					onerror={() => fail(logo)}
					decoding="async"
				/>
			{/if}
		</div>

		<div class="flex gap-4 px-4 pb-5">
			<div class="-mt-12 w-24 shrink-0">{@render poster('')}</div>
			<div class="min-w-0 pt-3">
				<h3 class="text-title-3 leading-tight text-primary">{title}</h3>
				{#if developers.length}
					<p class="truncate text-footnote text-muted-foreground">{developers.join(', ')}</p>
				{/if}
				<div
					class="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-footnote text-muted-foreground"
				>
					<span class="inline-flex items-center gap-1.5">
						<CalendarIcon class="size-3.5" aria-hidden="true" />{release}
					</span>
					{#if hours}
						<span class="inline-flex items-center gap-1.5">
							<ClockIcon class="size-3.5" aria-hidden="true" />~{hours} h
						</span>
					{/if}
				</div>
			</div>
		</div>

		<div class="grid gap-3 px-4 pb-5">
			{#if genres.length || platformNames.length}
				<div class="flex flex-wrap gap-1.5">
					{#each genres as genre (genre)}
						<Badge variant="secondary">{genre}</Badge>
					{/each}
					{#each platformNames as name (name)}
						<Badge variant="outline">{name}</Badge>
					{/each}
				</div>
			{/if}
			<p class="line-clamp-4 text-callout text-muted-foreground">
				{values.summary.trim() || 'The summary shows here, on cards and in search results.'}
			</p>
		</div>
	</div>

	<!-- The card in the games grid -->
	<div class="flex items-end gap-4 rounded-2xl bg-card p-4 ring-1 ring-foreground/10">
		<div class="w-28 shrink-0">{@render poster('')}</div>
		<div class="min-w-0 pb-1">
			<p class="truncate text-callout font-medium text-primary">{title}</p>
			{#if meta}<p class="truncate text-footnote text-muted-foreground">{meta}</p>{/if}
			<p class="mt-2 text-footnote text-muted-foreground">How it looks in the games grid.</p>
		</div>
	</div>
</div>
