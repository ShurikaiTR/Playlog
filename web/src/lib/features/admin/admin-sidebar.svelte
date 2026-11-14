<script lang="ts">
	import { page } from '$app/state';
	import type { Component } from 'svelte';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
	import GamepadIcon from '@lucide/svelte/icons/gamepad-2';
	import UsersIcon from '@lucide/svelte/icons/users';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';

	let {
		staff
	}: { staff: { username: string; displayName: string | null; role: 'moderator' | 'admin' } } =
		$props();

	type Item = { label: string; href: string; icon: Component; admin?: boolean };

	// A Moderation group (edit suggestions, queue, history) goes here when that work starts.
	const groups: { label: string; items: Item[] }[] = [
		{
			label: 'Overview',
			items: [{ label: 'Dashboard', href: '/admin', icon: LayoutDashboardIcon }]
		},
		{ label: 'Catalog', items: [{ label: 'Games', href: '/admin/games', icon: GamepadIcon }] },
		{
			label: 'People',
			items: [{ label: 'Users', href: '/admin/users', icon: UsersIcon, admin: true }]
		}
	];

	const visible = $derived(
		groups
			.map((group) => ({
				...group,
				items: group.items.filter((item) => !item.admin || staff.role === 'admin')
			}))
			.filter((group) => group.items.length > 0)
	);

	const sidebar = Sidebar.useSidebar();
	const isActive = (href: string) =>
		href === '/admin' ? page.url.pathname === href : page.url.pathname.startsWith(href);
	const name = $derived(staff.displayName || staff.username);
</script>

<Sidebar.Root
	collapsible="icon"
	variant="floating"
	class="overflow-hidden [&_[data-slot=sidebar-inner]]:overflow-hidden [&_[data-slot=sidebar-inner]]:rounded-2xl"
>
	<Sidebar.Header class="p-2">
		<div class="group/brand relative flex h-10 items-center gap-2 px-1 pointer-coarse:h-11">
			<a
				href="/admin"
				class="flex min-w-0 flex-1 items-center gap-2 rounded-full text-primary outline-none group-data-[collapsible=icon]:hidden focus-visible:ring-[3px] focus-visible:ring-ring/50"
				aria-label="Admin home"
			>
				<img
					src="/logos/playthru.svg"
					alt="Playthru"
					width="282"
					height="56"
					class="h-8 w-auto dark:invert"
				/>
				<Badge variant="secondary">Admin</Badge>
			</a>
			<a
				href="/admin"
				class="hidden size-10 items-center justify-center transition-opacity group-hover/brand:opacity-0 group-data-[collapsible=icon]:flex"
				aria-label="Admin home"
			>
				<img
					src="/logos/playthru-mascot.svg"
					alt="Playthru"
					width="100"
					height="100"
					class="size-8 object-contain dark:invert"
				/>
			</a>
			<Sidebar.Trigger
				class="shrink-0 group-hover/brand:!opacity-100 group-data-[collapsible=icon]:absolute group-data-[collapsible=icon]:left-1/2 group-data-[collapsible=icon]:-translate-x-1/2 group-data-[collapsible=icon]:opacity-0 group-data-[collapsible=icon]:transition-opacity"
			/>
		</div>
	</Sidebar.Header>

	<Sidebar.Content>
		{#each visible as group (group.label)}
			<Sidebar.Group>
				<Sidebar.GroupLabel>{group.label}</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each group.items as item (item.href)}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton isActive={isActive(item.href)} tooltipContent={item.label}>
									{#snippet child({ props })}
										<a
											{...props}
											href={item.href}
											aria-current={isActive(item.href) ? 'page' : undefined}
											onclick={() => sidebar.isMobile && sidebar.setOpenMobile(false)}
										>
											<div data-sidebar="icon" aria-hidden="true"><item.icon /></div>
											<span>{item.label}</span>
										</a>
									{/snippet}
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>

	<Sidebar.Footer class="p-2">
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton tooltipContent="Back to Playthru">
					{#snippet child({ props })}
						<a {...props} href="/">
							<div data-sidebar="icon" aria-hidden="true"><ArrowLeftIcon /></div>
							<span>Back to Playthru</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
			<Sidebar.MenuItem class="group-data-[collapsible=icon]:hidden">
				<p class="truncate px-2 py-1.5 text-footnote text-muted-foreground">
					{name} · {staff.role}
				</p>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>

	<Sidebar.Rail />
</Sidebar.Root>
