<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { Button } from '#lib/components/ui/button/index.js';
	import { settingsDialog } from '#lib/settings-dialog.svelte.js';
	import AccountAvatar from './account-avatar.svelte';
	import ScreenHeader from './screen-header.svelte';
	import SettingsGroup from './settings-group.svelte';
	import SettingsRow from './settings-row.svelte';

	// The account section of the settings window: who is signed in, "Yönet" for the /hesap page.
	const user = $derived(page.data.user);
	const name = $derived(user ? user.displayName || user.username : '');

	// Closing the window first would drop the link's own navigation, so it navigates itself.
	function manage(event: MouseEvent) {
		if (event.metaKey || event.ctrlKey || event.shiftKey || event.button !== 0) return;
		event.preventDefault();
		settingsDialog.open = false;
		goto(resolve('/hesap'));
	}
</script>

<ScreenHeader title="Hesap" />

{#if user}
	<div class="space-y-6">
		<SettingsGroup plain>
			<SettingsRow label={name} description={user.email ?? undefined}>
				{#snippet media()}<AccountAvatar {name} class="size-10" />{/snippet}
				<Button
					href={resolve('/hesap')}
					variant="outline"
					size="sm"
					class="rounded-xl px-4"
					onclick={manage}>Yönet</Button
				>
			</SettingsRow>
		</SettingsGroup>
	</div>
{/if}
