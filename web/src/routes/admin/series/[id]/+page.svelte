<script lang="ts">
	import { enhance } from '$app/forms';
	import { toast } from 'svelte-sonner';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Switch } from '#lib/components/ui/switch/index.js';
	import * as Card from '#lib/components/ui/card/index.js';
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';
	import AdminPageHeader from '#lib/features/admin/admin-page-header.svelte';
	import { formatRelease, type DatePrecision } from '#lib/features/games/games.js';
	import type { AdminSeriesGame } from '#lib/server/admin.js';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ArrowUpIcon from '@lucide/svelte/icons/arrow-up';
	import ArrowDownIcon from '@lucide/svelte/icons/arrow-down';
	import GripVerticalIcon from '@lucide/svelte/icons/grip-vertical';
	import XIcon from '@lucide/svelte/icons/x';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// svelte-ignore state_referenced_locally
	let name = $state(data.series.name);
	// svelte-ignore state_referenced_locally
	let ordered = $state(data.series.ordered);
	// svelte-ignore state_referenced_locally
	let games = $state<AdminSeriesGame[]>(data.series.games.map((game) => ({ ...game })));
	let saving = $state(false);
	let confirmDelete = $state(false);

	const payload = $derived(JSON.stringify(games.map(({ id, note }) => ({ id, note: note ?? '' }))));

	function move(from: number, to: number) {
		if (to < 0 || to >= games.length || from === to) return;
		const next = [...games];
		const [item] = next.splice(from, 1);
		next.splice(to, 0, item);
		games = next;
		ordered = true;
	}

	// Drag and drop for the mouse; the arrow buttons cover touch and the keyboard.
	let dragging = $state<number | null>(null);
	let over = $state<number | null>(null);

	// Adding games.
	let query = $state('');
	let results = $state<AdminSeriesGame[]>([]);
	let searching = $state(false);
	let typing: ReturnType<typeof setTimeout> | undefined;
	let latest = 0;

	function onSearch(value: string) {
		query = value;
		clearTimeout(typing);
		if (value.trim().length < 2) {
			results = [];
			return;
		}
		typing = setTimeout(async () => {
			const id = ++latest;
			searching = true;
			try {
				const response = await fetch(`/admin/series/search?q=${encodeURIComponent(value.trim())}`);
				const body = (await response.json()) as { games: AdminSeriesGame[] };
				if (id === latest) results = body.games;
			} catch {
				if (id === latest) results = [];
			} finally {
				if (id === latest) searching = false;
			}
		}, 250);
	}

	const inSeries = (id: string) => games.some((game) => game.id === id);

	function add(game: AdminSeriesGame) {
		if (inSeries(game.id)) return;
		games = [...games, { ...game, note: null }];
	}

	const release = (game: AdminSeriesGame) =>
		formatRelease(game.release_date, game.release_precision as DatePrecision);
</script>

<AdminPageHeader
	title={data.series.name}
	description="Add games, then drag them into the order to play."
>
	{#snippet actions()}
		<Button variant="secondary" size="lg" href="/admin/series">
			<ArrowLeftIcon aria-hidden="true" /> All series
		</Button>
	{/snippet}
</AdminPageHeader>

<form
	method="POST"
	action="?/save"
	class="grid gap-6"
	use:enhance={() => {
		saving = true;
		return async ({ result, update }) => {
			saving = false;
			if (result.type === 'success') toast.success('Series saved');
			else if (result.type === 'failure')
				toast.error(String(result.data?.error ?? 'Could not save.'));
			await update({ reset: false });
		};
	}}
>
	<input type="hidden" name="games" value={payload} />

	<Card.Root class="rounded-2xl">
		<Card.Content class="grid gap-5">
			<div class="grid gap-2">
				<Label for="series-name" class="text-callout">Name</Label>
				<Input
					id="series-name"
					name="name"
					bind:value={name}
					required
					maxlength={200}
					class="h-11 rounded-xl"
				/>
			</div>
			<div class="flex min-h-11 items-center justify-between gap-4">
				<div class="min-w-0">
					<Label for="series-ordered" class="text-callout">Use my order</Label>
					<p class="text-footnote text-muted-foreground">
						Off: the series shows in release-date order. Moving a game turns this on.
					</p>
				</div>
				<Switch id="series-ordered" name="ordered" bind:checked={ordered} />
			</div>
		</Card.Content>
	</Card.Root>

	<section class="grid gap-3" aria-label="Games in this series">
		<h3 class="text-headline">Games in this series</h3>
		{#if games.length}
			<ol class="grid gap-2">
				{#each games as game, index (game.id)}
					<li
						class="flex flex-col gap-3 rounded-2xl border bg-card p-3 transition-colors sm:flex-row sm:items-center {over ===
							index && dragging !== index
							? 'border-primary bg-muted'
							: ''} {dragging === index ? 'opacity-50' : ''}"
						ondragover={(event) => {
							if (dragging === null) return;
							event.preventDefault();
							over = index;
						}}
						ondrop={(event) => {
							event.preventDefault();
							if (dragging !== null) move(dragging, index);
							dragging = over = null;
						}}
					>
						<div class="flex min-w-0 flex-1 items-center gap-3">
							<span
								class="hidden size-11 shrink-0 cursor-grab items-center justify-center rounded-xl text-muted-foreground hover:bg-muted active:cursor-grabbing sm:inline-flex"
								role="presentation"
								draggable="true"
								ondragstart={(event) => {
									dragging = index;
									event.dataTransfer?.setData('text/plain', game.id);
									if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move';
								}}
								ondragend={() => (dragging = over = null)}
							>
								<GripVerticalIcon class="size-4" aria-hidden="true" />
							</span>
							<span class="w-6 shrink-0 text-center text-callout text-muted-foreground tabular-nums"
								>{index + 1}</span
							>
							{#if game.cover_url}
								<img
									src={game.cover_url}
									alt=""
									width="40"
									height="60"
									class="h-15 w-10 shrink-0 rounded-lg object-cover"
								/>
							{:else}
								<div class="h-15 w-10 shrink-0 rounded-lg bg-muted"></div>
							{/if}
							<div class="min-w-0">
								<p class="truncate text-callout font-medium">{game.title}</p>
								<p class="truncate text-footnote text-muted-foreground">
									{release(game)}
									{#if game.status !== 'published'}
										<Badge variant="secondary" class="ms-1">{game.status}</Badge>
									{/if}
								</p>
							</div>
						</div>
						<Input
							value={game.note ?? ''}
							oninput={(event) => (game.note = event.currentTarget.value)}
							placeholder="Note (optional)"
							aria-label="Note for {game.title}"
							maxlength={200}
							class="h-11 rounded-xl sm:w-64"
						/>
						<div class="flex shrink-0 items-center gap-1">
							<Button
								type="button"
								variant="ghost"
								size="icon-lg"
								class="size-11"
								aria-label="Move {game.title} up"
								disabled={index === 0}
								onclick={() => move(index, index - 1)}
							>
								<ArrowUpIcon aria-hidden="true" />
							</Button>
							<Button
								type="button"
								variant="ghost"
								size="icon-lg"
								class="size-11"
								aria-label="Move {game.title} down"
								disabled={index === games.length - 1}
								onclick={() => move(index, index + 1)}
							>
								<ArrowDownIcon aria-hidden="true" />
							</Button>
							<Button
								type="button"
								variant="ghost"
								size="icon-lg"
								class="size-11"
								aria-label="Remove {game.title} from the series"
								onclick={() => (games = games.filter((other) => other.id !== game.id))}
							>
								<XIcon aria-hidden="true" />
							</Button>
						</div>
					</li>
				{/each}
			</ol>
		{:else}
			<p
				class="rounded-2xl border border-dashed p-6 text-center text-callout text-muted-foreground"
			>
				No games yet. Search below to add the first one.
			</p>
		{/if}
	</section>

	<section class="grid gap-3" aria-label="Add a game">
		<h3 class="text-headline">Add a game</h3>
		<div class="relative">
			<SearchIcon
				class="pointer-events-none absolute start-4 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
				aria-hidden="true"
			/>
			<Input
				type="search"
				value={query}
				oninput={(event) => onSearch(event.currentTarget.value)}
				placeholder="Search games by title"
				aria-label="Search games to add"
				maxlength={100}
				class="h-11 rounded-xl ps-10"
			/>
		</div>
		{#if results.length}
			<ul class="grid gap-2">
				{#each results as game (game.id)}
					<li class="flex min-h-11 items-center justify-between gap-3 rounded-2xl border p-3">
						<div class="min-w-0">
							<p class="truncate text-callout font-medium">{game.title}</p>
							<p class="truncate text-footnote text-muted-foreground">{release(game)}</p>
						</div>
						<Button
							type="button"
							variant="secondary"
							size="lg"
							disabled={inSeries(game.id)}
							onclick={() => add(game)}
						>
							<PlusIcon aria-hidden="true" />
							{inSeries(game.id) ? 'Added' : 'Add'}
						</Button>
					</li>
				{/each}
			</ul>
			<p class="text-footnote text-muted-foreground">
				A game already in another series moves here when you save.
			</p>
		{:else if query.trim().length >= 2 && !searching}
			<p class="text-callout text-muted-foreground">No games match.</p>
		{/if}
	</section>

	<div class="flex flex-wrap items-center justify-between gap-3">
		{#if data.canDelete}
			<Button
				type="button"
				variant="ghost"
				size="lg"
				class="text-destructive"
				onclick={() => (confirmDelete = true)}>Delete series</Button
			>
		{:else}
			<span></span>
		{/if}
		<Button type="submit" size="lg" disabled={saving || !name.trim()}>
			{saving ? 'Saving…' : 'Save'}
		</Button>
	</div>
</form>

<AlertDialog.Root bind:open={confirmDelete}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Delete this series?</AlertDialog.Title>
			<AlertDialog.Description>
				The games stay in the catalog; they just leave the series and lose their order.
			</AlertDialog.Description>
		</AlertDialog.Header>
		<form method="POST" action="?/delete" use:enhance>
			<AlertDialog.Footer>
				<AlertDialog.Cancel type="button">Cancel</AlertDialog.Cancel>
				<AlertDialog.Action type="submit">Delete</AlertDialog.Action>
			</AlertDialog.Footer>
		</form>
	</AlertDialog.Content>
</AlertDialog.Root>
