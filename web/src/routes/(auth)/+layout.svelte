<script lang="ts">
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
</script>

<div class="flex min-h-svh flex-col bg-background px-4 py-5 md:px-8">
	<header>
		<a
			href="/"
			class="inline-block rounded-full outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
			aria-label="Playthru home"
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
		<div class="w-full max-w-100">{@render children()}</div>
	</main>
</div>
