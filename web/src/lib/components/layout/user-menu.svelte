<script lang="ts">
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import { showAuthTransition } from '#lib/features/auth/auth-transition.svelte.js';
	import UserIcon from '@lucide/svelte/icons/user';
	import LogOutIcon from '@lucide/svelte/icons/log-out';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import { openSettings } from '#lib/features/settings/settings-dialog-state.svelte.js';
	import { cn } from '#lib/utils.js';
	import SettingsItems from '#lib/features/settings/settings-items.svelte';

	// Account menu at the bottom of the sidebar for signed-in users.
	let { user }: { user: { username: string; displayName: string | null } } = $props();

	const name = $derived(user.displayName || user.username);
	const initial = $derived(name.charAt(0).toLocaleUpperCase('tr'));
	const sidebar = Sidebar.useSidebar();
	const collapsed = $derived(!sidebar.isMobile && sidebar.state === 'collapsed');

	let logoutForm = $state<HTMLFormElement | null>(null);

	function logOut() {
		showAuthTransition({ title: 'Logging out', description: 'See you soon…' });
		logoutForm?.requestSubmit();
	}
</script>

<form bind:this={logoutForm} method="POST" action="/sign-out" hidden></form>

{#snippet avatar(size: string)}
	<Avatar.Root class="{size} shrink-0">
		<Avatar.Fallback class="bg-foreground text-xs font-semibold text-background"
			>{initial}</Avatar.Fallback
		>
	</Avatar.Root>
{/snippet}

<Sidebar.Menu>
	<Sidebar.MenuItem>
		<DropdownMenu.Root>
			<DropdownMenu.Trigger>
				{#snippet child({ props })}
					<Sidebar.MenuButton
						{...props}
						size="lg"
						tooltipContent={name}
						class="h-auto min-h-10 cursor-pointer gap-1.5 p-1.5 pe-2 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:p-1! data-[state=open]:bg-foreground/8 dark:data-[state=open]:bg-foreground/12 pointer-coarse:min-h-11"
					>
						{@render avatar('size-7')}
						<span class="grid min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
							<span class="truncate">{name}</span>
						</span>
					</Sidebar.MenuButton>
				{/snippet}
			</DropdownMenu.Trigger>
			<!-- Opens above the row, as wide as the sidebar. Collapsed to icons, there is no room above, so it opens beside it. -->
			<DropdownMenu.Content
				side={collapsed ? 'right' : 'top'}
				align={collapsed ? 'end' : 'start'}
				sideOffset={8}
				class={cn(
					'rounded-2xl p-2',
					collapsed ? 'w-60' : 'w-(--bits-dropdown-menu-anchor-width) min-w-0'
				)}
			>
				<DropdownMenu.Label
					class="flex items-center gap-2.5 px-2 py-1.5 font-normal text-foreground"
				>
					{@render avatar('size-8')}
					<span class="min-w-0 leading-tight">
						<span class="block truncate text-callout font-medium">{name}</span>
						<span class="block truncate text-footnote text-muted-foreground">@{user.username}</span>
					</span>
				</DropdownMenu.Label>
				<DropdownMenu.Separator />
				<DropdownMenu.Item class="cursor-pointer">
					{#snippet child({ props })}
						<a
							{...props}
							href={`/u/${user.username}`}
							onclick={() => sidebar.isMobile && sidebar.setOpenMobile(false)}
						>
							<UserIcon />My profile
						</a>
					{/snippet}
				</DropdownMenu.Item>
				<DropdownMenu.Item class="cursor-pointer" onSelect={() => openSettings('account')}>
					<SettingsIcon />Settings
				</DropdownMenu.Item>
				<DropdownMenu.Separator />
				<SettingsItems theme={false} language={false} />
				<DropdownMenu.Separator />
				<DropdownMenu.Item class="cursor-pointer" onSelect={logOut}>
					<LogOutIcon />Log out
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</Sidebar.MenuItem>
</Sidebar.Menu>
