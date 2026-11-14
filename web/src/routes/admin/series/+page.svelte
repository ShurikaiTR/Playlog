<script lang="ts">
	import { enhance } from '$app/forms';
	import { toast } from 'svelte-sonner';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Item from '#lib/components/ui/item/index.js';
	import * as Empty from '#lib/components/ui/empty/index.js';
	import AdminPageHeader from '#lib/features/admin/admin-page-header.svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	let creating = $state(false);
	let saving = $state(false);
</script>

<AdminPageHeader
	title="Series"
	description="Games that belong together, with the order we recommend playing them in."
>
	{#snippet actions()}
		<Button size="lg" onclick={() => (creating = true)}>
			<PlusIcon aria-hidden="true" /> New series
		</Button>
	{/snippet}
</AdminPageHeader>

{#if data.failed}
	<p class="text-callout text-destructive" role="alert">
		The series could not be loaded. Try again.
	</p>
{/if}

{#if data.series.length}
	<Item.Group class="gap-2">
		{#each data.series as series (series.id)}
			<Item.Root variant="outline" class="min-h-11 rounded-2xl">
				{#snippet child({ props })}
					<a href="/admin/series/{series.id}" {...props}>
						<Item.Content>
							<Item.Title class="text-callout">{series.name}</Item.Title>
							<Item.Description class="text-footnote">
								{series.games}
								{series.games === 1 ? 'game' : 'games'}
							</Item.Description>
						</Item.Content>
						<Item.Actions>
							<Badge variant={series.ordered ? 'default' : 'secondary'}>
								{series.ordered ? 'Custom order' : 'By release date'}
							</Badge>
							<ChevronRightIcon class="size-4 text-muted-foreground" aria-hidden="true" />
						</Item.Actions>
					</a>
				{/snippet}
			</Item.Root>
		{/each}
	</Item.Group>
{:else if !data.failed}
	<Empty.Root class="rounded-2xl border border-dashed">
		<Empty.Header>
			<Empty.Title>No series yet</Empty.Title>
			<Empty.Description>
				Create one, add its games and put them in the order players should follow.
			</Empty.Description>
		</Empty.Header>
	</Empty.Root>
{/if}

<Dialog.Root
	open={creating}
	onOpenChange={(open) => {
		if (!open && !saving) creating = false;
	}}
>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>New series</Dialog.Title>
			<Dialog.Description>You add the games on the next page.</Dialog.Description>
		</Dialog.Header>
		<form
			method="POST"
			action="?/create"
			class="grid gap-6"
			use:enhance={() => {
				saving = true;
				return async ({ result, update }) => {
					saving = false;
					if (result.type === 'failure')
						toast.error(String(result.data?.error ?? 'Could not create.'));
					await update({ reset: false });
				};
			}}
		>
			<Input
				name="name"
				required
				maxlength={200}
				placeholder="Call of Duty"
				aria-label="Series name"
				class="h-11 rounded-xl"
			/>
			<Dialog.Footer>
				<Button
					type="button"
					variant="secondary"
					size="lg"
					disabled={saving}
					onclick={() => (creating = false)}>Cancel</Button
				>
				<Button type="submit" size="lg" disabled={saving}>Create</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
