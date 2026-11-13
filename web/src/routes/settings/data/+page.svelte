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
	<title>Playthru: Veri</title>
</svelte:head>

<h1 class="mb-10 text-large-title">Veri</h1>

<div class="space-y-10">
	<SettingsGroup>
		<SettingsRow
			label="Verilerimi indir"
			description="Profilin, giriş yöntemlerin ve açık oturumların tek bir dosya olarak iner."
		>
			<Button
				href="/settings/data/export"
				download
				data-sveltekit-reload
				variant="outline"
				class="h-11 px-5">İndir</Button
			>
		</SettingsRow>
	</SettingsGroup>

	<SettingsGroup>
		<SettingsRow
			label="Hesabı sil"
			description="Hesabın ve ona bağlı tüm veriler kalıcı olarak silinir. Bu işlem geri alınamaz."
		>
			<Button variant="destructive" class="h-11 px-5" onclick={() => (deleteOpen = true)}
				>Sil</Button
			>
		</SettingsRow>
	</SettingsGroup>
</div>

<DeleteAccountDialog bind:open={deleteOpen} hasPassword={data.account.has_password} />
