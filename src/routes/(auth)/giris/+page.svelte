<script lang="ts">
	import PasswordInput from '#lib/components/auth/password-input.svelte';
	import SocialButtons from '#lib/components/auth/social-buttons.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';

	let email = $state('');
	let password = $state('');
	let status = $state('');

	// There is no account service behind these pages yet; they only collect and validate input.
	function onProvider(provider: string) {
		status = `${provider} ile giriş henüz kullanılamıyor.`;
	}

	function onSubmit(event: SubmitEvent) {
		event.preventDefault();
		status = 'Hesap sistemi henüz hazır değil. Giriş yakında burada olacak.';
	}
</script>

<svelte:head>
	<title>Giriş yap | Playlog</title>
	<meta name="description" content="Playlog hesabına giriş yap." />
</svelte:head>

<div lang="tr">
	<h1 class="mb-8 text-center text-h2 text-balance">Tekrar hoş geldin</h1>

	<SocialButtons onselect={onProvider} />

	<div class="my-6 flex items-center gap-3" aria-hidden="true">
		<Separator class="flex-1" />
		<span class="text-xs text-muted-foreground">veya</span>
		<Separator class="flex-1" />
	</div>

	<form class="grid gap-4" onsubmit={onSubmit}>
		<div class="grid gap-2">
			<Label for="giris-eposta">E-posta</Label>
			<Input
				id="giris-eposta"
				type="email"
				bind:value={email}
				autocomplete="email"
				placeholder="sen@ornek.com"
				class="h-11"
				required
			/>
		</div>

		<div class="grid gap-2">
			<div class="flex items-center justify-between">
				<Label for="giris-sifre">Şifre</Label>
				<button
					type="button"
					class="text-xs text-muted-foreground underline-offset-4 hover:text-primary hover:underline"
					onclick={() => (status = 'Şifre sıfırlama henüz kullanılamıyor.')}
				>
					Şifreni mi unuttun?
				</button>
				>
			</div>
			<PasswordInput id="giris-sifre" bind:value={password} />
		</div>

		<Button type="submit" size="lg" class="mt-2 h-11 w-full">Giriş yap</Button>
	</form>

	{#if status}
		<p class="mt-5 text-center text-xs text-muted-foreground" role="status">{status}</p>
	{/if}

	<p class="mt-8 text-center text-sm text-muted-foreground">
		Hesabın yok mu?
		<a href="/kayit-ol" class="font-medium text-primary hover:underline">Kayıt ol</a>
	</p>
</div>
