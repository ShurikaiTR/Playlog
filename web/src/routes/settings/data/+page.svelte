<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import DeleteAccountDialog from '#lib/features/account/delete-account-dialog.svelte';
	import SettingsGroup from '#lib/features/settings/settings-group.svelte';
	import SettingsRow from '#lib/features/settings/settings-row.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let deleteOpen = $state(false);
</script>

<svelte:head>
	<title>Playthru: Data</title>
</svelte:head>

<h1 class="mb-10 text-large-title">Data</h1>

<div class="space-y-10">
	<SettingsGroup>
		<SettingsRow
			label="Download my data"
			description="Your profile, sign-in methods and active sessions, as a single file."
		>
			<Button
				href="/settings/data/export"
				download
				data-sveltekit-reload
				variant="outline"
				class="h-11 px-5">Download</Button
			>
		</SettingsRow>
	</SettingsGroup>

	<SettingsGroup>
		<SettingsRow
			label="Delete account"
			description="Your account and everything connected to it are deleted for good. This can’t be undone."
		>
			<Button variant="destructive" class="h-11 px-5" onclick={() => (deleteOpen = true)}
				>Delete</Button
			>
		</SettingsRow>
	</SettingsGroup>
</div>

<DeleteAccountDialog bind:open={deleteOpen} hasPassword={data.account.has_password} />
