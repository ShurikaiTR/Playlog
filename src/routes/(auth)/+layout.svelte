<script lang="ts">
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
</script>

<div class="flex min-h-svh flex-col bg-background px-4 py-5 md:px-8">
	<header>
		<a
			href="/"
			class="inline-block rounded-full outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
			aria-label="Playthru ana sayfa"
		>
			<img
				src="/logos/playthru.svg"
				alt="Playthru"
				width="282"
				height="56"
				class="h-8 w-auto dark:invert"
			/>
		</a>
	</header>

	<main class="flex flex-1 items-center justify-center py-12">
		<div class="w-full max-w-sm">{@render children()}</div>
	</main>

	<footer class="text-center text-xs leading-5 text-muted-foreground">
		Devam ederek Playlog'un
		<a href="/kullanim-sartlari" class="underline underline-offset-4 hover:text-primary"
			>Kullanım Şartları</a
		>'nı ve
		<a href="/gizlilik-politikasi" class="underline underline-offset-4 hover:text-primary"
			>Gizlilik Politikası</a
		>'nı kabul etmiş olursun.
	</footer>
</div>
