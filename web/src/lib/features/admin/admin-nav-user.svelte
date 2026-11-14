<script lang="ts">
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import EllipsisVerticalIcon from '@lucide/svelte/icons/ellipsis-vertical';
	import LogOutIcon from '@lucide/svelte/icons/log-out';
	import UserIcon from '@lucide/svelte/icons/user';

	let {
		staff
	}: { staff: { username: string; displayName: string | null; role: 'moderator' | 'admin' } } =
		$props();

	const sidebar = Sidebar.useSidebar();
	const name = $derived(staff.displayName || staff.username);
	const initial = $derived(name.charAt(0).toLocaleUpperCase('en'));
	let logoutForm = $state<HTMLFormElement | null>(null);
</script>

<form bind:this={logoutForm} method="POST" action="/sign-out" hidden></form>

{#snippet avatar()}
	<Avatar.Root class="size-8 shrink-0">
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
						class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
					>
						{@render avatar()}
						<div class="grid flex-1 text-start text-sm leading-tight">
							<span class="truncate font-medium">{name}</span>
							<span class="truncate text-xs text-muted-foreground capitalize">{staff.role}</span>
						</div>
						<EllipsisVerticalIcon class="ms-auto size-4" />
					</Sidebar.MenuButton>
				{/snippet}
			</DropdownMenu.Trigger>
			<DropdownMenu.Content
				class="w-(--bits-dropdown-menu-anchor-width) min-w-56 rounded-2xl p-2"
				side={sidebar.isMobile ? 'bottom' : 'right'}
				align="end"
				sideOffset={4}
			>
				<DropdownMenu.Label class="flex items-center gap-2 px-2 py-1.5 font-normal">
					{@render avatar()}
					<div class="grid flex-1 text-start leading-tight">
						<span class="truncate text-callout font-medium">{name}</span>
						<span class="truncate text-footnote text-muted-foreground">@{staff.username}</span>
					</div>
				</DropdownMenu.Label>
				<DropdownMenu.Separator />
				<DropdownMenu.Item class="cursor-pointer">
					{#snippet child({ props })}
						<a {...props} href={`/@${staff.username}`}><UserIcon />My profile</a>
					{/snippet}
				</DropdownMenu.Item>
				<DropdownMenu.Item class="cursor-pointer" onSelect={() => logoutForm?.requestSubmit()}>
					<LogOutIcon />Log out
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</Sidebar.MenuItem>
</Sidebar.Menu>
