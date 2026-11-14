<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import ChangePasswordDialog from '#lib/features/account/change-password-dialog.svelte';
	import SetPasswordDialog from '#lib/features/account/set-password-dialog.svelte';
	import SettingsGroup from '#lib/features/settings/settings-group.svelte';
	import SettingsRow from '#lib/features/settings/settings-row.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const account = $derived(data.account);

	let setOpen = $state(false);
	let changeOpen = $state(false);
	let changed = $state(false);
</script>

<svelte:head>
	<title>Playthru: Security</title>
</svelte:head>

<h1 class="mb-10 text-large-title">Security</h1>

<SettingsGroup>
	<SettingsRow
		label="Password sign-in"
		description={changed
			? 'Your password was changed.'
			: account.has_password
				? 'Manage the password you use to sign in with email.'
				: 'You currently sign in with a social account only. Set a password to sign in with your email too.'}
	>
		{#if account.has_password}
			<Button variant="outline" class="h-11 px-5" onclick={() => (changeOpen = true)}
				>Change password</Button
			>
		{:else}
			<Button variant="outline" class="h-11 px-5" onclick={() => (setOpen = true)}
				>Set password</Button
			>
		{/if}
	</SettingsRow>
</SettingsGroup>

<SetPasswordDialog bind:open={setOpen} email={account.email} />
<ChangePasswordDialog
	bind:open={changeOpen}
	email={account.email}
	onChanged={() => (changed = true)}
/>
