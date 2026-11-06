<script lang="ts">
	import { hideAuthTransition, withAuthTransition } from '#lib/auth-transition.svelte.js';
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

	// Sign-up options, then email, then the emailed code, then the account details.
	let step = $state<'providers' | 'email' | 'verify' | 'details'>('providers');
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

	// There is no account service behind these pages yet; they only collect and validate input.
	function signUpWith() {
		statusMessage = 'Bu kayıt yöntemi henüz kullanılamıyor. E-posta ile devam edebilirsin.';
	}

	function goTo(next: typeof step) {
		statusMessage = '';
		step = next;
	}

	// Placeholder: no e-mail is sent until the account service exists.
	async function requestCode() {
		return true;
	}

	async function submitEmail(event: SubmitEvent) {
		event.preventDefault();
		pending = true;
		statusMessage = '';
		const sent = await requestCode();
		pending = false;
		if (!sent) return;
		code = '';
		startResendCountdown();
		goTo('verify');
	}

	// Counts down the "resend code" wait; a new countdown replaces the old one.
	let countdown: ReturnType<typeof setInterval> | undefined;
	function startResendCountdown() {
		clearInterval(countdown);
		resendIn = 30;
		countdown = setInterval(() => {
			resendIn -= 1;
			if (resendIn <= 0) clearInterval(countdown);
		}, 1000);
	}
	onDestroy(() => clearInterval(countdown));

	async function resendCode() {
		statusMessage = '';
		if (await requestCode()) {
			startResendCountdown();
			statusMessage = 'Yeni kod gönderildi.';
		}
	}

	function submitCode(event: SubmitEvent) {
		event.preventDefault();
		if (code.length < 6) {
			statusMessage = 'E-postana gelen 6 haneli kodu gir.';
			return;
		}
		goTo('details');
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		pending = true;
		statusMessage = '';
		await withAuthTransition(
			{ title: 'Hesabın oluşturuluyor', description: 'Her şeyi hazırlıyoruz, lütfen bekle…' },
			() => new Promise((done) => setTimeout(done, 0))
		);
		pending = false;
		hideAuthTransition();
		statusMessage = 'Hesap sistemi henüz hazır değil. Kayıt yakında burada olacak.';
	}
</script>

<svelte:head>
	<title>Kayıt ol | Playthru</title>
	<meta name="description" content="Ücretsiz bir Playthru hesabı oluştur." />
</svelte:head>

<div lang="tr">
			{#if step === 'providers'}
		<div in:fade={{ duration: 150 }}>
			<h1
				class="mb-6 text-center text-title-1 text-balance text-primary"
			>
				Hesabını oluştur
			</h1>

			<Button
				class="h-11 w-full cursor-pointer rounded-full"
				onclick={() => signUpWith()}><GoogleIcon />Google ile kayıt ol</Button
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
					onclick={() => signUpWith()}><DiscordIcon />Discord ile kayıt ol</Button
				>
			</div>
		</div>
	{:else if step === 'email'}
		<form class="space-y-4" onsubmit={submitEmail} in:fade={{ duration: 150 }}>
			<h1
				class="mb-6 text-center text-title-1 text-balance text-primary"
			>
				E-postanla kayıt ol
			</h1>
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

			<Button type="submit" class="h-11 w-full cursor-pointer rounded-full" disabled={pending}
				>{pending ? 'Kod gönderiliyor…' : 'Kayıt ol'}</Button
			>

			<Button
				variant="outline"
				class="h-11 w-full cursor-pointer rounded-full"
				onclick={() => goTo('providers')}>Geri dön</Button
			>
		</form>
	{:else if step === 'details'}
		<form class="space-y-4" onsubmit={handleSubmit} in:fade={{ duration: 150 }}>
			<div class="mb-8 text-center">
				<h1 class="text-title-1 text-balance text-primary">
					Kaydını tamamla
				</h1>
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
				<Label for="signup-updates" class="text-callout cursor-pointer font-normal"
					>Yeni özellikler ve duyurular hakkında e-posta almak istiyorum</Label
				>
			</div>


			<Button
				type="submit"
				class="h-11 w-full cursor-pointer rounded-full"
				disabled={pending}
				>{pending ? 'Kayıt tamamlanıyor…' : 'Kaydı tamamla'}</Button
			>

			<Button
				variant="outline"
				class="h-11 w-full cursor-pointer rounded-full"
				onclick={() => goTo('verify')}>Geri dön</Button
			>
		</form>
	{:else}
		<form class="space-y-4" onsubmit={submitCode} in:fade={{ duration: 150 }}>
			<div class="text-center">
				<h1 class="text-title-1 text-balance text-primary">
					E-postanı doğrula
				</h1>
				<p class="mt-3 text-callout text-pretty text-muted-foreground">
					<span class="font-medium text-foreground">{email}</span> adresine tek kullanımlık bir güvenlik
					kodu gönderdik. Birkaç dakika içinde gelmezse spam klasörünü de kontrol et. Kodu aşağıya
					gir:
				</p>
			</div>

			<div class="flex justify-center pt-4">
				<InputOTP.Root
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
					type="button"
					variant="link"
					class="h-auto cursor-pointer p-0 font-medium text-foreground disabled:font-normal disabled:text-muted-foreground disabled:no-underline"
					disabled={resendIn > 0}
					onclick={resendCode}>Tekrar gönder{resendIn > 0 ? ` (${resendIn})` : ''}</Button
				>
			</p>

			<Button type="submit" class="h-11 w-full cursor-pointer rounded-full" disabled={pending}
				>E-postayı doğrula</Button
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
		Zaten hesabın var mı?

		<a href={'/giris'} class="font-medium text-foreground hover:underline">Giriş yap</a>
	</p>
</div>
