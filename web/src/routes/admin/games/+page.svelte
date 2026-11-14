<script lang="ts">
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import { SvelteURLSearchParams } from 'svelte/reactivity';
	import { toast } from 'svelte-sonner';
	import type { ColumnDef } from '@tanstack/table-core';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import { renderComponent } from '#lib/components/ui/data-table/index.js';
	import AdminDataTable from '#lib/features/admin/admin-data-table.svelte';
	import AdminListFooter from '#lib/features/admin/admin-list-footer.svelte';
	import AdminPageHeader from '#lib/features/admin/admin-page-header.svelte';
	import GameRowActions from '#lib/features/admin/game-row-actions.svelte';
	import GameTitleCell from '#lib/features/admin/game-title-cell.svelte';
	import StatusBadge from '#lib/features/admin/status-badge.svelte';
	import { formatRelease, type DatePrecision } from '#lib/features/games/games.js';
	import type { AdminGameRow } from '#lib/server/admin.js';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let pendingDelete = $state<AdminGameRow | null>(null);
	let deleting = $state(false);
	// svelte-ignore state_referenced_locally
	let search = $state(data.q);
	let typing: ReturnType<typeof setTimeout> | undefined;

	const statusLabels: Record<string, string> = {
		all: 'All statuses',
		published: 'Published',
		draft: 'Draft',
		hidden: 'Hidden'
	};

	const dateFormat = new Intl.DateTimeFormat('en', { dateStyle: 'medium' });

	function go(next: { q?: string; status?: string; page?: number }) {
		const params = new SvelteURLSearchParams();
		const q = next.q ?? data.q;
		const status = next.status ?? data.status;
		if (q) params.set('q', q);
		if (status !== 'all') params.set('status', status);
		if ((next.page ?? 1) > 1) params.set('page', String(next.page));
		const query = params.toString();
		goto(`/admin/games${query ? `?${query}` : ''}`, { reset: false });
	}

	function onSearch(value: string) {
		search = value;
		clearTimeout(typing);
		typing = setTimeout(() => go({ q: value.trim() }), 300);
	}

	const columns = $derived<ColumnDef<AdminGameRow>[]>([
		{
			accessorKey: 'title',
			header: 'Game',
			cell: ({ row }) =>
				renderComponent(GameTitleCell, {
					slug: row.original.slug,
					title: row.original.title,
					cover: row.original.cover_url
				})
		},
		{
			accessorKey: 'kind',
			header: 'Type',
			cell: ({ row }) => row.original.kind
		},
		{
			accessorKey: 'status',
			header: 'Status',
			cell: ({ row }) => renderComponent(StatusBadge, { status: row.original.status })
		},
		{
			accessorKey: 'release_date',
			header: 'Release',
			cell: ({ row }) =>
				formatRelease(row.original.release_date, row.original.release_precision as DatePrecision)
		},
		{
			accessorKey: 'updated_at',
			header: 'Updated',
			cell: ({ row }) => dateFormat.format(new Date(row.original.updated_at))
		},
		{
			id: 'actions',
			header: '',
			cell: ({ row }) =>
				renderComponent(GameRowActions, {
					slug: row.original.slug,
					published: row.original.status === 'published',
					canDelete: data.canDelete,
					onDelete: () => (pendingDelete = row.original)
				})
		}
	]);
</script>

<AdminPageHeader title="Games" description="Everything in the catalog, drafts included.">
	{#snippet actions()}
		<Button href="/admin/games/new" size="lg"><PlusIcon />New game</Button>
	{/snippet}
</AdminPageHeader>

<div class="mb-4 flex flex-col gap-3 sm:flex-row">
	<div class="relative flex-1">
		<SearchIcon
			class="pointer-events-none absolute start-4 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
			aria-hidden="true"
		/>
		<Input
			type="search"
			value={search}
			oninput={(event) => onSearch(event.currentTarget.value)}
			placeholder="Search by title or slug"
			aria-label="Search games"
			class="ps-10"
			maxlength={100}
		/>
	</div>
	<Select.Root type="single" value={data.status} onValueChange={(status) => go({ status })}>
		<Select.Trigger class="h-11 w-full rounded-full sm:w-48" aria-label="Filter by status">
			{statusLabels[data.status]}
		</Select.Trigger>
		<Select.Content>
			{#each Object.entries(statusLabels) as [value, label] (value)}
				<Select.Item {value} {label} />
			{/each}
		</Select.Content>
	</Select.Root>
</div>

{#if data.failed}
	<p class="mb-4 text-callout text-destructive" role="alert">
		The games could not be loaded. Try again.
	</p>
{/if}

<AdminDataTable
	{columns}
	data={data.games}
	empty={data.q || data.status !== 'all' ? 'No games match.' : 'There are no games yet.'}
/>
<AdminListFooter
	total={data.total}
	perPage={data.perPage}
	page={data.page}
	onPage={(page) => go({ page })}
/>

<AlertDialog.Root
	open={pendingDelete !== null}
	onOpenChange={(open) => {
		if (!open && !deleting) pendingDelete = null;
	}}
>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Delete “{pendingDelete?.title}”?</AlertDialog.Title>
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
					if (result.type === 'success') {
						toast.success('Game deleted');
						pendingDelete = null;
					} else if (result.type === 'failure') {
						toast.error(String(result.data?.error ?? 'Could not delete the game.'));
					}
					await update({ reset: false });
				};
			}}
		>
			<input type="hidden" name="id" value={pendingDelete?.id ?? ''} />
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
