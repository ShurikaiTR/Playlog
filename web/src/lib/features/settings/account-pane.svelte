<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { Button } from '#lib/components/ui/button/index.js';
	import { settingsDialog } from './settings-dialog-state.svelte.js';
	import AccountAvatar from '#lib/features/account/account-avatar.svelte';
	import ScreenHeader from './screen-header.svelte';

	// The account section of the settings window: who is signed in, "Yönet" for the /settings page.
	const user = $derived(page.data.user);
	const name = $derived(user ? user.displayName || user.username : '');

	// Closing the window first would drop the link's own navigation, so it navigates itself.
	function manage(event: MouseEvent) {
		if (event.metaKey || event.ctrlKey || event.shiftKey || event.button !== 0) return;
		event.preventDefault();
		settingsDialog.open = false;
		goto(resolve('/settings'));
	}
</script>

<ScreenHeader title="Hesap" />

{#if user}
	<div class="flex flex-col gap-6">
		<div class="flex items-center justify-between gap-4 px-5">
			<div class="flex min-w-0 items-center gap-3">
				<AccountAvatar {name} class="size-9" />
				<div class="min-w-0 leading-tight">
					<p class="truncate text-sm font-medium">{name}</p>
					{#if user.email}
						<p class="truncate text-xs text-muted-foreground">{user.email}</p>
					{/if}
				</div>
			</div>
			<Button
				href={resolve('/settings')}
				variant="outline"
				size="action"
				class="shrink-0"
				onclick={manage}>Yönet</Button
			>
		</div>
		<hr class="mx-5 border-border" />
	</div>
{/if}
