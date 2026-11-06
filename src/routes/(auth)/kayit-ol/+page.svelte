<script lang="ts">
	import { hideAuthTransition, showAuthTransition } from '#lib/auth-transition.svelte.js';
	import { applyAction, enhance, type SubmitFunction } from '$app/forms';
	import { onDestroy } from 'svelte';
	import { fade } from 'svelte/transition';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as InputGroup from '#lib/components/ui/input-group/index.js';
	import { Checkbox } from '#lib/components/ui/checkbox/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as InputOTP from '#lib/components/ui/input-otp/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';
	import DiscordIcon from '#lib/components/icons/discord.svelte';
	import GoogleIcon from '#lib/components/icons/google.svelte';
	import MailIcon from '#lib/components/icons/mail.svelte';

	// Sign-up options, then email, then the account details (which mails the code), then the code.
	// The right code creates the account and signs in.
	let step = $state<'providers' | 'email' | 'details' | 'verify'>('providers');
	let code = $state('');
	let resendIn = $state(0);
	let showPassword = $state(false);
	let statusMessage = $state('');
	let displayName = $state('');
	let username = $state('');
	let wantsUpdates = $state(false);
	let email = $state('');
	let password = $state('');
	let pending = $state(false);

	// Google and Discord share the sign-in route; a new account then picks a username.
	function signUpWith(provider: 'google' | 'discord') {
		window.location.assign(`/auth/${provider}/start`);
	}

	function goTo(next: typeof step) {
		statusMessage = '';
		step = next;
	}

	function submitEmail(event: SubmitEvent) {
		event.preventDefault();
		goTo('details');
	}

	// Counts down the "resend code" wait; the API allows a new code once a minute.
	let countdown: ReturnType<typeof setInterval> | undefined;
	function startResendCountdown() {
		clearInterval(countdown);
		resendIn = 60;
		countdown = setInterval(() => {
			resendIn -= 1;
			if (resendIn <= 0) clearInterval(countdown);
		}, 1000);
	}
	onDestroy(() => clearInterval(countdown));

	const failure = (data: Record<string, unknown> | undefined) =>
		typeof data?.error === 'string' ? data.error : 'Bir şeyler ters gitti. Lütfen tekrar dene.';

	// Errors that belong to the email step send the visitor back there.
	const EMAIL_ERRORS = new Set(['invalid_email', 'email_taken']);

	// Details step and "Tekrar gönder": checks the details and mails the code.
	const submitStart: SubmitFunction = () => {
		pending = true;
		statusMessage = '';
		return async ({ result }) => {
			pending = false;
			if (result.type === 'success') {
				startResendCountdown();
				if (step === 'verify') {
					statusMessage = 'Yeni kod gönderildi.';
				} else {
					code = '';
					goTo('verify');
				}
				return;
			}
			const data = result.type === 'failure' ? result.data : undefined;
			if (typeof data?.code === 'string' && EMAIL_ERRORS.has(data.code)) step = 'email';
			else if (step === 'verify' && data?.code !== 'code_cooldown') step = 'details';
			statusMessage = failure(data);
		};
	};

	// Verify step: the right code creates the account.
	const submitVerify: SubmitFunction = ({ cancel }) => {
		if (code.length < 6) {
			statusMessage = 'E-postana gelen 6 haneli kodu gir.';
			cancel();
			return;
		}
		pending = true;
		statusMessage = '';
		showAuthTransition({
			title: 'Hesabın oluşturuluyor',
			description: 'Her şeyi hazırlıyoruz, lütfen bekle…'
		});
		return async ({ result }) => {
			pending = false;
			if (result.type === 'redirect') {
				await applyAction(result);
				hideAuthTransition();
				return;
			}
			hideAuthTransition();
			const data = result.type === 'failure' ? result.data : undefined;
			if (data?.code === 'invalid_code') code = '';
			else if (data?.code === 'username_taken') step = 'details';
			else if (data?.code === 'email_taken') step = 'email';
			statusMessage = failure(data);
		};
	};
</script>

<svelte:head>
	<title>Kayıt ol | Playthru</title>
	<meta name="description" content="Ücretsiz bir Playthru hesabı oluştur." />
</svelte:head>

<div lang="tr">
	<!-- "Tekrar gönder" posts the same details again; kept outside the code form so Enter there verifies. -->
	<form id="signup-resend" method="POST" action="?/start" hidden use:enhance={submitStart}>
		<input type="hidden" name="email" value={email} />
		<input type="hidden" name="username" value={username} />
		<input type="hidden" name="password" value={password} />
		<input type="hidden" name="display_name" value={displayName} />
		<input type="hidden" name="marketing_opt_in" value={wantsUpdates ? 'true' : ''} />
	</form>

	{#if step === 'providers'}
		<div in:fade={{ duration: 150 }}>
			<h1 class="mb-6 text-center text-title-1 text-balance text-primary">Hesabını oluştur</h1>

			<Button class="h-11 w-full cursor-pointer rounded-full" onclick={() => signUpWith('google')}
				><GoogleIcon />Google ile kayıt ol</Button
			>

			<Separator class="mx-auto my-6 w-3/4!" />

			<div class="space-y-3">
				<Button
					variant="outline"
					class="h-11 w-full cursor-pointer rounded-full"
					onclick={() => goTo('email')}><MailIcon />E-posta ile kayıt ol</Button
				>
				<Button
					variant="outline"
					class="h-11 w-full cursor-pointer rounded-full"
					onclick={() => signUpWith('discord')}><DiscordIcon />Discord ile kayıt ol</Button
				>
			</div>
		</div>
	{:else if step === 'email'}
		<form class="space-y-4" onsubmit={submitEmail} in:fade={{ duration: 150 }}>
			<h1 class="mb-6 text-center text-title-1 text-balance text-primary">E-postanla kayıt ol</h1>
			<div class="space-y-2">
				<Label for="signup-email">E-posta</Label>
				<Input
					id="signup-email"
					type="email"
					bind:value={email}
					autocomplete="email"
					placeholder="sen@ornek.com"
					class="h-12 rounded-full px-5"
					required
					autofocus
				/>
			</div>

			<Button type="submit" class="h-11 w-full cursor-pointer rounded-full">İleri</Button>

			<Button
				variant="outline"
				class="h-11 w-full cursor-pointer rounded-full"
				onclick={() => goTo('providers')}>Geri dön</Button
			>
		</form>
	{:else if step === 'details'}
		<form
			method="POST"
			action="?/start"
			class="space-y-4"
			use:enhance={submitStart}
			in:fade={{ duration: 150 }}
		>
			<input type="hidden" name="email" value={email} />
			<input type="hidden" name="marketing_opt_in" value={wantsUpdates ? 'true' : ''} />
			<div class="mb-8 text-center">
				<h1 class="text-title-1 text-balance text-primary">Kaydını tamamla</h1>
				<span
					class="mt-4 inline-flex max-w-full items-center gap-2 rounded-full border px-4 py-1.5 text-footnote"
				>
					<MailIcon class="size-4 shrink-0" />
					<span class="truncate">{email}</span>
				</span>
			</div>
			<div class="grid grid-cols-2 gap-3">
				<div class="space-y-2">
					<Label for="signup-name">Görünen ad</Label>
					<Input
						id="signup-name"
						name="display_name"
						bind:value={displayName}
						maxlength={40}
						autocomplete="nickname"
						class="h-12 rounded-full px-5"
						autofocus
					/>
				</div>
				<div class="space-y-2">
					<Label for="signup-username">Kullanıcı adı</Label>
					<Input
						id="signup-username"
						name="username"
						bind:value={username}
						minlength={3}
						maxlength={30}
						autocomplete="username"
						class="h-12 rounded-full px-5"
						required
					/>
				</div>
			</div>
			<div class="space-y-2">
				<Label for="signup-password">Şifre</Label>
				<InputGroup.Root class="h-12 rounded-full pl-2">
					<InputGroup.Input
						id="signup-password"
						name="password"
						type={showPassword ? 'text' : 'password'}
						bind:value={password}
						autocomplete="new-password"
						placeholder="En az 8 karakter"
						minlength={8}
						required
					/>
					<InputGroup.Addon align="inline-end">
						<InputGroup.Button
							size="icon-xs"
							class="cursor-pointer rounded-full"
							aria-label={showPassword ? 'Şifreyi gizle' : 'Şifreyi göster'}
							onclick={() => (showPassword = !showPassword)}
						>
							{#if showPassword}
								<EyeOffIcon />
							{:else}
								<EyeIcon />
							{/if}
						</InputGroup.Button>
					</InputGroup.Addon>
				</InputGroup.Root>
			</div>
			<div class="flex items-center gap-2.5 py-1">
				<Checkbox id="signup-updates" bind:checked={wantsUpdates} />
				<Label for="signup-updates" class="cursor-pointer text-callout font-normal"
					>Yeni özellikler ve duyurular hakkında e-posta almak istiyorum</Label
				>
			</div>

			<Button type="submit" class="h-11 w-full cursor-pointer rounded-full" disabled={pending}
				>{pending ? 'Kod gönderiliyor…' : 'Devam et'}</Button
			>

			<Button
				variant="outline"
				class="h-11 w-full cursor-pointer rounded-full"
				onclick={() => goTo('email')}>Geri dön</Button
			>
		</form>
	{:else}
		<form
			method="POST"
			action="?/verify"
			class="space-y-4"
			use:enhance={submitVerify}
			in:fade={{ duration: 150 }}
		>
			<input type="hidden" name="email" value={email} />
			<div class="text-center">
				<h1 class="text-title-1 text-balance text-primary">E-postanı doğrula</h1>
				<p class="mt-3 text-callout text-pretty text-muted-foreground">
					<span class="font-medium text-foreground">{email}</span> adresine tek kullanımlık bir güvenlik
					kodu gönderdik. Birkaç dakika içinde gelmezse spam klasörünü de kontrol et. Kodu aşağıya gir:
				</p>
			</div>

			<div class="flex justify-center pt-4">
				<InputOTP.Root
					name="code"
					maxlength={6}
					bind:value={code}
					pattern="^[0-9]*$"
					pasteTransformer={(text) => text.replace(/\D/g, '')}
					autofocus
				>
					{#snippet children({ cells })}
						<InputOTP.Group>
							{#each cells.slice(0, 3) as cell (cell)}
								<InputOTP.Slot {cell} class="size-11 text-base" />
							{/each}
						</InputOTP.Group>
						<InputOTP.Separator />
						<InputOTP.Group>
							{#each cells.slice(3, 6) as cell (cell)}
								<InputOTP.Slot {cell} class="size-11 text-base" />
							{/each}
						</InputOTP.Group>
					{/snippet}
				</InputOTP.Root>
			</div>

			<p class="text-center text-footnote text-muted-foreground">
				Kod gelmedi mi?
				<Button
					type="submit"
					form="signup-resend"
					variant="link"
					class="h-auto cursor-pointer p-0 font-medium text-foreground disabled:font-normal disabled:text-muted-foreground disabled:no-underline"
					disabled={resendIn > 0 || pending}
					>Tekrar gönder{resendIn > 0 ? ` (${resendIn})` : ''}</Button
				>
			</p>

			<Button type="submit" class="h-11 w-full cursor-pointer rounded-full" disabled={pending}
				>E-postayı doğrula</Button
			>

			<Button
				variant="outline"
				class="h-11 w-full cursor-pointer rounded-full"
				onclick={() => goTo('details')}>Geri dön</Button
			>
		</form>
	{/if}

	{#if statusMessage}
		<p class="mt-5 text-center text-footnote text-muted-foreground" aria-live="polite">
			{statusMessage}
		</p>
	{/if}

	<p class="mt-8 text-center text-footnote text-muted-foreground">
		Zaten hesabın var mı?

		<a href={'/giris'} class="font-medium text-foreground hover:underline">Giriş yap</a>
	</p>
</div>
