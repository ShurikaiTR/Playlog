<script lang="ts">
	import GoogleIcon from '#lib/components/icons/google.svelte';
	import DiscordIcon from '#lib/components/icons/discord.svelte';
	import MailIcon from '#lib/components/icons/mail.svelte';
	import { hideAuthTransition, withAuthTransition } from '#lib/auth-transition.svelte.js';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as InputGroup from '#lib/components/ui/input-group/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';

	// The page moves from the provider list to an email step, then a password step.
	let step = $state<'providers' | 'email' | 'password'>('providers');
	let showPassword = $state(false);
	let statusMessage = $state('');
	let email = $state('');
	let password = $state('');
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

	// There is no account service behind these pages yet; they only collect and validate input.
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

	function showPasswordResetMessage() {
		statusMessage = 'Şifre sıfırlama henüz kullanılamıyor.';
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		pending = true;
		statusMessage = '';
		await withAuthTransition(
			{ title: 'Giriş yapılıyor', description: 'Hesabın doğrulanıyor, lütfen bekle…' },
			() => new Promise((done) => setTimeout(done, 0))
		);
		pending = false;
		hideAuthTransition();
		statusMessage = 'Hesap sistemi henüz hazır değil. Giriş yakında burada olacak.';
	}
</script>

<svelte:head>
	<title>Giriş yap | Playthru</title>
	<meta name="description" content="Playthru hesabına giriş yap." />
</svelte:head>

<div lang="tr">
			{#if step === 'providers'}
		<div in:fade={{ duration: 150 }}>
			<h1
				class="mb-6 text-center text-title-1 text-balance text-primary"
			>
				Hesabına giriş yap
			</h1>

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
			<h1
				class="mb-6 text-center text-title-1 text-balance text-primary"
			>
				E-postanla giriş yap
			</h1>
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
	{:else}
		<form class="space-y-4" onsubmit={handleSubmit} in:fade={{ duration: 150 }}>
			<h1
				class="mb-6 text-center text-title-1 text-balance text-primary"
			>
				E-postanla giriş yap
			</h1>
			<div class="space-y-2">
				<Label for="login-email">E-posta</Label>
				<Input
					id="login-email"
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
						type="button"
						variant="link"
						class="h-auto cursor-pointer p-0 text-footnote font-normal text-muted-foreground hover:text-foreground"
						onclick={showPasswordResetMessage}>Şifreni mi unuttun?</Button
					>
				</div>
				<InputGroup.Root class="h-12 rounded-full pl-2">
					<InputGroup.Input
						id="login-password"
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


			<Button
				type="submit"
				class="h-11 w-full cursor-pointer rounded-full"
				disabled={pending}
				>{pending ? 'Giriş yapılıyor…' : 'Giriş yap'}</Button
			>

			<Button
				variant="outline"
				class="h-11 w-full cursor-pointer rounded-full"
				onclick={() => goTo('email')}>Geri dön</Button
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

		<a href="/kayit-ol" class="font-medium text-foreground hover:underline"
			>Hesap oluştur</a
		>
	</p>
</div>
