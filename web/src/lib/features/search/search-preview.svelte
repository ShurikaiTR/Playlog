<script lang="ts">
	import SearchIcon from '@lucide/svelte/icons/search';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Empty from '#lib/components/ui/empty/index.js';
	import { ScrollArea } from '#lib/components/ui/scroll-area/index.js';
	import type { GameResult } from './search.js';

	// The right-hand pane of the search dialog. Hidden on small screens, where a tap opens the result directly.
	let {
		game,
		gameHref,
		onnavigate
	}: {
		game: GameResult | undefined;
		gameHref: (game: GameResult) => string;
		onnavigate: () => void;
	} = $props();

	const buttonClass = 'w-full cursor-pointer rounded-full text-callout pointer-coarse:h-11';
</script>

<ScrollArea class="hidden min-h-0 bg-muted/20 md:block">
	<section class="flex min-h-full flex-col" aria-label="Preview">
		{#if game}
			<div class="relative h-40 shrink-0 bg-muted">
				{#if game.backdrop}<img src={game.backdrop} alt="" class="size-full object-cover" />{/if}
				<div
					class="absolute inset-0 bg-linear-to-t from-background via-background/40 to-transparent"
				></div>
			</div>
			<div class="relative -mt-20 flex flex-1 flex-col px-8 pb-6">
				<div class="flex items-end gap-5">
					<div
						class="h-40 w-28 shrink-0 overflow-hidden rounded-2xl bg-muted shadow-lg ring-1 ring-foreground/10"
					>
						{#if game.cover}<img src={game.cover} alt="" class="size-full object-cover" />{/if}
					</div>
					<div class="min-w-0 pb-1">
						<h2 class="text-title-3 text-balance">{game.title}</h2>
						<p class="mt-1 text-footnote text-muted-foreground">
							{[game.developer, game.year].filter(Boolean).join(' • ')}
						</p>
					</div>
				</div>
				{#if game.genres.length}
					<div class="mt-5 flex flex-wrap gap-1.5">
						{#each game.genres as genre (genre)}
							<Badge variant="secondary">{genre}</Badge>
						{/each}
					</div>
				{/if}
				{#if game.summary}
					<p class="mt-4 line-clamp-5 text-callout text-muted-foreground">{game.summary}</p>
				{/if}
				<div class="mt-auto pt-6">
					<Button href={gameHref(game)} class={buttonClass} onclick={onnavigate}>View game</Button>
				</div>
			</div>
		{:else}
			<Empty.Root class="m-auto border-0 p-8">
				<Empty.Media variant="icon"><SearchIcon /></Empty.Media>
				<Empty.Header>
					<Empty.Title class="text-headline">Preview</Empty.Title>
					<Empty.Description class="text-callout"
						>Highlight a result to see its details here.</Empty.Description
					>
				</Empty.Header>
			</Empty.Root>
		{/if}
	</section>
</ScrollArea>
