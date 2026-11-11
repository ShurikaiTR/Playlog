<script lang="ts">
	import { enhance } from '$app/forms';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import LaptopIcon from '@lucide/svelte/icons/laptop';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import TabletIcon from '@lucide/svelte/icons/tablet';
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import SettingsGroup from '#lib/features/settings/settings-group.svelte';
	import SettingsRow from '#lib/features/settings/settings-row.svelte';
	import { describeDevice, formatDateTime, formatMediumDate } from '#lib/features/account/account.js';
	import type { PageProps } from './$types';

	// One device of the session list, as in an Apple Account: its details and a way to sign it out.
	let { data, params }: PageProps = $props();

	const icons = { phone: SmartphoneIcon, tablet: TabletIcon, computer: LaptopIcon };

	const session = $derived(data.sessions.find((item) => item.id === params.id));
	const device = $derived(session ? describeDevice(session.user_agent) : null);

	let confirmOpen = $state(false);
	let pending = $state(false);
	let logoutForm = $state<HTMLFormElement | null>(null);
</script>

<svelte:head>
	<title>Playthru: {device?.name ?? 'Oturum'}</title>
</svelte:head>

<Button
	href="/hesap/oturumlar"
	variant="ghost"
	class="-ms-3 mb-4 h-11 gap-0.5 rounded-xl ps-1.5 pe-3 text-callout font-normal"
>
	<ChevronLeftIcon class="size-5" aria-hidden="true" />Oturumlar
</Button>

{#if session && device}
	{@const Icon = icons[device.kind]}
	<div class="mb-10 flex flex-col items-center gap-3 text-center">
		<span class="flex size-20 items-center justify-center rounded-3xl bg-muted">
			<Icon class="size-9" strokeWidth={1.5} aria-hidden="true" />
		</span>
		<div>
			<h1 class="text-title-2">{device.name}</h1>
			{#if session.current}
				<p class="text-footnote text-muted-foreground">Bu cihaz</p>
			{/if}
		</div>
	</div>

	<div class="space-y-10">
		<SettingsGroup>
			<SettingsRow label="IP adresi" value={session.ip || 'Bilinmiyor'} />
			<SettingsRow label="İşletim sistemi" value={device.os} />
			<SettingsRow label="Tarayıcı" value={device.browser} />
			<SettingsRow label="İstemci" value="Web" />
			<SettingsRow label="Son etkinlik" value={formatDateTime(session.last_seen_at)} />
			<SettingsRow label="Giriş tarihi" value={formatMediumDate(session.created_at)} />
			<SettingsRow label="Bitiş tarihi" value={formatMediumDate(session.expires_at)} />
		</SettingsGroup>

		<SettingsGroup
			footer={session.current
				? 'Bu cihazda Playthru hesabından çıkış yaparsın.'
				: 'Bu cihaz yeniden giriş yapana kadar hesabına erişemez.'}
		>
			<SettingsRow label="Oturumu kapat" destructive onclick={() => (confirmOpen = true)} />
		</SettingsGroup>
	</div>

	<form bind:this={logoutForm} method="POST" action="/cikis" hidden></form>

	<AlertDialog.Root bind:open={confirmOpen}>
		<AlertDialog.Content size="sm">
			<form
				method="POST"
				action="/hesap/oturumlar?/revoke"
				class="grid gap-6"
				use:enhance={({ cancel }) => {
					pending = true;
					// This device signs out like "Çıkış yap" does.
					if (session.current) {
						cancel();
						logoutForm?.requestSubmit();
						return;
					}
					return async ({ update }) => {
						await update();
						pending = false;
						confirmOpen = false;
					};
				}}
			>
				<input type="hidden" name="id" value={session.id} />
				<AlertDialog.Header>
					<AlertDialog.Title
						>{session.current
							? 'Bu cihazda çıkış yapılsın mı?'
							: 'Oturum kapatılsın mı?'}</AlertDialog.Title
					>
					<AlertDialog.Description>
						{session.current
							? 'Yeniden giriş yapman gerekecek.'
							: `${device.name} hesabına yeniden giriş yapana kadar erişemez.`}
					</AlertDialog.Description>
				</AlertDialog.Header>
				<AlertDialog.Footer>
					<AlertDialog.Cancel type="button" class="h-11" disabled={pending}
						>Vazgeç</AlertDialog.Cancel
					>
					<Button type="submit" variant="destructive" class="h-11" disabled={pending}>
						{#if pending}<Spinner />{/if}{session.current ? 'Çıkış yap' : 'Oturumu kapat'}
					</Button>
				</AlertDialog.Footer>
			</form>
		</AlertDialog.Content>
	</AlertDialog.Root>
{:else}
	<h1 class="mb-3 text-title-2">Oturum bulunamadı</h1>
	<p class="text-muted-foreground">Bu cihazın oturumu artık açık değil.</p>
{/if}
