<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import { Checkbox } from '#lib/components/ui/checkbox/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import PasswordInput from '#lib/components/auth/password-input.svelte';
	import { accountErrorMessage, accountRequest, NEW_PASSWORD_HINT } from '#lib/account.js';

	let {
		open = $bindable(false),
		email,
		onChanged
	}: { open?: boolean; email: string | null; onChanged?: () => void } = $props();

	let currentPassword = $state('');
	let newPassword = $state('');
	let signOutOthers = $state(true);
	let pending = $state(false);
	let error = $state('');

	// The form is cleared whenever the dialog closes, however it was closed.
	function reset() {
		currentPassword = newPassword = error = '';
		signOutOthers = true;
	}

	function close() {
		open = false;
		reset();
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const length = [...newPassword].length;
		if (length < 8 || length > 128) {
			error = NEW_PASSWORD_HINT;
			return;
		}
		pending = true;
		error = '';
		const result = await accountRequest('PUT', '/password', {
			current_password: currentPassword,
			new_password: newPassword,
			sign_out_others: signOutOthers
		});
		pending = false;
		if (!result.ok) {
			error = accountErrorMessage(result.error);
			return;
		}
		close();
		onChanged?.();
	}
</script>

<Dialog.Root bind:open onOpenChange={(value) => value || reset()}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title class="text-headline">Şifreyi değiştir</Dialog.Title>
			<Dialog.Description
				>Mevcut şifreni ve yeni şifreni girerek giriş şifreni güncelle.</Dialog.Description
			>
		</Dialog.Header>
		<form class="space-y-4" onsubmit={submit} novalidate>
			<input type="text" name="username" value={email ?? ''} autocomplete="username" hidden />
			<div class="space-y-2">
				<Label for="change-current-password">Mevcut şifre</Label>
				<PasswordInput id="change-current-password" bind:value={currentPassword} />
			</div>
			<div class="space-y-2">
				<Label for="change-new-password">Yeni şifre</Label>
				<PasswordInput
					id="change-new-password"
					bind:value={newPassword}
					autocomplete="new-password"
				/>
				<p class="px-4 text-footnote text-muted-foreground">En az 8 karakter.</p>
			</div>
			<Label class="flex min-h-11 items-center gap-2.5 font-normal">
				<Checkbox bind:checked={signOutOthers} />Diğer cihazlardaki oturumları kapat
			</Label>
			{#if error}
				<p class="text-footnote text-destructive" role="alert">{error}</p>
			{/if}
			<Dialog.Footer>
				<Button type="button" variant="outline" class="h-11 px-5" onclick={close}>Vazgeç</Button>
				<Button type="submit" class="h-11 px-5" disabled={pending || !currentPassword}>
					{#if pending}<Spinner />{/if}Şifreyi kaydet
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
