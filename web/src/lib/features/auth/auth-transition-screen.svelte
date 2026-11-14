<script lang="ts">
	import { fade } from 'svelte/transition';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import { authTransition } from './auth-transition.svelte.js';
</script>

{#if authTransition.current}
	<div
		class="fixed inset-0 z-100 flex flex-col items-center justify-center bg-background px-6 text-center"
		role="status"
		aria-live="polite"
		transition:fade={{ duration: 150 }}
	>
		<Spinner class="size-7 text-muted-foreground" aria-hidden="true" />
		<h1 class="mt-5 text-title-1 text-primary">
			{authTransition.current.title}
		</h1>
		<p class="mt-2 text-footnote text-muted-foreground">{authTransition.current.description}</p>
	</div>
{/if}
