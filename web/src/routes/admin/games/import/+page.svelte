<script lang="ts">
	import { enhance } from '$app/forms';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Card from '#lib/components/ui/card/index.js';
	import * as Field from '#lib/components/ui/field/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Progress } from '#lib/components/ui/progress/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import { Switch } from '#lib/components/ui/switch/index.js';
	import * as Table from '#lib/components/ui/table/index.js';
	import * as Tabs from '#lib/components/ui/tabs/index.js';
	import AdminPageHeader from '#lib/features/admin/admin-page-header.svelte';
	import GameForm from '#lib/features/admin/game-form.svelte';
	import type { GameFormValues } from '#lib/features/admin/game-form.js';
	import { adminErrorMessage } from '#lib/features/admin/game-form.js';
	import { skipReasons, type ImportHit, type ImportJob } from '#lib/features/admin/import.js';
	import DownloadIcon from '@lucide/svelte/icons/download';
	import SearchIcon from '@lucide/svelte/icons/search';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	type Preview = {
		title: string;
		review: string | null;
		wikidata: boolean;
		steam: boolean;
		existingSlug: string | null;
		values: GameFormValues;
		imported: string;
	};

	// ---------- One game ----------

	let query = $state('');
	let hits = $state<ImportHit[] | null>(null);
	let searching = $state(false);
	let searchError = $state<string | null>(null);
	let loadingId = $state<string | null>(null);
	let preview = $state<Preview | null>(null);

	// ---------- Bulk ----------

	let limit = $state('50');
	let platform = $state('all');
	let yearFrom = $state('');
	let yearTo = $state('');
	let publish = $state(false);
	let refresh = $state(false);
	let job = $state<ImportJob | null>(null);
	let starting = $state(false);
	let jobError = $state<string | null>(null);

	const running = $derived(job?.state === 'running');
	const percent = $derived(job && job.limit ? Math.min(100, (job.processed / job.limit) * 100) : 0);
	const platformName = $derived(
		platform === 'all'
			? 'Any platform'
			: (data.platforms.find((item) => item.slug === platform)?.name ?? platform)
	);

	async function loadJob() {
		try {
			const response = await fetch('/admin/games/import/job');
			if (response.ok) job = (await response.json()).job;
		} catch {
			// The next poll tries again.
		}
	}

	onMount(() => {
		loadJob();
		const timer = setInterval(() => {
			if (running) loadJob();
		}, 3000);
		return () => clearInterval(timer);
	});

	async function startJob() {
		starting = true;
		jobError = null;
		try {
			const response = await fetch('/admin/games/import/job', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({
					limit: Number(limit),
					platform: platform === 'all' ? null : platform,
					yearFrom,
					yearTo,
					status: publish ? 'published' : 'draft',
					updateExisting: refresh
				})
			});
			const body = await response.json();
			if (response.ok) job = body.job;
			else jobError = body.error;
		} catch {
			jobError = adminErrorMessage('unavailable');
		}
		starting = false;
	}

	async function stopJob() {
		const response = await fetch('/admin/games/import/job', { method: 'DELETE' });
		if (response.ok) {
			toast.success('Stopping after the current game');
			await loadJob();
		}
	}

	const outcomeLabels = {
		created: 'Added',
		updated: 'Updated',
		skipped: 'Skipped',
		failed: 'Failed'
	} as const;

	const noteOf = (item: ImportJob['items'][number]) =>
		item.review ??
		(item.reason ? (skipReasons[item.reason] ?? adminErrorMessage(item.reason)) : '');
</script>

<AdminPageHeader
	title="Import games"
	description="Wikidata lists the games and Steam fills in the details. Fields you edit by hand are never overwritten by a later import."
/>

<Tabs.Root value="one">
	<Tabs.List>
		<Tabs.Trigger value="one">One game</Tabs.Trigger>
		<Tabs.Trigger value="bulk">Many games</Tabs.Trigger>
	</Tabs.List>

	<Tabs.Content value="one" class="mt-4 flex flex-col gap-6">
		<Card.Root>
			<Card.Header>
				<Card.Title>Find a game</Card.Title>
				<Card.Description>Type a name, or paste a Steam link or app id.</Card.Description>
			</Card.Header>
			<Card.Content>
				<form
					method="POST"
					action="?/search"
					class="flex flex-col gap-3 sm:flex-row"
					use:enhance={() => {
						searching = true;
						searchError = null;
						return async ({ result }) => {
							searching = false;
							if (result.type === 'success') hits = (result.data?.hits as ImportHit[]) ?? [];
							else if (result.type === 'failure')
								searchError = String(result.data?.error ?? adminErrorMessage('unavailable'));
						};
					}}
				>
					<div class="relative flex-1">
						<SearchIcon
							class="pointer-events-none absolute start-4 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
							aria-hidden="true"
						/>
						<Input
							name="q"
							bind:value={query}
							placeholder="Elden Ring, or store.steampowered.com/app/1245620"
							aria-label="Game name or Steam link"
							class="h-11 rounded-xl ps-10"
							maxlength={200}
							autocomplete="off"
							required
						/>
					</div>
					<Button type="submit" size="lg" disabled={searching}>
						{#if searching}<Spinner />Searching…{:else}Search{/if}
					</Button>
				</form>

				{#if searchError}
					<p class="mt-3 text-callout text-destructive" role="alert">{searchError}</p>
				{/if}

				{#if hits}
					{#if hits.length === 0}
						<p class="mt-4 text-callout text-muted-foreground">Nothing found. Try another name.</p>
					{:else}
						<ul class="mt-4 flex flex-col gap-2">
							{#each hits as hit (hit.source + hit.id)}
								<li>
									<form
										method="POST"
										action="?/preview"
										use:enhance={() => {
											loadingId = hit.source + hit.id;
											searchError = null;
											return async ({ result }) => {
												loadingId = null;
												if (result.type === 'success') {
													preview = (result.data?.preview as Preview) ?? null;
												} else if (result.type === 'failure') {
													searchError = String(
														result.data?.error ?? adminErrorMessage('unavailable')
													);
												}
											};
										}}
									>
										<input type="hidden" name="source" value={hit.source} />
										<input type="hidden" name="id" value={hit.id} />
										<button
											type="submit"
											disabled={loadingId !== null}
											class="flex min-h-11 w-full items-center gap-3 rounded-xl px-3 py-2 text-start ring-1 ring-border transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
										>
											<span class="h-11 w-24 shrink-0 overflow-hidden rounded-lg bg-muted">
												{#if hit.image}
													<img src={hit.image} alt="" class="size-full object-cover" />
												{/if}
											</span>
											<span class="min-w-0 flex-1">
												<span class="block truncate font-medium">{hit.name}</span>
												<span class="block truncate text-footnote text-muted-foreground">
													{hit.description ??
														(hit.source === 'steam' ? `Steam app ${hit.id}` : hit.id)}
												</span>
											</span>
											{#if hit.existing_slug}
												<Badge variant="secondary">In catalog</Badge>
											{:else if loadingId === hit.source + hit.id}
												<Spinner />
											{:else}
												<Badge variant="outline"
													>{hit.source === 'steam' ? 'Steam' : 'Wikidata'}</Badge
												>
											{/if}
										</button>
									</form>
								</li>
							{/each}
						</ul>
					{/if}
				{/if}
			</Card.Content>
		</Card.Root>

		{#if preview}
			{#if preview.existingSlug}
				<Card.Root>
					<Card.Header>
						<Card.Title>{preview.title} is already in the catalog</Card.Title>
						<Card.Description>
							Edit it instead, or use “Many games” with “Refresh games already in the catalog”.
						</Card.Description>
					</Card.Header>
					<Card.Footer>
						<Button href={`/admin/games/${preview.existingSlug}`} size="lg">Open game</Button>
					</Card.Footer>
				</Card.Root>
			{:else}
				{#if preview.review}
					<p
						class="flex items-start gap-3 rounded-2xl bg-amber-500/10 px-4 py-3 text-callout text-amber-700 dark:text-amber-300"
						role="status"
					>
						<TriangleAlertIcon class="mt-0.5 size-4 shrink-0" aria-hidden="true" />
						<span>Check this one before saving: {preview.review.toLowerCase()}.</span>
					</p>
				{/if}
				{#if !preview.steam}
					<p class="text-callout text-muted-foreground">
						No Steam page was found, so there are no images or description. Add them by hand.
					</p>
				{/if}
				{#key preview.imported}
					<GameForm
						initial={preview.values}
						platforms={data.platforms}
						submitLabel="Create game"
						action="?/save"
					>
						{#snippet extra()}
							<input type="hidden" name="imported" value={preview?.imported ?? ''} />
						{/snippet}
					</GameForm>
				{/key}
			{/if}
		{/if}
	</Tabs.Content>

	<Tabs.Content value="bulk" class="mt-4 flex flex-col gap-6">
		<Card.Root>
			<Card.Header>
				<Card.Title>Import the most popular games</Card.Title>
				<Card.Description>
					Games come from Wikidata, most covered first. About two seconds per game; it keeps running
					if you leave this page.
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<form
					class="grid gap-6"
					onsubmit={(event) => {
						event.preventDefault();
						startJob();
					}}
				>
					<div class="grid gap-6 md:grid-cols-4">
						<Field.Field>
							<Field.Label for="bulk-limit">How many</Field.Label>
							<Input
								id="bulk-limit"
								type="number"
								min="1"
								max="500"
								bind:value={limit}
								class="rounded-xl"
								required
							/>
						</Field.Field>
						<Field.Field>
							<Field.Label for="bulk-platform">Platform</Field.Label>
							<Select.Root type="single" bind:value={platform}>
								<Select.Trigger
									id="bulk-platform"
									class="w-full rounded-xl data-[size=default]:h-11"
								>
									{platformName}
								</Select.Trigger>
								<Select.Content>
									<Select.Item value="all" label="Any platform" />
									{#each data.platforms as item (item.slug)}
										<Select.Item value={item.slug} label={item.name} />
									{/each}
								</Select.Content>
							</Select.Root>
						</Field.Field>
						<Field.Field>
							<Field.Label for="bulk-from">Released from</Field.Label>
							<Input
								id="bulk-from"
								type="number"
								min="1950"
								max="2100"
								placeholder="Year"
								bind:value={yearFrom}
								class="rounded-xl"
							/>
						</Field.Field>
						<Field.Field>
							<Field.Label for="bulk-to">Released to</Field.Label>
							<Input
								id="bulk-to"
								type="number"
								min="1950"
								max="2100"
								placeholder="Year"
								bind:value={yearTo}
								class="rounded-xl"
							/>
						</Field.Field>
					</div>

					<div class="grid gap-3 md:grid-cols-2">
						<label
							class="flex min-h-11 items-center justify-between gap-4 rounded-xl px-3 ring-1 ring-border"
						>
							<span class="text-callout"
								>Publish right away <span class="text-muted-foreground">(otherwise drafts)</span
								></span
							>
							<Switch bind:checked={publish} />
						</label>
						<label
							class="flex min-h-11 items-center justify-between gap-4 rounded-xl px-3 ring-1 ring-border"
						>
							<span class="text-callout">Refresh games already in the catalog</span>
							<Switch bind:checked={refresh} />
						</label>
					</div>

					{#if jobError}
						<p class="text-callout text-destructive" role="alert">{jobError}</p>
					{/if}
					<div>
						<Button type="submit" size="lg" disabled={starting || running}>
							{#if starting || running}<Spinner />Importing…{:else}<DownloadIcon />Start import{/if}
						</Button>
					</div>
				</form>
			</Card.Content>
		</Card.Root>

		{#if job}
			<Card.Root>
				<Card.Header>
					<Card.Title>
						{#if job.state === 'running'}Importing…
						{:else if job.state === 'done'}Finished
						{:else if job.state === 'cancelled'}Stopped
						{:else}Failed{/if}
					</Card.Title>
					<Card.Description>
						{job.processed} of {job.limit} · {job.created} added · {job.updated} updated ·
						{job.skipped} skipped · {job.failed} failed{job.review
							? ` · ${job.review} to review`
							: ''}
					</Card.Description>
					{#if running}
						<Card.Action>
							<Button variant="secondary" size="lg" onclick={stopJob}>Stop</Button>
						</Card.Action>
					{/if}
				</Card.Header>
				<Card.Content class="flex flex-col gap-4">
					{#if running}<Progress value={percent} aria-label="Import progress" />{/if}
					{#if job.error}
						<p class="text-callout text-destructive" role="alert">{adminErrorMessage(job.error)}</p>
					{/if}
					{#if job.review}
						<p class="text-callout text-muted-foreground">
							Games with a note need a look. Find them under Games → “Needs review”.
						</p>
					{/if}
					{#if job.items.length}
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>Game</Table.Head>
									<Table.Head>Result</Table.Head>
									<Table.Head>Note</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each [...job.items].reverse() as item, index (index)}
									<Table.Row>
										<Table.Cell class="max-w-72 truncate font-medium">
											{#if item.slug}
												<a href={`/admin/games/${item.slug}`} class="hover:underline"
													>{item.title}</a
												>
											{:else}
												{item.title}
											{/if}
										</Table.Cell>
										<Table.Cell>
											<Badge variant={item.outcome === 'failed' ? 'destructive' : 'secondary'}>
												{outcomeLabels[item.outcome]}
											</Badge>
										</Table.Cell>
										<Table.Cell class="text-muted-foreground">{noteOf(item)}</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					{/if}
				</Card.Content>
			</Card.Root>
		{/if}
	</Tabs.Content>
</Tabs.Root>
