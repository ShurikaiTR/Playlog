<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as InputOTP from '#lib/components/ui/input-otp/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import PasswordInput from '#lib/components/auth/password-input.svelte';
	import { accountErrorMessage, accountRequest, NEW_PASSWORD_HINT } from '#lib/account.js';

	// Adds email + password sign-in to an account that only uses social sign-in. The email starts as
	// the provider's address; a different one must be confirmed with a code before it is saved.
	let {
		open = $bindable(false),
		email: accountEmail,
		onSaved
	}: { open?: boolean; email: string | null; onSaved?: () => void } = $props();

	let email = $state('');
	let password = $state('');
	let code = $state('');
	let step = $state<'form' | 'code'>('form');
	let pending = $state(false);
	let error = $state('');

	// The form is cleared whenever the dialog closes, however it was closed.
	function reset() {
		email = accountEmail ?? '';
		password = code = error = '';
		step = 'form';
	}
	$effect(() => {
		if (open) email = accountEmail ?? '';
	});

	function close() {
		open = false;
		reset();
	}

	const normalized = $derived(email.trim().toLowerCase());
	const changed = $derived(normalized !== (accountEmail ?? '').toLowerCase());

	async function save() {
		const result = await accountRequest('PUT', '/password', {
			new_password: password,
			email: normalized,
			code: changed ? code : undefined
		});
		pending = false;
		if (!result.ok) {
			error = accountErrorMessage(result.error);
			return;
		}
		close();
		onSaved?.();
		await invalidateAll();
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const length = [...password].length;
		if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(normalized)) {
			error = accountErrorMessage('invalid_email');
			return;
		}
		if (length < 8 || length > 128) {
			error = NEW_PASSWORD_HINT;
			return;
		}
		pending = true;
		error = '';
		if (!changed) return save();
		const sent = await accountRequest('POST', '/email-code', { email: normalized });
		pending = false;
		if (!sent.ok) {
			error = accountErrorMessage(sent.error);
			return;
		}
		step = 'code';
	}

	async function verify(event: SubmitEvent) {
		event.preventDefault();
		if (code.length !== 6) return;
		pending = true;
		error = '';
		await save();
	}
</script>

<Dialog.Root bind:open onOpenChange={(value) => value || reset()}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title class="text-headline">E-posta ve şifre ile giriş</Dialog.Title>
			<Dialog.Description>
				{#if step === 'form'}
					Giriş için kullanacağın e-postayı ve bir şifre belirle.
				{:else}
					<strong>{normalized}</strong> adresine 6 haneli bir kod gönderdik.
				{/if}
			</Dialog.Description>
		</Dialog.Header>
		{#if step === 'form'}
			<form class="space-y-4" onsubmit={submit} novalidate>
				<div class="space-y-2">
					<Label for="set-email">E-posta</Label>
					<Input
						id="set-email"
						type="email"
						name="username"
						bind:value={email}
						autocomplete="username"
						class="h-11"
					/>
					{#if changed}
						<p class="px-4 text-footnote text-muted-foreground">
							Bu adrese bir doğrulama kodu göndereceğiz.
						</p>
					{/if}
				</div>
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
						{#if pending}<Spinner />{/if}{changed ? 'Kod gönder' : 'Kaydet'}
					</Button>
				</Dialog.Footer>
			</form>
		{:else}
			<form class="space-y-4" onsubmit={verify} novalidate>
				<InputOTP.Root
					maxlength={6}
					bind:value={code}
					pattern="^[0-9]*$"
					pasteTransformer={(text) => text.replace(/\D/g, '')}
					class="justify-center gap-1 sm:gap-2"
					autofocus
				>
					{#snippet children({ cells })}
						<InputOTP.Group class="gap-1 sm:gap-2">
							{#each cells.slice(0, 3) as cell (cell)}
								<InputOTP.Slot {cell} class="h-11 w-9 rounded-xl text-base sm:h-12 sm:w-11" />
							{/each}
						</InputOTP.Group>
						<InputOTP.Separator />
						<InputOTP.Group class="gap-1 sm:gap-2">
							{#each cells.slice(3, 6) as cell (cell)}
								<InputOTP.Slot {cell} class="h-11 w-9 rounded-xl text-base sm:h-12 sm:w-11" />
							{/each}
						</InputOTP.Group>
					{/snippet}
				</InputOTP.Root>
				{#if error}
					<p class="text-center text-footnote text-destructive" role="alert">{error}</p>
				{/if}
				<Dialog.Footer>
					<Button
						type="button"
						variant="outline"
						class="h-11 px-5"
						onclick={() => {
							step = 'form';
							code = error = '';
						}}>Geri</Button
					>
					<Button type="submit" class="h-11 px-5" disabled={pending || code.length !== 6}>
						{#if pending}<Spinner />{/if}Doğrula ve kaydet
					</Button>
				</Dialog.Footer>
			</form>
		{/if}
	</Dialog.Content>
</Dialog.Root>
