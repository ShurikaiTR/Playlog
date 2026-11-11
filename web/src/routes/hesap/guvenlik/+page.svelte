<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import ChangePasswordDialog from '#lib/components/settings/change-password-dialog.svelte';
	import SetPasswordDialog from '#lib/components/settings/set-password-dialog.svelte';
	import SettingsGroup from '#lib/components/settings/settings-group.svelte';
	import SettingsRow from '#lib/components/settings/settings-row.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const account = $derived(data.account);

	let setOpen = $state(false);
	let changeOpen = $state(false);
	let changed = $state(false);
</script>

<svelte:head>
	<title>Playthru: Güvenlik</title>
</svelte:head>

<h1 class="mb-10 text-large-title">Güvenlik</h1>

<SettingsGroup>
	<SettingsRow
		label="Şifre ile giriş"
		description={changed
			? 'Şifren değiştirildi.'
			: account.has_password
				? 'Hesabına e-posta ve şifreyle girmek için kullandığın şifreyi yönet.'
				: 'Şu an sadece sosyal hesabınla giriş yapıyorsun. Şifre belirlersen e-postanla da girebilirsin.'}
	>
		{#if account.has_password}
			<Button variant="outline" class="h-11 px-5" onclick={() => (changeOpen = true)}
				>Şifreyi değiştir</Button
			>
		{:else}
			<Button variant="outline" class="h-11 px-5" onclick={() => (setOpen = true)}
				>Şifre belirle</Button
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
