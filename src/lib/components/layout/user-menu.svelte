<script lang="ts">
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import { showAuthTransition } from '#lib/auth-transition.svelte.js';
	import UserIcon from '@lucide/svelte/icons/user';
	import LogOutIcon from '@lucide/svelte/icons/log-out';

	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';

	let { username }: { username: string } = $props();
	const sidebar = Sidebar.useSidebar();

	let logoutForm = $state<HTMLFormElement | null>(null);

	function logOut() {
		showAuthTransition({ title: 'Çıkış yapılıyor', description: 'Görüşmek üzere…' });
		logoutForm?.requestSubmit();
	}
</script>

<form bind:this={logoutForm} method="POST" action="/cikis" hidden></form>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Sidebar.MenuButton {...props} size="lg" tooltipContent={`@${username}`} aria-label="Hesap menüsü" class="rounded-xl">
				<Avatar.Root class="size-8">
					<Avatar.Fallback class="text-callout font-medium uppercase"
						>{username.slice(0, 1)}</Avatar.Fallback
					>
				</Avatar.Root>
				<span class="truncate font-medium">@{username}</span>
				<ChevronsUpDownIcon class="ms-auto size-4" />
			</Sidebar.MenuButton>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content side={sidebar.isMobile ? "top" : "right"} align="end" sideOffset={10} class="w-48">
		<DropdownMenu.Label class="truncate text-footnote text-muted-foreground"
			>@{username}</DropdownMenu.Label
		>
		<DropdownMenu.Separator />
		<DropdownMenu.Item>
			{#snippet child({ props })}
				<a {...props} href={`/profil/${username}`}><UserIcon />Profilim</a>
			{/snippet}
		</DropdownMenu.Item>
		<DropdownMenu.Item onSelect={logOut}><LogOutIcon />Çıkış yap</DropdownMenu.Item>
	</DropdownMenu.Content>
</DropdownMenu.Root>
