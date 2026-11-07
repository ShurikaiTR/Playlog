<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import LogOutIcon from '@lucide/svelte/icons/log-out';
	import { showAuthTransition } from '#lib/auth-transition.svelte.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { settingsDialog } from '#lib/settings-dialog.svelte.js';
	import AccountAvatar from './account-avatar.svelte';
	import ScreenHeader from './screen-header.svelte';
	import SettingsGroup from './settings-group.svelte';
	import SettingsRow from './settings-row.svelte';

	// The account section of the settings window: who is signed in, "Yönet" for the /hesap page,
	// and signing out.
	const user = $derived(page.data.user);
	const name = $derived(user ? user.displayName || user.username : '');

	let logoutForm = $state<HTMLFormElement | null>(null);

	// Closing the window first would drop the link's own navigation, so it navigates itself.
	function manage(event: MouseEvent) {
		if (event.metaKey || event.ctrlKey || event.shiftKey || event.button !== 0) return;
		event.preventDefault();
		settingsDialog.open = false;
		goto(resolve('/hesap'));
	}

	function signOut() {
		showAuthTransition({ title: 'Çıkış yapılıyor', description: 'Görüşmek üzere…' });
		logoutForm?.requestSubmit();
		settingsDialog.open = false;
	}
</script>

<form bind:this={logoutForm} method="POST" action="/cikis" hidden></form>

<ScreenHeader title="Hesap" />

{#if user}
	<div class="space-y-6">
		<SettingsGroup plain>
			<SettingsRow label={name} description={user.email}>
				{#snippet media()}<AccountAvatar {name} class="size-10" />{/snippet}
				<Button
					href={resolve('/hesap')}
					variant="outline"
					class="h-11 rounded-xl px-5"
					onclick={manage}>Yönet</Button
				>
			</SettingsRow>
		</SettingsGroup>
		<SettingsGroup plain>
			<SettingsRow label="Oturum">
				<Button variant="outline" class="h-11 rounded-xl px-5" onclick={signOut}>
					<LogOutIcon />Çıkış yap
				</Button>
			</SettingsRow>
		</SettingsGroup>
	</div>
{/if}
