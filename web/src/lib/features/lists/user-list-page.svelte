<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import * as Empty from '#lib/components/ui/empty/index.js';
	import * as InputGroup from '#lib/components/ui/input-group/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import { listCover, saveListDetails, setGame, type UserList } from './user-lists.svelte.js';
	import { toast } from 'svelte-sonner';
	import ArrowDownWideNarrowIcon from '@lucide/svelte/icons/arrow-down-wide-narrow';
	import ArrowUpRightIcon from '@lucide/svelte/icons/arrow-up-right';
	import ClockIcon from '@lucide/svelte/icons/clock-3';
	import GamepadIcon from '@lucide/svelte/icons/gamepad-2';
	import GlobeIcon from '@lucide/svelte/icons/globe';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import LockIcon from '@lucide/svelte/icons/lock';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import SearchIcon from '@lucide/svelte/icons/search';
	import XIcon from '@lucide/svelte/icons/x';

	let { list }: { list: UserList } = $props();

	const copy = {
		played: {
			removeLabel: 'Remove from list',
			emptyTitle: 'You haven’t played anything yet.',
			emptyDescription: 'Add the games you have played from the save menu on a game’s page.',
			meta: 'Games you have played.'
		},
		play_later: {
			removeLabel: 'Remove from list',
			emptyTitle: 'Nothing is waiting to be played.',
			emptyDescription: 'Save a game that catches your eye and it will wait for you here.',
			meta: 'Games you saved to play later.'
		},
		custom: {
			removeLabel: 'Remove from list',
			emptyTitle: 'This list is empty.',
			emptyDescription: 'Add games from a game’s page with the Add to library menu.',
			meta: 'A list of your own.'
		}
	} as const;

	const text = $derived(copy[list.kind]);
	const sortOptions = [
		{ value: 'recent', label: 'Recently added' },
		{ value: 'title', label: 'Title (A–Z)' }
	];

	let query = $state('');
	let selectedSort = $state('recent');
	let flippedSlug = $state<string | null>(null);
	let editOpen = $state(false);
	let editMode = $state<'details' | 'cover'>('details');
	let draftName = $state('');
	let draftDescription = $state('');
	let draftVisibility = $state<string>('private');
	let draftCover = $state<string | null>(null);
	let nameError = $state('');

	const selectedSortLabel = $derived(
		sortOptions.find((option) => option.value === selectedSort)?.label
	);
	const visibleItems = $derived.by(() => {
		const search = query.trim().toLocaleLowerCase('en');
		return list.items
			.filter((game) => game.title.toLocaleLowerCase('en').includes(search))
			.toSorted((a, b) => {
				if (selectedSort === 'title') return a.title.localeCompare(b.title, 'en');
				return b.added - a.added;
			});
	});

	function openSettings(mode: 'details' | 'cover' = 'details') {
		editMode = mode;
		draftName = list.name;
		draftDescription = list.description;
		draftVisibility = list.visibility;
		draftCover = list.coverSlug;
		nameError = '';
		editOpen = true;
	}

	async function saveSettings(event: SubmitEvent) {
		event.preventDefault();
		if (editMode === 'details') {
			if (!draftName.trim()) {
				nameError = 'Give the list a name.';
				return;
			}
			const saved = await saveListDetails(list, {
				name: draftName.trim(),
				description: draftDescription.trim(),
				visibility: draftVisibility === 'public' ? 'public' : 'private'
			});
			if (saved) {
				editOpen = false;
				toast('List settings updated');
			}
			return;
		}
		const saved = await saveListDetails(list, {
			name: list.name,
			description: list.description,
			visibility: list.visibility,
			coverSlug: draftCover
		});
		if (saved) {
			editOpen = false;
			toast('Cover updated');
		}
	}

	async function removeItem(game: UserList['items'][number]) {
		// `added` is dropped on purpose: the summary is the game without its list timestamp.
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		const { added, ...summary } = game;
		flippedSlug = null;
		if (!(await setGame(list, summary, false))) return;
		toast(`${game.title} removed from the list`, {
			action: { label: 'Undo', onClick: () => void setGame(list, summary, true) }
		});
	}
</script>

{#snippet listIcon(className: string)}
	{#if list.kind === 'played'}<CircleCheckIcon class={className} />
	{:else if list.kind === 'play_later'}<ClockIcon class={className} />
	{:else}<GamepadIcon class={className} />{/if}
{/snippet}

<svelte:head>
	<title>{list.name} | Playthru</title>
</svelte:head>

<div class="flex min-w-0 flex-col pb-16">
	<section class="list-hero relative isolate min-w-0" aria-labelledby="list-title">
		<!-- Full-bleed cover like the profile and game pages. -->
		<div
			class="pointer-events-none absolute inset-0 -z-10 overflow-hidden"
			aria-hidden="true"
			style="mask-image: linear-gradient(to bottom, black 55%, transparent 100%)"
		>
			{#if listCover(list)}
				<img
					src={listCover(list)}
					alt=""
					width="1600"
					height="560"
					class="size-full object-cover object-right"
				/>
			{/if}
			<div class="list-scrim absolute inset-0"></div>
		</div>

		<div
			class="flex flex-col gap-6 px-4 pt-32 pb-2 sm:flex-row sm:items-end sm:gap-8 md:px-8 md:pt-40"
		>
			<div
				class="relative flex size-32 shrink-0 items-center justify-center overflow-hidden rounded-xl bg-muted shadow-sm ring-4 ring-background sm:size-40"
			>
				{@render listIcon('size-8 text-muted-foreground')}
			</div>
			<div class="min-w-0 flex-1 pb-1">
				<Badge variant="secondary" class="mb-3 gap-1.5">
					{#if list.visibility === 'private'}<LockIcon class="size-3.5" />Private list{:else}<GlobeIcon
							class="size-3.5"
						/>Public list{/if}
				</Badge>
				<div class="flex items-center gap-2">
					<h1
						id="list-title"
						class="min-w-0 text-[2rem] leading-tight font-medium tracking-[-0.025em] wrap-break-word text-primary md:text-[2.5rem]"
					>
						{list.name}
					</h1>
					<DropdownMenu.Root>
						<DropdownMenu.Trigger
							class={buttonVariants({
								variant: 'ghost',
								size: 'icon-sm',
								class:
									'shrink-0 cursor-pointer rounded-full text-muted-foreground hover:text-foreground'
							})}
							aria-label="List options"><PencilIcon class="size-4" /></DropdownMenu.Trigger
						>
						<DropdownMenu.Content align="start" class="w-48 rounded-xl p-1.5">
							<DropdownMenu.Item
								class="cursor-pointer rounded-lg"
								onclick={() => openSettings('details')}>Edit list</DropdownMenu.Item
							>
							<DropdownMenu.Item
								class="cursor-pointer rounded-lg"
								disabled={list.items.length === 0}
								onclick={() => openSettings('cover')}>Change cover</DropdownMenu.Item
							>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				</div>
				{#if list.description}
					<p
						class="mt-2 max-w-xl text-sm leading-6 wrap-break-word whitespace-pre-line text-muted-foreground"
					>
						{list.description}
					</p>
				{/if}
				<p class="mt-3 text-xs font-medium text-muted-foreground">
					{list.items.length} oyun · {text.meta}
				</p>
			</div>
		</div>
	</section>

	<section aria-label="List contents" class="mt-12 min-w-0 px-4 md:px-8">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<p class="text-sm font-medium tabular-nums" aria-live="polite">
				{visibleItems.length} / {list.items.length} oyun
			</p>
			<div class="flex w-full flex-col gap-2 sm:w-auto sm:flex-row sm:items-center">
				<InputGroup.Root class="h-10 rounded-xl sm:w-64">
					<InputGroup.Addon><SearchIcon /></InputGroup.Addon>
					<InputGroup.Input
						bind:value={query}
						aria-label="Search this list"
						placeholder="Search this list…"
					/>
					{#if query}
						<InputGroup.Addon align="inline-end">
							<InputGroup.Button
								size="icon-xs"
								class="cursor-pointer rounded-full"
								aria-label="Clear search"
								onclick={() => (query = '')}><XIcon /></InputGroup.Button
							>
						</InputGroup.Addon>
					{/if}
				</InputGroup.Root>
				<Select.Root type="single" bind:value={selectedSort}>
					<Select.Trigger class="h-10 w-48" aria-label="Sort: {selectedSortLabel}">
						<span data-slot="select-value"
							><ArrowDownWideNarrowIcon class="text-muted-foreground" />{selectedSortLabel}</span
						>
					</Select.Trigger>
					<Select.Content align="end" class="rounded-xl">
						{#each sortOptions as option (option.value)}
							<Select.Item value={option.value} class="rounded-lg">{option.label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
		</div>

		<div class="pt-6">
			{#if visibleItems.length}
				<div
					class="grid grid-cols-2 gap-x-4 gap-y-8 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6"
				>
					{#each visibleItems as game (game.slug)}
						<article
							class="min-w-0"
							onpointerenter={(event) => {
								if (event.pointerType === 'mouse') flippedSlug = game.slug;
							}}
							onpointerleave={(event) => {
								if (event.pointerType === 'mouse' && flippedSlug === game.slug) flippedSlug = null;
							}}
						>
							<div class="flip-perspective relative aspect-2/3">
								<div
									class="flip-turn relative size-full"
									class:is-flipped={flippedSlug === game.slug}
								>
									<button
										type="button"
										class="flip-face absolute inset-0 cursor-pointer overflow-hidden rounded-xl bg-muted ring-1 ring-foreground/10 outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
										inert={flippedSlug === game.slug}
										aria-label="{game.title} options"
										onclick={() => (flippedSlug = game.slug)}
									>
										{#if game.cover}
											<img
												src={game.cover}
												alt="{game.title} cover"
												width="600"
												height="900"
												loading="lazy"
												class="size-full object-cover"
											/>
										{/if}
									</button>
									<div
										class="flip-face flip-back absolute inset-0 flex flex-col items-center justify-center gap-3 rounded-xl bg-card p-4 text-center text-card-foreground ring-1 ring-foreground/10"
										inert={flippedSlug !== game.slug}
									>
										{@render listIcon('size-5 text-muted-foreground')}
										<p class="line-clamp-2 text-sm font-medium">{game.title}</p>
										<Button
											variant="default"
											size="sm"
											href="/games/{game.slug}"
											class="w-full cursor-pointer rounded-full text-xs">View game</Button
										>
										<Button
											variant="outline"
											size="sm"
											class="w-full cursor-pointer rounded-full text-xs"
											onclick={() => removeItem(game)}>{text.removeLabel}</Button
										>
									</div>
								</div>
							</div>
							<h3 class="truncate px-0.5 pt-3 text-sm font-medium text-primary md:text-[15px]">
								{game.title}
							</h3>
							<p class="mt-0.5 flex items-center gap-1.5 px-0.5 text-xs text-muted-foreground">
								<span>{game.year}</span>
							</p>
						</article>
					{/each}
				</div>
			{:else}
				<Empty.Root class="min-h-80 rounded-xl border border-dashed">
					<Empty.Header>
						<Empty.Media variant="icon">
							{#if list.items.length}<SearchIcon />{:else if list.kind === 'played'}<CircleCheckIcon
								/>{:else if list.kind === 'play_later'}<ClockIcon />{:else}<GamepadIcon />{/if}
						</Empty.Media>
						<Empty.Title>{list.items.length ? 'No matching games' : text.emptyTitle}</Empty.Title>
						<Empty.Description
							>{list.items.length
								? 'Try another title or clear the search.'
								: text.emptyDescription}</Empty.Description
						>
					</Empty.Header>
					<Empty.Content>
						{#if list.items.length}
							<Button
								variant="outline"
								class="cursor-pointer rounded-full"
								onclick={() => (query = '')}>Clear search</Button
							>
						{:else}
							<Button href="/games" class="cursor-pointer rounded-full">
								Discover games
								<ArrowUpRightIcon data-icon="inline-end" class="size-4" />
							</Button>
						{/if}
					</Empty.Content>
				</Empty.Root>
			{/if}
		</div>
	</section>
</div>

<Dialog.Root bind:open={editOpen}>
	<Dialog.Content>
		<Dialog.Header class="text-left">
			<Dialog.Title>
				{editMode === 'details' ? 'Edit list' : 'Change cover'}
			</Dialog.Title>
			<Dialog.Description>
				{editMode === 'details'
					? 'Make this list your own.'
					: 'The cover is one of the games in the list.'}
			</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-5" onsubmit={saveSettings}>
			{#if editMode === 'cover'}
				<div class="max-h-80 overflow-y-auto">
					<ul class="grid grid-cols-3 gap-2 sm:grid-cols-4" aria-label="Cover game">
						{#each [{ slug: null, title: 'Default', cover: list.items[0]?.cover ?? null }, ...list.items] as game (game.slug ?? 'default')}
							<li>
								<button
									type="button"
									onclick={() => (draftCover = game.slug)}
									aria-pressed={draftCover === game.slug}
									class="group block w-full cursor-pointer rounded-xl text-left outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
								>
									<span
										class="relative block aspect-[3/4] overflow-hidden rounded-xl bg-muted ring-2 {draftCover ===
										game.slug
											? 'ring-primary'
											: 'ring-transparent'}"
									>
										{#if game.cover}
											<img src={game.cover} alt="" loading="lazy" class="size-full object-cover" />
										{/if}
									</span>
									<span class="mt-1 block truncate text-xs text-muted-foreground">{game.title}</span
									>
								</button>
							</li>
						{/each}
					</ul>
				</div>
			{:else}
				<div class="space-y-2">
					<Label for="list-name">List name</Label>
					<Input
						id="list-name"
						bind:value={draftName}
						disabled={list.kind !== 'custom'}
						maxlength={80}
						required
						aria-invalid={nameError ? true : undefined}
					/>
					{#if nameError}<p class="text-sm text-destructive" role="alert">{nameError}</p>{/if}
				</div>
				<div class="space-y-2">
					<Label for="list-description"
						>Description <span class="font-normal text-muted-foreground">(optional)</span></Label
					>
					<Textarea
						id="list-description"
						bind:value={draftDescription}
						maxlength={280}
						rows={3}
						placeholder="What makes this list special?"
						class="max-h-40 resize-y"
					/>
				</div>
				<div class="space-y-2">
					<Label id="list-visibility-label">Visibility</Label>
					<Select.Root type="single" bind:value={draftVisibility}>
						<Select.Trigger class="h-10 w-full" aria-labelledby="list-visibility-label">
							<span data-slot="select-value">
								{#if draftVisibility === 'private'}<LockIcon />Private{:else}<GlobeIcon
									/>Public{/if}
							</span>
						</Select.Trigger>
						<Select.Content class="rounded-xl">
							<Select.Item value="private" class="rounded-lg"
								><LockIcon class="size-4" />Private</Select.Item
							>
							<Select.Item value="public" class="rounded-lg"
								><GlobeIcon class="size-4" />Public</Select.Item
							>
						</Select.Content>
					</Select.Root>
					<p class="text-xs leading-5 text-muted-foreground">
						{draftVisibility === 'private'
							? 'Only you can see this list.'
							: 'Anyone can see this list on your profile.'}
					</p>
				</div>
			{/if}
			<Dialog.Footer>
				<Button
					type="button"
					variant="outline"
					class="cursor-pointer"
					onclick={() => (editOpen = false)}>Cancel</Button
				>
				<Button type="submit" class="cursor-pointer">Save changes</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<style>
	.list-hero {
		--offset: var(--sidebar-offset, 0px);
		margin-top: -4rem;
		margin-left: calc(-1 * var(--offset));
		padding-left: var(--offset);
	}

	.list-scrim {
		background:
			linear-gradient(
				0deg,
				var(--background) 0%,
				color-mix(in oklch, var(--background) 70%, transparent) 35%,
				transparent 75%
			),
			linear-gradient(
				90deg,
				color-mix(in oklch, var(--background) 70%, transparent) 0%,
				color-mix(in oklch, var(--background) 30%, transparent) 60%,
				color-mix(in oklch, var(--background) 20%, transparent) 100%
			),
			linear-gradient(
				180deg,
				color-mix(in oklch, var(--background) 55%, transparent) 0%,
				transparent 22%
			);
	}

	.flip-perspective {
		perspective: 1000px;
	}

	.flip-turn {
		transform-style: preserve-3d;
		transition: transform 450ms cubic-bezier(0.2, 0.7, 0.2, 1);
	}

	.flip-turn.is-flipped {
		transform: rotateY(180deg);
	}

	.flip-face {
		backface-visibility: hidden;
	}

	.flip-back {
		transform: rotateY(180deg);
	}

	@media (prefers-reduced-motion: reduce) {
		.flip-turn {
			transition: none;
		}
	}
</style>
