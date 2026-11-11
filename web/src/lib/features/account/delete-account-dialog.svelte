<script lang="ts">
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import { showAuthTransition } from '#lib/features/auth/auth-transition.svelte.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import PasswordInput from '#lib/features/auth/password-input.svelte';
	import { accountErrorMessage, accountRequest } from './account.js';

	// Password accounts confirm with their password; social-only accounts need a recent sign-in.
	let { open = $bindable(false), hasPassword }: { open?: boolean; hasPassword: boolean } = $props();

	let password = $state('');
	let deleting = $state(false);
	let error = $state('');

	function reset() {
		password = error = '';
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		deleting = true;
		error = '';
		const result = await accountRequest('DELETE', '', hasPassword ? { password } : {});
		if (!result.ok) {
			error = accountErrorMessage(result.error);
			deleting = false;
			return;
		}
		open = false;
		showAuthTransition({ title: 'Hesabın silindi', description: 'Görüşmek üzere…' });
		// A full load: the session is gone, so every page must render signed out.
		window.location.assign('/');
	}
</script>

<Dialog.Root bind:open onOpenChange={(value) => value || reset()}>
	<Dialog.Content class="sm:max-w-sm">
		<Dialog.Header class="items-center gap-3 pt-2 text-center">
			<TriangleAlertIcon class="size-8 text-destructive" strokeWidth={1.6} />
			<Dialog.Title class="text-center text-headline">Hesabın silinsin mi?</Dialog.Title>
			<Dialog.Description class="text-center">
				Profilin, bağlı hesapların ve oturumların kalıcı olarak silinir. Bu işlem geri alınamaz.
			</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4" onsubmit={submit}>
			{#if hasPassword}
				<div class="space-y-2">
					<Label for="delete-password">Onaylamak için şifreni gir</Label>
					<PasswordInput id="delete-password" bind:value={password} />
				</div>
			{/if}
			{#if error}
				<p class="text-center text-footnote text-destructive" role="alert">{error}</p>
			{/if}
			<div class="grid gap-2">
				<Button
					type="submit"
					variant="destructive"
					class="h-11 w-full"
					disabled={deleting || (hasPassword && !password)}
					>{deleting ? 'Siliniyor…' : 'Hesabımı kalıcı olarak sil'}</Button
				>
				<Button
					type="button"
					variant="outline"
					class="h-11 w-full"
					onclick={() => {
						open = false;
						reset();
					}}>Vazgeç</Button
				>
			</div>
		</form>
	</Dialog.Content>
</Dialog.Root>
