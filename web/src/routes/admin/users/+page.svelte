<script lang="ts">
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import { SvelteURLSearchParams } from 'svelte/reactivity';
	import { toast } from 'svelte-sonner';
	import type { ColumnDef } from '@tanstack/table-core';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import { renderComponent } from '#lib/components/ui/data-table/index.js';
	import AdminDataTable from '#lib/features/admin/admin-data-table.svelte';
	import AdminListFooter from '#lib/features/admin/admin-list-footer.svelte';
	import AdminPageHeader from '#lib/features/admin/admin-page-header.svelte';
	import RoleBadge from '#lib/features/admin/role-badge.svelte';
	import UserCell from '#lib/features/admin/user-cell.svelte';
	import UserRowActions from '#lib/features/admin/user-row-actions.svelte';
	import { ROLES, type Role } from '#lib/features/auth/roles.js';
	import type { AdminUserRow } from '#lib/server/admin.js';
	import SearchIcon from '@lucide/svelte/icons/search';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let editing = $state<AdminUserRow | null>(null);
	let newRole = $state<Role>('user');
	let saving = $state(false);
	// svelte-ignore state_referenced_locally
	let search = $state(data.q);
	let typing: ReturnType<typeof setTimeout> | undefined;

	const label = (value: string) => value.charAt(0).toUpperCase() + value.slice(1);
	const roleFilters = ['all', ...ROLES];
	const dateFormat = new Intl.DateTimeFormat('en', { dateStyle: 'medium' });

	function go(next: { q?: string; role?: string; page?: number }) {
		const params = new SvelteURLSearchParams();
		const q = next.q ?? data.q;
		const role = next.role ?? data.role;
		if (q) params.set('q', q);
		if (role !== 'all') params.set('role', role);
		if ((next.page ?? 1) > 1) params.set('page', String(next.page));
		const query = params.toString();
		goto(`/admin/users${query ? `?${query}` : ''}`, { reset: false });
	}

	function onSearch(value: string) {
		search = value;
		clearTimeout(typing);
		typing = setTimeout(() => go({ q: value.trim() }), 300);
	}

	function edit(user: AdminUserRow) {
		editing = user;
		newRole = user.role;
	}

	const columns = $derived<ColumnDef<AdminUserRow>[]>([
		{
			accessorKey: 'username',
			header: 'User',
			cell: ({ row }) =>
				renderComponent(UserCell, {
					username: row.original.username,
					displayName: row.original.display_name
				})
		},
		{
			accessorKey: 'email',
			header: 'Email',
			cell: ({ row }) => row.original.email ?? '—'
		},
		{
			accessorKey: 'role',
			header: 'Role',
			cell: ({ row }) => renderComponent(RoleBadge, { role: row.original.role })
		},
		{
			accessorKey: 'created_at',
			header: 'Joined',
			cell: ({ row }) => dateFormat.format(new Date(row.original.created_at))
		},
		{
			id: 'actions',
			header: '',
			cell: ({ row }) =>
				renderComponent(UserRowActions, {
					disabled: row.original.id === data.meId,
					onChange: () => edit(row.original)
				})
		}
	]);
</script>

<AdminPageHeader
	title="Users"
	description="Everyone with an account. Only admins can see this page."
/>

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
			placeholder="Search by name, username or email"
			aria-label="Search users"
			class="ps-10"
			maxlength={100}
		/>
	</div>
	<Select.Root type="single" value={data.role} onValueChange={(role) => go({ role })}>
		<Select.Trigger class="h-11 w-full rounded-full sm:w-48" aria-label="Filter by role">
			{data.role === 'all' ? 'All roles' : label(data.role)}
		</Select.Trigger>
		<Select.Content>
			{#each roleFilters as value (value)}
				<Select.Item {value} label={value === 'all' ? 'All roles' : label(value)} />
			{/each}
		</Select.Content>
	</Select.Root>
</div>

{#if data.failed}
	<p class="mb-4 text-callout text-destructive" role="alert">
		The users could not be loaded. Try again.
	</p>
{/if}

<AdminDataTable
	{columns}
	data={data.users}
	empty={data.q || data.role !== 'all' ? 'No users match.' : 'There are no users yet.'}
/>
<AdminListFooter
	total={data.total}
	perPage={data.perPage}
	page={data.page}
	onPage={(page) => go({ page })}
/>

<Dialog.Root
	open={editing !== null}
	onOpenChange={(open) => {
		if (!open && !saving) editing = null;
	}}
>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Change role</Dialog.Title>
			<Dialog.Description>
				{editing?.display_name || editing?.username} (@{editing?.username}) gets the new role the
				next time they open the site.
			</Dialog.Description>
		</Dialog.Header>
		<form
			method="POST"
			action="?/role"
			class="grid gap-6"
			use:enhance={() => {
				saving = true;
				return async ({ result, update }) => {
					saving = false;
					if (result.type === 'success') {
						toast.success('Role changed');
						editing = null;
					} else if (result.type === 'failure') {
						toast.error(String(result.data?.error ?? 'Could not change the role.'));
					}
					await update({ reset: false });
				};
			}}
		>
			<input type="hidden" name="id" value={editing?.id ?? ''} />
			<Select.Root type="single" name="role" bind:value={newRole as string}>
				<Select.Trigger class="h-11 w-full rounded-full" aria-label="Role">
					{label(newRole)}
				</Select.Trigger>
				<Select.Content>
					{#each ROLES as value (value)}
						<Select.Item {value} label={label(value)} />
					{/each}
				</Select.Content>
			</Select.Root>
			<Dialog.Footer>
				<Button
					type="button"
					variant="secondary"
					size="lg"
					disabled={saving}
					onclick={() => (editing = null)}>Cancel</Button
				>
				<Button type="submit" size="lg" disabled={saving || newRole === editing?.role}>
					{saving ? 'Saving…' : 'Save role'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
