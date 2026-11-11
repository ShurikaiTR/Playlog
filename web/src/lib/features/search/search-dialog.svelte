<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import SearchIcon from '@lucide/svelte/icons/search';
	import UserRoundIcon from '@lucide/svelte/icons/user-round';
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Command from '#lib/components/ui/command/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Empty from '#lib/components/ui/empty/index.js';
	import { Kbd } from '#lib/components/ui/kbd/index.js';
	import * as Tabs from '#lib/components/ui/tabs/index.js';
	import {
		searchGames,
		searchUsers,
		suggestedGames,
		suggestedUsers,
		type GameResult,
		type UserResult
	} from './search.js';
	import { searchDialog } from './search-state.svelte.js';
	import SearchPreview from './search-preview.svelte';

	type Tab = 'games' | 'users';

	let query = $state('');
	let tab = $state<Tab>('games');
	let highlighted = $state('');

	const typed = $derived(query.trim().length > 0);
	const games = $derived(typed ? searchGames(query) : suggestedGames());
	const users = $derived(typed ? searchUsers(query) : suggestedUsers());
	const count = $derived(tab === 'games' ? games.length : users.length);

	// The preview follows the highlighted row, falling back to the first one.
	const selectedGame = $derived(
		tab === 'games' ? (games.find((g) => `game:${g.slug}` === highlighted) ?? games[0]) : undefined
	);
	const selectedUser = $derived(
		tab === 'users'
			? (users.find((u) => `user:${u.username}` === highlighted) ?? users[0])
			: undefined
	);

	// The query and tab only live while the dialog is open.
	function onOpenChange(open: boolean) {
		if (!open) {
			query = '';
			tab = 'games';
		}
	}

	function close() {
		searchDialog.open = false;
		onOpenChange(false);
	}

	function go(href: ReturnType<typeof resolve>) {
		close();
		goto(href);
	}

	const gameHref = (game: GameResult) => resolve('/(app)/oyun/[slug]', { slug: game.slug });
	const userHref = (user: UserResult) =>
		resolve('/(app)/profil/[kullanici]', { kullanici: user.username });
	const initial = (name: string) => name.charAt(0).toLocaleUpperCase('tr');

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
		class="flex h-[min(720px,calc(100dvh-2rem))] w-[min(1040px,calc(100vw-2rem))] max-w-none flex-col gap-0 overflow-hidden p-0 sm:max-w-none motion-reduce:data-open:animate-none motion-reduce:data-closed:animate-none"
	>
		<Dialog.Title class="sr-only">Ara</Dialog.Title>
		<Dialog.Description class="sr-only">Oyunlar ve kullanıcılar arasında ara.</Dialog.Description>

		<Command.Root
			bind:value={highlighted}
			shouldFilter={false}
			loop
			label="Arama sonuçları"
			class="min-h-0 flex-1 rounded-none bg-transparent p-0"
		>
			<header
				class="flex shrink-0 flex-col gap-2 border-b border-border px-4 py-3 sm:flex-row sm:items-center sm:gap-3 sm:px-5"
			>
				<div
					class="flex min-w-0 flex-1 items-center gap-2 [&_[data-slot=command-input-wrapper]]:flex-1 [&_[data-slot=command-input-wrapper]]:border-0 [&_[data-slot=command-input-wrapper]]:px-0"
				>
					<Command.Input
						bind:value={query}
						placeholder="Oyun veya kullanıcı ara"
						aria-label="Oyun veya kullanıcı ara"
						class="h-10 text-base pointer-coarse:h-11"
					/>
					<!-- Touch screens have no Esc key, so they get an explicit way out. -->
					<Button
						variant="ghost"
						class="hidden h-11 shrink-0 cursor-pointer rounded-xl px-3 text-callout pointer-coarse:inline-flex"
						onclick={close}>Vazgeç</Button
					>
				</div>
				<Tabs.Root bind:value={tab} class="shrink-0">
					<Tabs.List class="h-9 w-full rounded-xl sm:w-auto pointer-coarse:h-11">
						<Tabs.Trigger
							value="games"
							class="cursor-pointer rounded-lg px-3 text-footnote pointer-coarse:h-11"
							>Oyunlar</Tabs.Trigger
						>
						<Tabs.Trigger
							value="users"
							class="cursor-pointer rounded-lg px-3 text-footnote pointer-coarse:h-11"
							>Kullanıcılar</Tabs.Trigger
						>
					</Tabs.List>
				</Tabs.Root>
			</header>

			<div class="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[40%_1fr]">
				<section class="flex min-h-0 flex-col border-border md:border-r">
					<div class="flex h-11 shrink-0 items-center justify-between px-4 text-footnote sm:px-5">
						<span class="font-[550] text-foreground">
							{typed ? (tab === 'games' ? 'Oyunlar' : 'Kullanıcılar') : 'Öneriler'}
						</span>
						<span class="text-muted-foreground" aria-live="polite">
							{typed ? `${count} sonuç` : ''}
						</span>
					</div>

					<Command.List class="max-h-none min-h-0 flex-1 scroll-py-2 px-2 pb-3">
						{#if count === 0}
							<Empty.Root class="min-h-64 border-0 p-6">
								<Empty.Media variant="icon">
									{#if tab === 'games'}<SearchIcon />{:else}<UserRoundIcon />{/if}
								</Empty.Media>
								<Empty.Header>
									<Empty.Title class="text-headline">Sonuç bulunamadı</Empty.Title>
									<Empty.Description class="text-callout">
										{tab === 'games'
											? 'Başka bir oyun adı, tür ya da geliştirici dene.'
											: 'Bir isim ya da kullanıcı adı dene.'}
									</Empty.Description>
								</Empty.Header>
							</Empty.Root>
						{:else if tab === 'games'}
							<Command.Group class="p-0">
								{#each games as game (game.slug)}
									<Command.Item
										value="game:{game.slug}"
										class={rowClass}
										onSelect={() => go(gameHref(game))}
									>
										<span class="h-14 w-10 shrink-0 overflow-hidden rounded-lg bg-muted">
											<img src={game.cover} alt="" class="size-full object-cover" loading="lazy" />
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
						{:else}
							<Command.Group class="p-0">
								{#each users as user (user.username)}
									<Command.Item
										value="user:{user.username}"
										class={rowClass}
										onSelect={() => go(userHref(user))}
									>
										<Avatar.Root class="size-10 shrink-0">
											{#if user.image}<Avatar.Image src={user.image} alt="" />{/if}
											<Avatar.Fallback class="text-callout font-semibold"
												>{initial(user.name)}</Avatar.Fallback
											>
										</Avatar.Root>
										<span class="min-w-0 flex-1">
											<span class="block truncate text-callout font-[550]">{user.name}</span>
											<span class="block truncate text-footnote text-muted-foreground"
												>@{user.username}</span
											>
										</span>
									</Command.Item>
								{/each}
							</Command.Group>
						{/if}
					</Command.List>
				</section>

				<SearchPreview
					game={selectedGame}
					user={selectedUser}
					{gameHref}
					{userHref}
					onnavigate={close}
				/>
			</div>

			<footer
				class="flex h-11 shrink-0 items-center gap-4 border-t border-border px-4 text-footnote text-muted-foreground sm:px-5 pointer-coarse:hidden"
			>
				<span class="flex items-center gap-1.5"><Kbd>↑</Kbd><Kbd>↓</Kbd> Gez</span>
				<span class="flex items-center gap-1.5"><Kbd>↵</Kbd> Aç</span>
				<span class="ms-auto flex items-center gap-1.5"><Kbd>Esc</Kbd> Kapat</span>
			</footer>
		</Command.Root>
	</Dialog.Content>
</Dialog.Root>
