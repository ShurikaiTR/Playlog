<script lang="ts">
	import PasswordInput from '#lib/components/auth/password-input.svelte';
	import SocialButtons from '#lib/components/auth/social-buttons.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Checkbox } from '#lib/components/ui/checkbox/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';

	let username = $state('');
	let email = $state('');
	let password = $state('');
	let wantsUpdates = $state(false);
	let status = $state('');

	// There is no account service behind these pages yet; they only collect and validate input.
	function onProvider(provider: string) {
		status = `${provider} ile kayıt henüz kullanılamıyor.`;
	}

	function onSubmit(event: SubmitEvent) {
		event.preventDefault();
		status = 'Hesap sistemi henüz hazır değil. Kayıt yakında burada olacak.';
	}
</script>

<svelte:head>
	<title>Kayıt ol | Playlog</title>
	<meta name="description" content="Ücretsiz bir Playlog hesabı oluştur." />
</svelte:head>

<div lang="tr">
	<h1 class="mb-8 text-center text-h2 text-balance">Hesabını oluştur</h1>

	<SocialButtons onselect={onProvider} />

	<div class="my-6 flex items-center gap-3" aria-hidden="true">
		<Separator class="flex-1" />
		<span class="text-xs text-muted-foreground">veya</span>
		<Separator class="flex-1" />
	</div>

	<form class="grid gap-4" onsubmit={onSubmit}>
		<div class="grid gap-2">
			<Label for="kayit-kullanici">Kullanıcı adı</Label>
			<Input
				id="kayit-kullanici"
				bind:value={username}
				autocomplete="username"
				minlength={3}
				maxlength={30}
				pattern="[A-Za-z0-9_.\-]+"
				title="Harf, rakam, nokta, tire ve alt çizgi kullanabilirsin."
				class="h-11"
				required
			/>
		</div>

		<div class="grid gap-2">
			<Label for="kayit-eposta">E-posta</Label>
			<Input
				id="kayit-eposta"
				type="email"
				bind:value={email}
				autocomplete="email"
				placeholder="sen@ornek.com"
				class="h-11"
				required
			/>
		</div>

		<div class="grid gap-2">
			<Label for="kayit-sifre">Şifre</Label>
			<PasswordInput
				id="kayit-sifre"
				bind:value={password}
				autocomplete="new-password"
				placeholder="En az 8 karakter"
			/>
		</div>

		<div class="flex items-start gap-2.5">
			<Checkbox id="kayit-duyuru" bind:checked={wantsUpdates} class="mt-0.5" />
			<Label for="kayit-duyuru" class="text-sm leading-5 font-normal text-muted-foreground">
				Yeni özellikler ve duyurular hakkında e-posta almak istiyorum
			</Label>
		</div>

		<Button type="submit" size="lg" class="mt-2 h-11 w-full">Kayıt ol</Button>
	</form>

	{#if status}
		<p class="mt-5 text-center text-xs text-muted-foreground" role="status">{status}</p>
	{/if}

	<p class="mt-8 text-center text-sm text-muted-foreground">
		Zaten hesabın var mı?
		<a href="/giris" class="font-medium text-primary hover:underline">Giriş yap</a>
	</p>
</div>
