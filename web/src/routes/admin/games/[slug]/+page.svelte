<script lang="ts">
	import { enhance } from '$app/forms';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/state';
	import { replaceState } from '$app/navigation';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';
	import AdminPageHeader from '#lib/features/admin/admin-page-header.svelte';
	import GameForm from '#lib/features/admin/game-form.svelte';
	import StatusBadge from '#lib/features/admin/status-badge.svelte';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
	let deleting = $state(false);
	let confirmOpen = $state(false);

	// A save redirects here with ?saved; say so once and tidy the address.
	$effect(() => {
		if (page.url.searchParams.has('saved')) {
			toast.success('Game saved');
			const url = new URL(page.url.href);
			url.searchParams.delete('saved');
			replaceState(url.pathname + url.search, page.state);
		}
	});
</script>

<Button href="/admin/games" variant="ghost" size="sm" class="-ms-3 mb-2">
	<ArrowLeftIcon />Games
</Button>

<AdminPageHeader title={data.title} description={`/${data.slug}`}>
	{#snippet actions()}
		<StatusBadge status={data.status} />
		{#if data.status === 'published'}
			<Button
				href={`/games/${data.slug}`}
				variant="outline"
				size="lg"
				target="_blank"
				rel="noopener"><ExternalLinkIcon />View</Button
			>
		{/if}
		{#if data.canDelete}
			<Button variant="destructive" size="lg" onclick={() => (confirmOpen = true)}>
				<Trash2Icon />Delete
			</Button>
		{/if}
	{/snippet}
</AdminPageHeader>

{#key data.id + data.slug}
	<GameForm
		initial={data.initial}
		platforms={data.platforms}
		action="?/save"
		submitLabel="Save changes"
		error={form?.error ?? null}
	/>
{/key}

<AlertDialog.Root
	open={confirmOpen}
	onOpenChange={(open) => {
		if (!deleting) confirmOpen = open;
	}}
>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Delete “{data.title}”?</AlertDialog.Title>
			<AlertDialog.Description>
				This removes the game for good, along with everyone’s library entries, ratings and reviews
				for it. This can’t be undone. To only take it off the site, set it to Hidden instead.
			</AlertDialog.Description>
		</AlertDialog.Header>
		<form
			method="POST"
			action="?/delete"
			use:enhance={() => {
				deleting = true;
				return async ({ result, update }) => {
					deleting = false;
					if (result.type === 'failure') {
						toast.error(String(result.data?.error ?? 'Could not delete the game.'));
						confirmOpen = false;
					}
					await update({ reset: false });
				};
			}}
		>
			<AlertDialog.Footer>
				<AlertDialog.Cancel disabled={deleting} type="button">Cancel</AlertDialog.Cancel>
				<!-- A plain button: AlertDialog.Action would close the dialog before the form posts. -->
				<Button type="submit" variant="destructive" size="lg" disabled={deleting}>
					{deleting ? 'Deleting…' : 'Delete game'}
				</Button>
			</AlertDialog.Footer>
		</form>
	</AlertDialog.Content>
</AlertDialog.Root>
