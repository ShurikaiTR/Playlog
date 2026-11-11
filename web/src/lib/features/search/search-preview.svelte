<script lang="ts">
	import SearchIcon from '@lucide/svelte/icons/search';
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Empty from '#lib/components/ui/empty/index.js';
	import { ScrollArea } from '#lib/components/ui/scroll-area/index.js';
	import type { GameResult, UserResult } from './search.js';

	// The right-hand pane of the search dialog. Hidden on small screens, where a tap opens the result directly.
	let {
		game,
		user,
		gameHref,
		userHref,
		onnavigate
	}: {
		game: GameResult | undefined;
		user: UserResult | undefined;
		gameHref: (game: GameResult) => string;
		userHref: (user: UserResult) => string;
		onnavigate: () => void;
	} = $props();

	const buttonClass = 'w-full cursor-pointer rounded-full text-callout pointer-coarse:h-11';
</script>

<ScrollArea class="hidden min-h-0 bg-muted/20 md:block">
	<section class="flex min-h-full flex-col" aria-label="Önizleme">
		{#if game}
			<div class="relative h-40 shrink-0 bg-muted">
				<img src={game.backdrop} alt="" class="size-full object-cover" />
				<div
					class="absolute inset-0 bg-linear-to-t from-background via-background/40 to-transparent"
				></div>
			</div>
			<div class="relative -mt-20 flex flex-1 flex-col px-8 pb-6">
				<div class="flex items-end gap-5">
					<div
						class="h-40 w-28 shrink-0 overflow-hidden rounded-2xl bg-muted shadow-lg ring-1 ring-foreground/10"
					>
						<img src={game.cover} alt="" class="size-full object-cover" />
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
				{#if game.description}
					<p class="mt-4 line-clamp-5 text-callout text-muted-foreground">{game.description}</p>
				{/if}
				<div class="mt-auto pt-6">
					<Button href={gameHref(game)} class={buttonClass} onclick={onnavigate}>Oyuna git</Button>
				</div>
			</div>
		{:else if user}
			<div class="relative h-32 shrink-0 bg-muted">
				<img src={user.cover} alt="" class="size-full object-cover" />
				<div class="absolute inset-0 bg-linear-to-t from-background/70 to-transparent"></div>
			</div>
			<div class="relative -mt-10 flex flex-1 flex-col px-8 pb-6">
				<Avatar.Root class="size-20 ring-4 ring-background">
					{#if user.image}<Avatar.Image src={user.image} alt="" />{/if}
					<Avatar.Fallback class="bg-foreground text-title-2 text-background"
						>{user.name.charAt(0).toLocaleUpperCase('tr')}</Avatar.Fallback
					>
				</Avatar.Root>
				<h2 class="mt-3 text-title-3">{user.name}</h2>
				<p class="text-footnote text-muted-foreground">@{user.username}</p>
				{#if user.bio}
					<p class="mt-4 line-clamp-5 text-callout text-muted-foreground">{user.bio}</p>
				{/if}
				<div class="mt-auto pt-6">
					<Button href={userHref(user)} class={buttonClass} onclick={onnavigate}>Profile git</Button
					>
				</div>
			</div>
		{:else}
			<Empty.Root class="m-auto border-0 p-8">
				<Empty.Media variant="icon"><SearchIcon /></Empty.Media>
				<Empty.Header>
					<Empty.Title class="text-headline">Önizleme</Empty.Title>
					<Empty.Description class="text-callout"
						>Bir sonucun üzerine gel, ayrıntıları burada görünsün.</Empty.Description
					>
				</Empty.Header>
			</Empty.Root>
		{/if}
	</section>
</ScrollArea>
