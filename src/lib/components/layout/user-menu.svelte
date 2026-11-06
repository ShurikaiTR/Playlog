<script lang="ts">
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import { showAuthTransition } from '#lib/auth-transition.svelte.js';
	import UserIcon from '@lucide/svelte/icons/user';
	import LogOutIcon from '@lucide/svelte/icons/log-out';

	let { username }: { username: string } = $props();

	let logoutForm = $state<HTMLFormElement | null>(null);

	function logOut() {
		showAuthTransition({ title: 'Çıkış yapılıyor', description: 'Görüşmek üzere…' });
		logoutForm?.requestSubmit();
	}
</script>

<form bind:this={logoutForm} method="POST" action="/cikis" hidden></form>

<DropdownMenu.Root>
	<DropdownMenu.Trigger
		class="rounded-full outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
		aria-label="Hesap menüsü"
	>
		<Avatar.Root class="size-9">
			<Avatar.Fallback class="text-callout font-medium uppercase"
				>{username.slice(0, 1)}</Avatar.Fallback
			>
		</Avatar.Root>
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end" class="w-48">
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
