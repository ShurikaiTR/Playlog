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

	<!-- Anchored near the top, not centred: steps and messages change the form height, and a
	     centred form would move every field each time. -->
	<main class="flex flex-1 justify-center pt-[clamp(3rem,14vh,8rem)] pb-12">
		<div class="w-full max-w-sm">{@render children()}</div>
	</main>

	<footer class="text-center text-footnote text-muted-foreground">
		Devam ederek Playthru'nun
		<a href="/kullanim-sartlari" class="underline underline-offset-4 hover:text-primary"
			>Kullanım Şartları</a
		>'nı ve
		<a href="/gizlilik-politikasi" class="underline underline-offset-4 hover:text-primary"
			>Gizlilik Politikası</a
		>'nı kabul etmiş olursun.
	</footer>
</div>
