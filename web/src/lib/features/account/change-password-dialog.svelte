<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import { Checkbox } from '#lib/components/ui/checkbox/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import PasswordInput from '#lib/features/auth/password-input.svelte';
	import { accountErrorMessage, accountRequest, NEW_PASSWORD_HINT } from './account.js';

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
			<Dialog.Title class="text-headline">Change password</Dialog.Title>
			<Dialog.Description
				>Enter your current password and a new one to update how you sign in.</Dialog.Description
			>
		</Dialog.Header>
		<form class="space-y-4" onsubmit={submit} novalidate>
			<input type="text" name="username" value={email ?? ''} autocomplete="username" hidden />
			<div class="space-y-2">
				<Label for="change-current-password">Current password</Label>
				<PasswordInput id="change-current-password" bind:value={currentPassword} />
			</div>
			<div class="space-y-2">
				<Label for="change-new-password">New password</Label>
				<PasswordInput
					id="change-new-password"
					bind:value={newPassword}
					autocomplete="new-password"
				/>
				<p class="px-4 text-footnote text-muted-foreground">At least 8 characters.</p>
			</div>
			<Label class="flex min-h-11 items-center gap-2.5 font-normal">
				<Checkbox bind:checked={signOutOthers} />Sign out of other devices
			</Label>
			{#if error}
				<p class="text-footnote text-destructive" role="alert">{error}</p>
			{/if}
			<Dialog.Footer>
				<Button type="button" variant="outline" class="h-11 px-5" onclick={close}>Cancel</Button>
				<Button type="submit" class="h-11 px-5" disabled={pending || !currentPassword}>
					{#if pending}<Spinner />{/if}Save password
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
