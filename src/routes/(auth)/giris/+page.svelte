<script lang="ts">
	import GoogleIcon from '#lib/components/icons/google.svelte';
	import DiscordIcon from '#lib/components/icons/discord.svelte';
	import MailIcon from '#lib/components/icons/mail.svelte';
	import { hideAuthTransition, showAuthTransition } from '#lib/auth-transition.svelte.js';
	import { applyAction, enhance, type SubmitFunction } from '$app/forms';
	import { onDestroy, onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as InputGroup from '#lib/components/ui/input-group/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as InputOTP from '#lib/components/ui/input-otp/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';

	// The page moves from the provider list to an email step, then a password step.
	// "Şifreni mi unuttun?" mails a code and opens the reset step.
	let step = $state<'providers' | 'email' | 'password' | 'reset'>('providers');
	let showPassword = $state(false);
	let statusMessage = $state('');
	let email = $state('');
	let password = $state('');
	let resetCode = $state('');
	let newPassword = $state('');
	let resendIn = $state(0);
	let pending = $state(false);

	type ProviderId = 'google' | 'discord' | 'email';
	const providers: { id: ProviderId; label: string; icon: typeof GoogleIcon }[] = [
		{ id: 'google', label: 'Google', icon: GoogleIcon },
		{ id: 'discord', label: 'Discord', icon: DiscordIcon },
		{ id: 'email', label: 'E-posta', icon: MailIcon }
	];
	const LAST_PROVIDER_KEY = 'playthru-last-login-provider';
	let lastProvider = $state<ProviderId | null>(null);
	const lastProviderLabel = $derived(
		providers.find((provider) => provider.id === lastProvider)?.label
	);

	onMount(() => {
		try {
			const saved = localStorage.getItem(LAST_PROVIDER_KEY);
			if (providers.some((provider) => provider.id === saved)) lastProvider = saved as ProviderId;
		} catch {
			// Storage can be unavailable (private mode); the hint is optional.
		}
	});

	function rememberProvider(id: ProviderId) {
		lastProvider = id;
		try {
			localStorage.setItem(LAST_PROVIDER_KEY, id);
		} catch {
			// Ignore: the hint just won't persist.
		}
	}

	// Google and Discord sign-in are not on the API yet.
	async function chooseProvider(id: ProviderId) {
		if (id === 'email') {
			goTo('email');
			return;
		}
		statusMessage = 'Bu giriş yöntemi henüz kullanılamıyor. E-posta ile devam edebilirsin.';
	}

	function goTo(next: typeof step) {
		statusMessage = '';
		step = next;
	}

	function submitEmail(event: SubmitEvent) {
		event.preventDefault();
		goTo('password');
	}

	function showEmailCodeMessage() {
		statusMessage = 'E-postayla giriş kodu henüz kullanılamıyor. Şifrenle giriş yapabilirsin.';
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

	// Signs in from the password step.
	const submitLogin: SubmitFunction = () => {
		pending = true;
		statusMessage = '';
		showAuthTransition({
			title: 'Giriş yapılıyor',
			description: 'Hesabın doğrulanıyor, lütfen bekle…'
		});
		return async ({ result }) => {
			pending = false;
			if (result.type === 'redirect') {
				rememberProvider('email');
				await applyAction(result);
				hideAuthTransition();
				return;
			}
			hideAuthTransition();
			statusMessage = failure(result.type === 'failure' ? result.data : undefined);
		};
	};

	// "Şifreni mi unuttun?" and "Tekrar gönder" both mail a reset code through the hidden form
	// below, so pressing Enter in a step's own form never triggers them.
	const submitResetStart: SubmitFunction = ({ cancel }) => {
		if (!email.trim()) {
			statusMessage = 'Önce e-posta adresini gir.';
			cancel();
			return;
		}
		pending = true;
		statusMessage = '';
		return async ({ result }) => {
			pending = false;
			if (result.type !== 'success') {
				statusMessage = failure(result.type === 'failure' ? result.data : undefined);
				return;
			}
			startResendCountdown();
			if (step === 'reset') {
				statusMessage = 'Yeni kod gönderildi.';
			} else {
				resetCode = '';
				newPassword = '';
				goTo('reset');
			}
		};
	};

	// Sets the new password from the reset step.
	const submitReset: SubmitFunction = ({ cancel }) => {
		if (resetCode.length < 6) {
			statusMessage = 'E-postana gelen 6 haneli kodu gir.';
			cancel();
			return;
		}
		pending = true;
		statusMessage = '';
		return async ({ result }) => {
			pending = false;
			if (result.type === 'success') {
				password = '';
				goTo('password');
				statusMessage = 'Şifren değişti. Yeni şifrenle giriş yapabilirsin.';
			} else {
				const data = result.type === 'failure' ? result.data : undefined;
				if (data?.code === 'invalid_code') resetCode = '';
				statusMessage = failure(data);
			}
		};
	};
</script>

<svelte:head>
	<title>Giriş yap | Playthru</title>
	<meta name="description" content="Playthru hesabına giriş yap." />
</svelte:head>

<div lang="tr">
	<form id="reset-start" method="POST" action="?/resetStart" hidden use:enhance={submitResetStart}>
		<input type="hidden" name="email" value={email} />
	</form>

	{#if step === 'providers'}
		<div in:fade={{ duration: 150 }}>
			<h1 class="mb-6 text-center text-title-1 text-balance text-primary">Hesabına giriş yap</h1>

			<div class="space-y-3">
				{#each providers as provider (provider.id)}
					<div class="relative">
						{#if lastProvider === provider.id}
							<!-- Fading ring that marks the provider used last time. -->
							<div
								class="pointer-events-none absolute -inset-1.5 rounded-full border border-amber-500"
								style="mask-image: radial-gradient(circle, transparent 90%, black 100%)"
								aria-hidden="true"
							></div>
						{/if}
						<Button
							variant="outline"
							class="h-11 w-full cursor-pointer rounded-full"
							onclick={() => chooseProvider(provider.id)}
							><provider.icon />{provider.label} ile devam et</Button
						>
					</div>
				{/each}
			</div>
			{#if lastProviderLabel}
				<p
					class="mt-4 flex items-center justify-center gap-2 text-footnote text-muted-foreground"
					aria-live="polite"
				>
					<span
						class="pointer-events-none size-3 rounded-full border border-amber-500"
						style="mask-image: linear-gradient(transparent 0%, black 50%, transparent 100%)"
						aria-hidden="true"
					></span>
					En son {lastProviderLabel} ile giriş yaptın
				</p>
			{/if}
		</div>
	{:else if step === 'email'}
		<form class="space-y-4" onsubmit={submitEmail} in:fade={{ duration: 150 }}>
			<h1 class="mb-6 text-center text-title-1 text-balance text-primary">E-postanla giriş yap</h1>
			<div class="space-y-2">
				<Label for="login-email">E-posta</Label>
				<Input
					id="login-email"
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
	{:else if step === 'password'}
		<form
			method="POST"
			action="?/login"
			class="space-y-4"
			use:enhance={submitLogin}
			in:fade={{ duration: 150 }}
		>
			<h1 class="mb-6 text-center text-title-1 text-balance text-primary">E-postanla giriş yap</h1>
			<div class="space-y-2">
				<Label for="login-email">E-posta</Label>
				<Input
					id="login-email"
					name="email"
					type="email"
					bind:value={email}
					autocomplete="email"
					class="h-12 rounded-full px-5"
					required
				/>
			</div>
			<Button
				variant="outline"
				class="h-11 w-full cursor-pointer rounded-full"
				onclick={showEmailCodeMessage}><MailIcon />E-postama giriş kodu gönder</Button
			>

			<div class="flex items-center gap-3 py-3" aria-hidden="true">
				<Separator class="flex-1" />
				<span class="text-footnote text-muted-foreground">veya</span>
				<Separator class="flex-1" />
			</div>

			<div class="space-y-2">
				<div class="flex items-center justify-between">
					<Label for="login-password">Şifre</Label>
					<Button
						type="submit"
						form="reset-start"
						variant="link"
						class="h-auto cursor-pointer p-0 text-footnote font-normal text-muted-foreground hover:text-foreground"
						disabled={pending}>Şifreni mi unuttun?</Button
					>
				</div>
				<InputGroup.Root class="h-12 rounded-full pl-2">
					<InputGroup.Input
						id="login-password"
						name="password"
						type={showPassword ? 'text' : 'password'}
						bind:value={password}
						autocomplete="current-password"
						required
						autofocus
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

			<Button type="submit" class="h-11 w-full cursor-pointer rounded-full" disabled={pending}
				>{pending ? 'Giriş yapılıyor…' : 'Giriş yap'}</Button
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
			action="?/resetConfirm"
			class="space-y-4"
			use:enhance={submitReset}
			in:fade={{ duration: 150 }}
		>
			<div class="text-center">
				<h1 class="text-title-1 text-balance text-primary">Şifreni sıfırla</h1>
				<p class="mt-3 text-callout text-pretty text-muted-foreground">
					<span class="font-medium text-foreground">{email}</span> bir hesaba bağlıysa, bu adrese tek
					kullanımlık bir kod gönderdik. Kodu ve yeni şifreni gir:
				</p>
			</div>
			<input type="hidden" name="email" value={email} />

			<div class="flex justify-center pt-4">
				<InputOTP.Root
					name="code"
					maxlength={6}
					bind:value={resetCode}
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
					form="reset-start"
					variant="link"
					class="h-auto cursor-pointer p-0 font-medium text-foreground disabled:font-normal disabled:text-muted-foreground disabled:no-underline"
					disabled={resendIn > 0 || pending}
					>Tekrar gönder{resendIn > 0 ? ` (${resendIn})` : ''}</Button
				>
			</p>

			<div class="space-y-2">
				<Label for="reset-password">Yeni şifre</Label>
				<InputGroup.Root class="h-12 rounded-full pl-2">
					<InputGroup.Input
						id="reset-password"
						name="password"
						type={showPassword ? 'text' : 'password'}
						bind:value={newPassword}
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

			<Button type="submit" class="h-11 w-full cursor-pointer rounded-full" disabled={pending}
				>{pending ? 'Kaydediliyor…' : 'Şifreyi değiştir'}</Button
			>

			<Button
				variant="outline"
				class="h-11 w-full cursor-pointer rounded-full"
				onclick={() => goTo('password')}>Geri dön</Button
			>
		</form>
	{/if}

	{#if statusMessage}
		<p class="mt-5 text-center text-footnote text-muted-foreground" aria-live="polite">
			{statusMessage}
		</p>
	{/if}

	<p class="mt-8 text-center text-footnote text-muted-foreground">
		Hesabın yok mu?

		<a href="/kayit-ol" class="font-medium text-foreground hover:underline">Hesap oluştur</a>
	</p>
</div>
