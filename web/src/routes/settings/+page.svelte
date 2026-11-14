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
	import { providerIcons } from '#lib/features/auth/provider-icons.js';
	import MailIcon from '#lib/components/icons/mail.svelte';
	import AccountAvatar from '#lib/features/account/account-avatar.svelte';
	import ChangePasswordDialog from '#lib/features/account/change-password-dialog.svelte';
	import SetPasswordDialog from '#lib/features/account/set-password-dialog.svelte';
	import SettingsGroup from '#lib/features/settings/settings-group.svelte';
	import SettingsRow from '#lib/features/settings/settings-row.svelte';
	import {
		accountErrorMessage,
		accountRequest,
		formatDate,
		providerLabel,
		providers,
		providerSignInLabel,
		type Provider
	} from '#lib/features/account/account.js';
	import { cn } from '#lib/utils.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const account = $derived(data.account);
	const name = $derived(account.display_name || account.username);

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
		{ id: 'dark', label: 'Dark theme', icon: MoonIcon },
		{ id: 'light', label: 'Light theme', icon: SunIcon },
		{ id: 'system', label: 'System theme', icon: LaptopIcon }
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
			methodError = `${providerSignInLabel(provider)} isn’t available yet.`;
			return;
		}
		busyProvider = provider;
		// The provider sends the browser back to this page.
		window.location.assign(`/auth/${provider}/start?link&return_to=%2Fsettings`);
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
			toast.success(`${providerLabel(provider)} was disconnected`);
		}
		busyProvider = null;
		removing = null;
	}

	// Back from connecting a provider: say how it went, then drop the query so a reload doesn't repeat it.
	$effect(() => {
		const params = page.url.searchParams;
		const done = params.get('linked');
		const failed = params.get('error');
		if (!done && !failed) return;
		if (done) toast.success(`${providerLabel(done as Provider)} was connected`);
		else methodError = accountErrorMessage(failed ?? undefined);
		// Only the query changes; the path stays the current page.
		replaceState(page.url.pathname, page.state);
	});
</script>

<svelte:head>
	<title>Playthru: Account</title>
</svelte:head>

<h1 class="mb-10 text-large-title">Account</h1>

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
			<Button variant="outline" class="h-11 px-5" onclick={openProfile}>Change</Button>
		</div>
		<SettingsRow label="Email" value={account.email ?? 'Not added'} />
		<SettingsRow label="Account created" value={formatDate(account.created_at)} />
		<SettingsRow label="Theme">
			<div class="flex gap-1" role="group" aria-label="Choose a theme">
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
		title="Sign-in methods"
		footer="You can’t remove your last way to sign in. Add another method first."
	>
		<SettingsRow label="Sign in with email and password">
			{#snippet media()}
				<span class="flex size-9 items-center justify-center rounded-full bg-background">
					<MailIcon class="size-4" />
				</span>
			{/snippet}
			{#if account.has_password}
				<Button variant="outline" class="h-11 px-5" onclick={() => (changePasswordOpen = true)}
					>Change password</Button
				>
			{:else}
				<Button class="h-11 px-5" onclick={() => (setPasswordOpen = true)}>Turn on</Button>
			{/if}
		</SettingsRow>

		{#each providers as provider (provider.id)}
			{@const Icon = providerIcons[provider.id]}
			{@const since = linked.get(provider.id)}
			<SettingsRow
				label={providerSignInLabel(provider.id)}
				description={since ? `Connected on ${formatDate(since)}` : undefined}
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
						onclick={() => askDisconnect(provider.id)}>Disconnect</Button
					>
				{:else}
					<Button
						class="h-11 px-5"
						disabled={busyProvider !== null}
						onclick={() => connect(provider.id)}
					>
						{#if busyProvider === provider.id}<Spinner />{/if}Connect
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
			<Dialog.Title class="text-headline">Edit profile</Dialog.Title>
			<Dialog.Description
				>Your display name and username are visible to everyone.</Dialog.Description
			>
		</Dialog.Header>
		<form class="space-y-4" onsubmit={saveProfile}>
			<div class="space-y-2">
				<Label for="profile-name">Display name</Label>
				<Input
					id="profile-name"
					bind:value={nameInput}
					maxlength={40}
					autocomplete="name"
					placeholder={account.username}
				/>
			</div>
			<div class="space-y-2">
				<Label for="profile-username">Username</Label>
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
					Lowercase letters, numbers and underscores. Your profile address changes too.
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
					onclick={() => (profileOpen = false)}>Cancel</Button
				>
				<Button type="submit" class="h-11 px-5" disabled={saving}>
					{#if saving}<Spinner />{/if}Save
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={lastMethodOpen}>
	<Dialog.Content class="text-center sm:max-w-sm">
		<Dialog.Header class="items-center gap-3 pt-2">
			<TriangleAlertIcon class="size-8" strokeWidth={1.6} />
			<Dialog.Title class="text-center text-headline">Can’t remove this sign-in method</Dialog.Title
			>
			<Dialog.Description class="text-center">
				This is your only sign-in method. Add another one before you continue.
			</Dialog.Description>
		</Dialog.Header>
		<Button variant="outline" class="h-11 w-full" onclick={() => (lastMethodOpen = false)}
			>Close</Button
		>
	</Dialog.Content>
</Dialog.Root>

<AlertDialog.Root open={removing !== null} onOpenChange={(open) => open || (removing = null)}>
	<AlertDialog.Content size="sm">
		{#if removing}
			<AlertDialog.Header>
				<AlertDialog.Title>Disconnect {providerLabel(removing)}?</AlertDialog.Title>
				<AlertDialog.Description>
					You won’t be able to sign in with {providerLabel(removing)}. You can connect it again any
					time.
				</AlertDialog.Description>
			</AlertDialog.Header>
			<AlertDialog.Footer>
				<AlertDialog.Cancel class="h-11" disabled={busyProvider !== null}>Cancel</AlertDialog.Cancel
				>
				<AlertDialog.Action
					variant="destructive"
					class="h-11"
					disabled={busyProvider !== null}
					onclick={disconnect}
				>
					{#if busyProvider}<Spinner />{/if}Disconnect
				</AlertDialog.Action>
			</AlertDialog.Footer>
		{/if}
	</AlertDialog.Content>
</AlertDialog.Root>

<SetPasswordDialog bind:open={setPasswordOpen} email={account.email} />
<ChangePasswordDialog
	bind:open={changePasswordOpen}
	email={account.email}
	onChanged={() => toast.success('Your password was changed')}
/>
