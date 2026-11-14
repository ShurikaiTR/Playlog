<script lang="ts">
	import { page } from '$app/state';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import type { AdminNavItem } from './admin-nav.js';

	let { items }: { items: AdminNavItem[] } = $props();

	const sidebar = Sidebar.useSidebar();
	const isActive = (url: string) =>
		url === '/admin' ? page.url.pathname === url : page.url.pathname.startsWith(url);
	const close = () => sidebar.isMobile && sidebar.setOpenMobile(false);
</script>

<Sidebar.Group>
	<Sidebar.GroupContent class="flex flex-col gap-2">
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton
					class="min-w-8 bg-primary text-primary-foreground duration-200 ease-linear hover:bg-primary/90 hover:text-primary-foreground active:bg-primary/90 active:text-primary-foreground dark:hover:bg-primary/90"
					tooltipContent="New game"
				>
					{#snippet child({ props })}
						<a {...props} href="/admin/games/new" onclick={close}>
							<div data-sidebar="icon" aria-hidden="true"><PlusIcon /></div>
							<span>New game</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
		<Sidebar.Menu>
			{#each items as item (item.url)}
				<Sidebar.MenuItem>
					<Sidebar.MenuButton isActive={isActive(item.url)} tooltipContent={item.title}>
						{#snippet child({ props })}
							<a
								{...props}
								href={item.url}
								aria-current={isActive(item.url) ? 'page' : undefined}
								onclick={close}
							>
								<div data-sidebar="icon" aria-hidden="true"><item.icon /></div>
								<span>{item.title}</span>
							</a>
						{/snippet}
					</Sidebar.MenuButton>
				</Sidebar.MenuItem>
			{/each}
		</Sidebar.Menu>
	</Sidebar.GroupContent>
</Sidebar.Group>
