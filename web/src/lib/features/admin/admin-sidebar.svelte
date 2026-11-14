<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import AdminNavMain from './admin-nav-main.svelte';
	import AdminNavSecondary from './admin-nav-secondary.svelte';
	import AdminNavUser from './admin-nav-user.svelte';
	import { adminNav } from './admin-nav.js';

	type Staff = { username: string; displayName: string | null; role: 'moderator' | 'admin' };

	let { staff, ...restProps }: { staff: Staff } & ComponentProps<typeof Sidebar.Root> = $props();

	const items = $derived(adminNav.filter((item) => !item.adminOnly || staff.role === 'admin'));
</script>

<Sidebar.Root collapsible="offcanvas" {...restProps}>
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton class="h-11 data-[slot=sidebar-menu-button]:!p-1.5">
					{#snippet child({ props })}
						<a {...props} href="/admin" aria-label="Admin home">
							<img
								src="/logos/playthru.svg"
								alt="Playthru"
								width="282"
								height="56"
								class="h-7 w-auto dark:invert"
							/>
							<Badge variant="secondary">Admin</Badge>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>
	<Sidebar.Content>
		<AdminNavMain {items} />
		<AdminNavSecondary class="mt-auto" />
	</Sidebar.Content>
	<Sidebar.Footer>
		<AdminNavUser {staff} />
	</Sidebar.Footer>
</Sidebar.Root>
