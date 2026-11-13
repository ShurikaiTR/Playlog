<script lang="ts">
	import { goto } from '$app/navigation';
	import SearchIcon from '@lucide/svelte/icons/search';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Command from '#lib/components/ui/command/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Empty from '#lib/components/ui/empty/index.js';
	import { Kbd } from '#lib/components/ui/kbd/index.js';
	import { searchGames, type GameResult } from './search.js';
	import { searchDialog } from './search-state.svelte.js';
	import SearchPreview from './search-preview.svelte';

	let query = $state('');
	let input = $state<HTMLInputElement | null>(null);
	let highlighted = $state('');
	let games = $state<GameResult[]>([]);
	// What the list on screen answers; it trails the input while a request is on its way.
	let answered = $state<string | null>(null);

	const typed = $derived(query.trim().length > 0);
	const loading = $derived(answered !== query.trim());

	// Results follow the input after a short pause; a newer query cancels the one before it.
	$effect(() => {
		if (!searchDialog.open) return;
		const wanted = query.trim();
		const controller = new AbortController();
		const timer = setTimeout(
			async () => {
				const results = await searchGames(wanted, controller.signal);
				if (controller.signal.aborted) return;
				games = results;
				answered = wanted;
				// Enter opens the best match straight away.
				highlighted = results[0] ? `game:${results[0].slug}` : '';
			},
			wanted ? 180 : 0
		);
		return () => {
			clearTimeout(timer);
			controller.abort();
		};
	});

	// The preview follows the highlighted row, falling back to the first one.
	const selectedGame = $derived(games.find((g) => `game:${g.slug}` === highlighted) ?? games[0]);

	// The query only lives while the dialog is open.
	function onOpenChange(open: boolean) {
		if (!open) query = '';
	}

	function close() {
		searchDialog.open = false;
		onOpenChange(false);
	}

	const gameHref = (game: GameResult) => `/games/${game.slug}`;

	function go(href: string) {
		close();
		goto(href);
	}

	function onWindowKeydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
			event.preventDefault();
			searchDialog.open = true;
		}
	}

	const rowClass =
		'cursor-pointer gap-3 rounded-xl p-2 text-left data-selected:bg-muted [&>svg:last-child]:hidden';
</script>

<svelte:window onkeydown={onWindowKeydown} />

<Dialog.Root bind:open={searchDialog.open} {onOpenChange}>
	<Dialog.Content
		showCloseButton={false}
		onOpenAutoFocus={() => input?.focus({ preventScroll: true })}
		class="flex h-[min(720px,calc(100dvh-2rem))] w-[min(1040px,calc(100vw-2rem))] max-w-none flex-col gap-0 overflow-hidden p-0 sm:max-w-none motion-reduce:data-open:animate-none motion-reduce:data-closed:animate-none"
	>
		<Dialog.Title class="sr-only">Search</Dialog.Title>
		<Dialog.Description class="sr-only">Search the game catalog.</Dialog.Description>

		<Command.Root
			bind:value={highlighted}
			shouldFilter={false}
			loop
			label="Search results"
			class="min-h-0 flex-1 rounded-none bg-transparent p-0"
		>
			<header
				class="flex shrink-0 flex-col gap-2 border-b border-border px-4 py-3 sm:flex-row sm:items-center sm:gap-3 sm:px-5"
			>
				<div
					class="flex min-w-0 flex-1 items-center gap-2 [&_[data-slot=command-input-wrapper]]:flex-1 [&_[data-slot=command-input-wrapper]]:border-0 [&_[data-slot=command-input-wrapper]]:px-0"
				>
					<Command.Input
						bind:ref={input}
						bind:value={query}
						placeholder="Search games"
						aria-label="Search games"
						class="h-10 text-base pointer-coarse:h-11"
					/>
					<!-- Touch screens have no Esc key, so they get an explicit way out. -->
					<Button
						variant="ghost"
						class="hidden h-11 shrink-0 cursor-pointer rounded-xl px-3 text-callout pointer-coarse:inline-flex"
						onclick={close}>Cancel</Button
					>
				</div>
			</header>

			<div class="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[40%_1fr]">
				<section class="flex min-h-0 flex-col border-border md:border-r">
					<div class="flex h-11 shrink-0 items-center justify-between px-4 text-footnote sm:px-5">
						<span class="font-[550] text-foreground">
							{typed ? 'Games' : 'New releases'}
						</span>
						<span class="text-muted-foreground" aria-live="polite">
							{typed && !loading
								? `${games.length} ${games.length === 1 ? 'result' : 'results'}`
								: ''}
						</span>
					</div>

					<Command.List class="max-h-none min-h-0 flex-1 scroll-py-2 px-2 pb-3">
						{#if games.length === 0}
							{#if !loading}
								<Empty.Root class="min-h-64 border-0 p-6">
									<Empty.Media variant="icon"><SearchIcon /></Empty.Media>
									<Empty.Header>
										<Empty.Title class="text-headline">No results</Empty.Title>
										<Empty.Description class="text-callout">Try another title.</Empty.Description>
									</Empty.Header>
								</Empty.Root>
							{/if}
						{:else}
							<Command.Group class="p-0">
								{#each games as game (game.slug)}
									<Command.Item
										value="game:{game.slug}"
										class={rowClass}
										onSelect={() => go(gameHref(game))}
									>
										<span class="h-14 w-10 shrink-0 overflow-hidden rounded-lg bg-muted">
											{#if game.cover}
												<img
													src={game.cover}
													alt=""
													class="size-full object-cover"
													loading="lazy"
												/>
											{/if}
										</span>
										<span class="min-w-0 flex-1">
											<span class="block truncate text-callout font-[550]">{game.title}</span>
											<span class="block truncate text-footnote text-muted-foreground">
												{[game.genres[0], game.year].filter(Boolean).join(' • ')}
											</span>
										</span>
									</Command.Item>
								{/each}
							</Command.Group>
						{/if}
					</Command.List>
				</section>

				<SearchPreview game={selectedGame} {gameHref} onnavigate={close} />
			</div>

			<footer
				class="flex h-11 shrink-0 items-center gap-4 border-t border-border px-4 text-footnote text-muted-foreground sm:px-5 pointer-coarse:hidden"
			>
				<span class="flex items-center gap-1.5"><Kbd>↑</Kbd><Kbd>↓</Kbd> Navigate</span>
				<span class="flex items-center gap-1.5"><Kbd>↵</Kbd> Open</span>
				<span class="ms-auto flex items-center gap-1.5"><Kbd>Esc</Kbd> Close</span>
			</footer>
		</Command.Root>
	</Dialog.Content>
</Dialog.Root>
