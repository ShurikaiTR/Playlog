<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import PasswordInput from '#lib/components/auth/password-input.svelte';
	import { accountErrorMessage, accountRequest, NEW_PASSWORD_HINT } from '#lib/account.js';

	// Adds a password to an account that only uses social sign-in.
	let {
		open = $bindable(false),
		email,
		onSaved
	}: { open?: boolean; email: string; onSaved?: () => void } = $props();

	let password = $state('');
	let pending = $state(false);
	let error = $state('');

	// The form is cleared whenever the dialog closes, however it was closed.
	function reset() {
		password = error = '';
	}

	function close() {
		open = false;
		reset();
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const length = [...password].length;
		if (length < 8 || length > 128) {
			error = NEW_PASSWORD_HINT;
			return;
		}
		pending = true;
		error = '';
		const result = await accountRequest('PUT', '/password', { new_password: password });
		pending = false;
		if (!result.ok) {
			error = accountErrorMessage(result.error);
			return;
		}
		close();
		onSaved?.();
		await invalidateAll();
	}
</script>

<Dialog.Root bind:open onOpenChange={(value) => value || reset()}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title class="text-headline">Şifre belirle</Dialog.Title>
			<Dialog.Description
				>E-postan ve bu şifreyle de giriş yapabilmek için aşağıdaki formu doldur.</Dialog.Description
			>
		</Dialog.Header>
		<form class="space-y-4" onsubmit={submit} novalidate>
			<!-- Lets password managers pair the new password with the account. -->
			<input type="text" name="username" value={email} autocomplete="username" hidden />
			<div class="space-y-2">
				<Label for="set-new-password">Yeni şifre</Label>
				<PasswordInput id="set-new-password" bind:value={password} autocomplete="new-password" />
				<p class="px-4 text-footnote text-muted-foreground">En az 8 karakter.</p>
			</div>
			{#if error}
				<p class="text-footnote text-destructive" role="alert">{error}</p>
			{/if}
			<Dialog.Footer>
				<Button type="button" variant="outline" class="h-11 px-5" onclick={close}>Vazgeç</Button>
				<Button type="submit" class="h-11 px-5" disabled={pending}>
					{#if pending}<Spinner />{/if}Şifreyi kaydet
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
