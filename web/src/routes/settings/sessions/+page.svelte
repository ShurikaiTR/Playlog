<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import LaptopIcon from '@lucide/svelte/icons/laptop';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import TabletIcon from '@lucide/svelte/icons/tablet';
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import SettingsGroup from '#lib/features/settings/settings-group.svelte';
	import SettingsRow from '#lib/features/settings/settings-row.svelte';
	import {
		accountErrorMessage,
		describeDevice,
		formatMediumDate
	} from '#lib/features/account/account.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const icons = { phone: SmartphoneIcon, tablet: TabletIcon, computer: LaptopIcon };

	const current = $derived(data.sessions.find((session) => session.current));
	const currentDetails = $derived.by(() => {
		if (!current) return [];
		const device = describeDevice(current.user_agent);
		return [
			['IP adresi', current.ip || 'Bilinmiyor'],
			['İşletim sistemi', device.os],
			['Tarayıcı', device.browser],
			['İstemci', 'Web'],
			['Giriş tarihi', formatMediumDate(current.created_at)],
			['Bitiş tarihi', formatMediumDate(current.expires_at)]
		];
	});

	let confirmAll = $state(false);
	let signingOut = $state(false);
</script>

<svelte:head>
	<title>Playthru: Oturumlar</title>
</svelte:head>

<div class="mb-10 flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-large-title">Oturumlar</h1>
	<Button variant="outline" class="h-11 px-5" onclick={() => (confirmAll = true)}
		>Tüm cihazlardan çıkış yap</Button
	>
</div>

{#if form?.error}
	<p class="mb-6 px-4 text-footnote text-destructive" role="alert">
		{accountErrorMessage(form.error)}
	</p>
{/if}

<div class="space-y-10">
	{#if current}
		<SettingsGroup title="Mevcut oturum">
			{#each currentDetails as [term, value] (term)}
				<SettingsRow label={term} {value} />
			{/each}
		</SettingsGroup>
	{/if}

	<!-- Like the device list of an Apple Account: each signed-in browser opens its own screen. -->
	<SettingsGroup
		title="Aktif oturumlar"
		footer="Tanımadığın bir cihaz görürsen oturumunu kapat ve şifreni değiştir."
	>
		{#each data.sessions as session (session.id)}
			{@const device = describeDevice(session.user_agent)}
			{@const Icon = icons[device.kind]}
			<SettingsRow
				label={device.name}
				description="{session.ip || '—'} · {formatMediumDate(session.created_at)} tarihinde giriş"
				href={resolve('/hesap/oturumlar/[id]', { id: session.id })}
			>
				{#snippet media()}
					<span class="flex size-9 items-center justify-center rounded-xl bg-background">
						<Icon class="size-4" strokeWidth={1.8} aria-hidden="true" />
					</span>
				{/snippet}
				{#if session.current}<Badge variant="secondary">Bu cihaz</Badge>{/if}
			</SettingsRow>
		{/each}
	</SettingsGroup>
</div>

<AlertDialog.Root bind:open={confirmAll}>
	<AlertDialog.Content size="sm">
		<form
			method="POST"
			action="?/revokeAll"
			class="grid gap-6"
			use:enhance={() => {
				signingOut = true;
				return async ({ update }) => {
					await update();
					signingOut = false;
					confirmAll = false;
				};
			}}
		>
			<AlertDialog.Header>
				<AlertDialog.Title>Tüm cihazlardan çıkış yapılsın mı?</AlertDialog.Title>
				<AlertDialog.Description>
					Bu cihaz dahil {data.sessions.length} cihazdaki oturumun kapanır. Yeniden giriş yapman gerekecek.
				</AlertDialog.Description>
			</AlertDialog.Header>
			<AlertDialog.Footer>
				<AlertDialog.Cancel type="button" class="h-11" disabled={signingOut}
					>Vazgeç</AlertDialog.Cancel
				>
				<Button type="submit" variant="destructive" class="h-11" disabled={signingOut}>
					{#if signingOut}<Spinner />{/if}Çıkış yap
				</Button>
			</AlertDialog.Footer>
		</form>
	</AlertDialog.Content>
</AlertDialog.Root>
