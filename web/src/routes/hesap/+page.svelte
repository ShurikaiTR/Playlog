<script lang="ts">
	import { invalidateAll, replaceState } from '$app/navigation';
	import { page } from '$app/state';
	import LaptopIcon from '@lucide/svelte/icons/laptop';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import SunIcon from '@lucide/svelte/icons/sun';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import { resetMode, setMode, userPrefersMode } from 'mode-watcher';
	import { toast } from 'svelte-sonner';
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import AppleIcon from '#lib/components/icons/apple.svelte';
	import DiscordIcon from '#lib/components/icons/discord.svelte';
	import SteamIcon from '#lib/components/icons/steam.svelte';
	import GoogleIcon from '#lib/components/icons/google.svelte';
	import MailIcon from '#lib/components/icons/mail.svelte';
	import AccountAvatar from '#lib/components/settings/account-avatar.svelte';
	import ChangePasswordDialog from '#lib/components/settings/change-password-dialog.svelte';
	import SetPasswordDialog from '#lib/components/settings/set-password-dialog.svelte';
	import SettingsGroup from '#lib/components/settings/settings-group.svelte';
	import SettingsRow from '#lib/components/settings/settings-row.svelte';
	import {
		accountErrorMessage,
		accountRequest,
		formatDate,
		providerLabel,
		providers,
		providerSignInLabel,
		type Provider
	} from '#lib/account.js';
	import { cn } from '#lib/utils.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const account = $derived(data.account);
	const name = $derived(account.display_name || account.username);
	const icons = { apple: AppleIcon, google: GoogleIcon, discord: DiscordIcon, steam: SteamIcon };

	// ---------- Profile ----------

	let profileOpen = $state(false);
	let nameInput = $state('');
	let usernameInput = $state('');
	let saving = $state(false);
	let saveError = $state('');

	function openProfile() {
		nameInput = account.display_name ?? '';
		usernameInput = account.username;
		saveError = '';
		profileOpen = true;
	}

	async function saveProfile(event: SubmitEvent) {
		event.preventDefault();
		saving = true;
		saveError = '';
		const result = await accountRequest('PUT', '/profile', {
			username: usernameInput.trim().toLowerCase(),
			display_name: nameInput.trim() || null
		});
		if (!result.ok) saveError = accountErrorMessage(result.error);
		else {
			await invalidateAll();
			profileOpen = false;
		}
		saving = false;
	}

	// ---------- Theme ----------

	const themes = [
		{ id: 'dark', label: 'Koyu tema', icon: MoonIcon },
		{ id: 'light', label: 'Açık tema', icon: SunIcon },
		{ id: 'system', label: 'Sistem teması', icon: LaptopIcon }
	] as const;

	function changeTheme(id: (typeof themes)[number]['id']) {
		if (id === 'system') resetMode();
		else setMode(id);
	}

	// ---------- Sign-in methods ----------

	const linked = $derived(new Map(account.sign_in_methods.map((m) => [m.provider, m.created_at])));
	// The last remaining sign-in method can't be removed.
	const canRemove = $derived(account.sign_in_methods.length + (account.has_password ? 1 : 0) > 1);

	let setPasswordOpen = $state(false);
	let changePasswordOpen = $state(false);
	let busyProvider = $state<Provider | null>(null);
	// Shown when someone tries to remove their only sign-in method.
	let lastMethodOpen = $state(false);
	let removing = $state<Provider | null>(null);
	let methodError = $state('');

	function connect(provider: Provider) {
		methodError = '';
		// Providers without keys on the server are listed but can't be connected yet.
		if (!account.available[provider]) {
			methodError = `${providerSignInLabel(provider)} henüz açık değil.`;
			return;
		}
		busyProvider = provider;
		// The provider sends the browser back to this page.
		window.location.assign(`/auth/${provider}/start?baglan&geri=%2Fhesap`);
	}

	function askDisconnect(provider: Provider) {
		methodError = '';
		if (!canRemove) lastMethodOpen = true;
		else removing = provider;
	}

	async function disconnect() {
		const provider = removing;
		if (!provider) return;
		busyProvider = provider;
		const result = await accountRequest('DELETE', `/sign-in-methods/${provider}`);
		if (!result.ok) {
			if (result.error === 'last_sign_in_method') lastMethodOpen = true;
			else methodError = accountErrorMessage(result.error);
		} else {
			await invalidateAll();
			toast.success(`${providerLabel(provider)} bağlantısı kaldırıldı`);
		}
		busyProvider = null;
		removing = null;
	}

	// Back from connecting a provider: say how it went, then drop the query so a reload doesn't repeat it.
	$effect(() => {
		const params = page.url.searchParams;
		const done = params.get('baglandi');
		const failed = params.get('hata');
		if (!done && !failed) return;
		if (done) toast.success(`${providerLabel(done as Provider)} hesabın bağlandı`);
		else methodError = accountErrorMessage(failed ?? undefined);
		// The current page's own URL, already resolved; only the query changes.
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		replaceState(page.url.pathname, page.state);
	});
</script>

<svelte:head>
	<title>Playthru: Hesap</title>
</svelte:head>

<h1 class="mb-10 text-large-title">Hesap</h1>

<div class="space-y-10">
	<SettingsGroup>
		<!-- Apple Account style header: who is signed in, with the edit action beside it. -->
		<div class="flex min-h-20 items-center justify-between gap-4 px-4 py-4">
			<div class="flex min-w-0 items-center gap-3.5">
				<AccountAvatar {name} src={account.avatar_url} class="size-14 text-title-3" />
				<div class="min-w-0 leading-tight">
					<p class="truncate text-title-3">{name}</p>
					<p class="truncate text-footnote text-muted-foreground">@{account.username}</p>
				</div>
			</div>
			<Button variant="outline" class="h-11 px-5" onclick={openProfile}>Değiştir</Button>
		</div>
		<SettingsRow label="E-posta" value={account.email ?? 'Eklenmedi'} />
		<SettingsRow label="Hesap oluşturma" value={formatDate(account.created_at)} />
		<SettingsRow label="Tema">
			<div class="flex gap-1" role="group" aria-label="Tema seç">
				{#each themes as theme (theme.id)}
					{@const selected = userPrefersMode.current === theme.id}
					<Button
						variant="ghost"
						size="icon"
						class={cn(
							'size-11 rounded-full',
							selected ? 'bg-muted text-foreground' : 'text-muted-foreground'
						)}
						aria-label={theme.label}
						title={theme.label}
						aria-pressed={selected}
						onclick={() => changeTheme(theme.id)}
					>
						<theme.icon class="size-4" />
					</Button>
				{/each}
			</div>
		</SettingsRow>
	</SettingsGroup>

	<SettingsGroup
		title="Giriş yöntemleri"
		footer="Hesabına girmenin son yolu kaldırılamaz; önce başka bir giriş yöntemi ekle."
	>
		<SettingsRow label="E-posta ve şifre ile giriş">
			{#snippet media()}
				<span class="flex size-9 items-center justify-center rounded-full bg-background">
					<MailIcon class="size-4" />
				</span>
			{/snippet}
			{#if account.has_password}
				<Button variant="outline" class="h-11 px-5" onclick={() => (changePasswordOpen = true)}
					>Şifreyi değiştir</Button
				>
			{:else}
				<Button class="h-11 px-5" onclick={() => (setPasswordOpen = true)}>Etkinleştir</Button>
			{/if}
		</SettingsRow>

		{#each providers as provider (provider.id)}
			{@const Icon = icons[provider.id]}
			{@const since = linked.get(provider.id)}
			<SettingsRow
				label={providerSignInLabel(provider.id)}
				description={since ? `${formatDate(since)} tarihinde bağlandı` : undefined}
			>
				{#snippet media()}
					<span class="flex size-9 items-center justify-center rounded-full bg-background">
						<Icon class="size-4" />
					</span>
				{/snippet}
				{#if since}
					<Button
						variant="outline"
						class="h-11 px-5"
						disabled={busyProvider !== null}
						onclick={() => askDisconnect(provider.id)}>Kaldır</Button
					>
				{:else}
					<Button
						class="h-11 px-5"
						disabled={busyProvider !== null}
						onclick={() => connect(provider.id)}
					>
						{#if busyProvider === provider.id}<Spinner />{/if}Bağla
					</Button>
				{/if}
			</SettingsRow>
		{/each}
	</SettingsGroup>
</div>
{#if methodError}
	<p class="mt-3 px-4 text-footnote text-destructive" role="alert">{methodError}</p>
{/if}

<Dialog.Root bind:open={profileOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title class="text-headline">Profili düzenle</Dialog.Title>
			<Dialog.Description>Görünen adın ve kullanıcı adın sitede herkese görünür.</Dialog.Description
			>
		</Dialog.Header>
		<form class="space-y-4" onsubmit={saveProfile}>
			<div class="space-y-2">
				<Label for="profile-name">Görünen ad</Label>
				<Input
					id="profile-name"
					bind:value={nameInput}
					maxlength={40}
					autocomplete="name"
					placeholder={account.username}
				/>
			</div>
			<div class="space-y-2">
				<Label for="profile-username">Kullanıcı adı</Label>
				<Input
					id="profile-username"
					bind:value={usernameInput}
					required
					minlength={3}
					maxlength={20}
					autocomplete="username"
					autocapitalize="none"
					spellcheck={false}
					aria-describedby="profile-username-hint"
				/>
				<p id="profile-username-hint" class="px-4 text-footnote text-muted-foreground">
					Küçük harf, rakam ve alt çizgi. Profil adresin de değişir.
				</p>
			</div>
			{#if saveError}
				<p class="text-footnote text-destructive" role="alert">{saveError}</p>
			{/if}
			<Dialog.Footer>
				<Button
					type="button"
					variant="outline"
					class="h-11 px-5"
					onclick={() => (profileOpen = false)}>Vazgeç</Button
				>
				<Button type="submit" class="h-11 px-5" disabled={saving}>
					{#if saving}<Spinner />{/if}Kaydet
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={lastMethodOpen}>
	<Dialog.Content class="text-center sm:max-w-sm">
		<Dialog.Header class="items-center gap-3 pt-2">
			<TriangleAlertIcon class="size-8" strokeWidth={1.6} />
			<Dialog.Title class="text-center text-headline">Giriş yöntemi kaldırılamıyor</Dialog.Title>
			<Dialog.Description class="text-center">
				Bu hesapta başka bir giriş yöntemin yok. Devam etmek için önce başka bir giriş yöntemi ekle.
			</Dialog.Description>
		</Dialog.Header>
		<Button variant="outline" class="h-11 w-full" onclick={() => (lastMethodOpen = false)}
			>Kapat</Button
		>
	</Dialog.Content>
</Dialog.Root>

<AlertDialog.Root open={removing !== null} onOpenChange={(open) => open || (removing = null)}>
	<AlertDialog.Content size="sm">
		{#if removing}
			<AlertDialog.Header>
				<AlertDialog.Title>{providerLabel(removing)} bağlantısı kaldırılsın mı?</AlertDialog.Title>
				<AlertDialog.Description>
					Artık {providerLabel(removing)} ile giriş yapamazsın. İstediğin zaman yeniden bağlayabilirsin.
				</AlertDialog.Description>
			</AlertDialog.Header>
			<AlertDialog.Footer>
				<AlertDialog.Cancel class="h-11" disabled={busyProvider !== null}>Vazgeç</AlertDialog.Cancel
				>
				<AlertDialog.Action
					variant="destructive"
					class="h-11"
					disabled={busyProvider !== null}
					onclick={disconnect}
				>
					{#if busyProvider}<Spinner />{/if}Bağlantıyı kaldır
				</AlertDialog.Action>
			</AlertDialog.Footer>
		{/if}
	</AlertDialog.Content>
</AlertDialog.Root>

<SetPasswordDialog bind:open={setPasswordOpen} email={account.email} />
<ChangePasswordDialog
	bind:open={changePasswordOpen}
	email={account.email}
	onChanged={() => toast.success('Şifren değiştirildi')}
/>
