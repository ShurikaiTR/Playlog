<script lang="ts">
	import { enhance } from '$app/forms';
	import { toast } from 'svelte-sonner';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Card from '#lib/components/ui/card/index.js';
	import { Checkbox } from '#lib/components/ui/checkbox/index.js';
	import * as Field from '#lib/components/ui/field/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import {
		DATE_PRECISIONS,
		GAME_KINDS,
		GAME_STATUSES,
		type GameFormValues
	} from '#lib/features/admin/game-form.js';
	import type { PlatformOption } from '#lib/server/admin.js';

	let {
		initial,
		platforms,
		submitLabel,
		action = undefined,
		error = null
	}: {
		initial: GameFormValues;
		platforms: PlatformOption[];
		submitLabel: string;
		/** The form action to post to; the page's default action when empty. */
		action?: string;
		error?: string | null;
	} = $props();

	// The form owns its values from here on, so a failed save keeps what was typed.
	// svelte-ignore state_referenced_locally
	let values = $state<GameFormValues>({ ...initial, platforms: [...initial.platforms] });
	let saving = $state(false);

	const label = (value: string) => value.charAt(0).toUpperCase() + value.slice(1);

	function togglePlatform(slug: string, checked: boolean) {
		values.platforms = checked
			? [...values.platforms, slug]
			: values.platforms.filter((platform) => platform !== slug);
	}
</script>

{#snippet text(
	name: keyof Omit<GameFormValues, 'platforms'>,
	title: string,
	options: {
		hint?: string;
		placeholder?: string;
		type?: 'text' | 'date' | 'number';
		required?: boolean;
	} = {}
)}
	<Field.Field>
		<Field.Label for={name}>{title}</Field.Label>
		<Input
			id={name}
			{name}
			type={options.type ?? 'text'}
			bind:value={values[name]}
			placeholder={options.placeholder}
			required={options.required}
			autocomplete="off"
			min={options.type === 'number' ? 1 : undefined}
		/>
		{#if options.hint}<Field.Description>{options.hint}</Field.Description>{/if}
	</Field.Field>
{/snippet}

{#snippet choice(
	name: 'kind' | 'status' | 'releasePrecision',
	title: string,
	options: readonly string[]
)}
	<Field.Field>
		<Field.Label for={name}>{title}</Field.Label>
		<Select.Root type="single" {name} bind:value={values[name]}>
			<Select.Trigger id={name} class="h-11 w-full rounded-full">
				{label(values[name])}
			</Select.Trigger>
			<Select.Content>
				{#each options as option (option)}
					<Select.Item value={option} label={label(option)} />
				{/each}
			</Select.Content>
		</Select.Root>
	</Field.Field>
{/snippet}

<form
	method="POST"
	{action}
	class="grid gap-6"
	use:enhance={() => {
		saving = true;
		return async ({ result, update }) => {
			saving = false;
			if (result.type === 'failure') {
				toast.error(String(result.data?.error ?? 'Could not save the game.'));
			} else if (result.type === 'redirect') {
				toast.success('Game saved');
			}
			await update({ reset: false });
		};
	}}
>
	{#if error}
		<p class="rounded-2xl bg-destructive/10 px-4 py-3 text-callout text-destructive" role="alert">
			{error}
		</p>
	{/if}

	<Card.Root>
		<Card.Header>
			<Card.Title>Basics</Card.Title>
		</Card.Header>
		<Card.Content>
			<Field.Group>
				{@render text('title', 'Title', { required: true })}
				{@render text('slug', 'Slug', {
					hint: 'The address of the game page. Leave it empty to build it from the title. Changing it keeps the old address working.',
					placeholder: 'the-witcher-3'
				})}
				<div class="grid gap-6 sm:grid-cols-2">
					{@render choice('kind', 'Type', GAME_KINDS)}
					{@render choice('status', 'Status', GAME_STATUSES)}
				</div>
				<Field.Description>
					Only published games show on the site. Drafts and hidden games stay in the admin.
				</Field.Description>
			</Field.Group>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Description</Card.Title>
		</Card.Header>
		<Card.Content>
			<Field.Group>
				<Field.Field>
					<Field.Label for="summary">Summary</Field.Label>
					<Textarea
						id="summary"
						name="summary"
						bind:value={values.summary}
						maxlength={500}
						rows={3}
						class="min-h-24"
					/>
					<Field.Description
						>One or two sentences for cards and search results. Up to 500 characters.</Field.Description
					>
				</Field.Field>
				<Field.Field>
					<Field.Label for="description">Description</Field.Label>
					<Textarea
						id="description"
						name="description"
						bind:value={values.description}
						rows={8}
						class="min-h-48"
					/>
				</Field.Field>
			</Field.Group>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Release</Card.Title>
		</Card.Header>
		<Card.Content>
			<Field.Group>
				<div class="grid gap-6 sm:grid-cols-3">
					{@render text('releaseDate', 'Release date', {
						type: 'date',
						hint: 'Empty means to be announced.'
					})}
					{@render choice('releasePrecision', 'Date precision', DATE_PRECISIONS)}
					{@render text('timeToBeat', 'Time to beat (minutes)', { type: 'number' })}
				</div>
				<Field.Set>
					<Field.Legend variant="label">Platforms</Field.Legend>
					<div class="grid grid-cols-1 gap-x-4 sm:grid-cols-2 lg:grid-cols-3">
						{#each platforms as platform (platform.slug)}
							<Field.Field orientation="horizontal" class="items-center">
								<Checkbox
									id={`platform-${platform.slug}`}
									checked={values.platforms.includes(platform.slug)}
									onCheckedChange={(checked) => togglePlatform(platform.slug, checked)}
								/>
								<Field.Label
									for={`platform-${platform.slug}`}
									class="min-h-9 flex-1 font-normal pointer-coarse:min-h-11"
									>{platform.name}</Field.Label
								>
							</Field.Field>
						{/each}
					</div>
					{#each values.platforms as slug (slug)}
						<input type="hidden" name="platforms" value={slug} />
					{/each}
				</Field.Set>
			</Field.Group>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Credits and tags</Card.Title>
			<Card.Description>Separate several names with commas.</Card.Description>
		</Card.Header>
		<Card.Content>
			<Field.Group>
				<div class="grid gap-6 sm:grid-cols-2">
					{@render text('developers', 'Developers')}
					{@render text('publishers', 'Publishers')}
					{@render text('genres', 'Genres')}
					{@render text('themes', 'Themes')}
					{@render text('modes', 'Game modes')}
					{@render text('perspectives', 'Perspectives')}
					{@render text('series', 'Series', { hint: 'One series name.' })}
					{@render text('parentSlug', 'Parent game slug', {
						hint: 'For DLC, expansions, remakes and ports.'
					})}
				</div>
			</Field.Group>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Images</Card.Title>
			<Card.Description>Addresses must start with https://.</Card.Description>
		</Card.Header>
		<Card.Content>
			<Field.Group>
				{@render text('coverUrl', 'Cover')}
				{@render text('backdropUrl', 'Backdrop')}
				{@render text('logoUrl', 'Logo')}
			</Field.Group>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>More</Card.Title>
		</Card.Header>
		<Card.Content>
			<Field.Group>
				{@render text('altTitles', 'Other titles', {
					hint: 'Other names the game goes by. They are searchable.'
				})}
				{@render text('languages', 'Languages', {
					hint: 'Language codes such as en, tr, pt-BR. New ones start with interface support.',
					placeholder: 'en, tr'
				})}
			</Field.Group>
		</Card.Content>
	</Card.Root>

	<div class="sticky bottom-4 z-10 flex justify-end gap-2">
		<Button href="/admin/games" variant="secondary" size="lg" disabled={saving}>Cancel</Button>
		<Button type="submit" size="lg" disabled={saving}>{saving ? 'Saving…' : submitLabel}</Button>
	</div>
</form>
