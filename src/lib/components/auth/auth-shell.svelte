<script lang="ts">
	import type { Snippet } from 'svelte';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import { Button } from '#lib/components/ui/button/index.js';
	import AuthTerms from './auth-terms.svelte';

	// The frame every auth step shares, in the modal and on the page: a fixed minimum height so
	// the layout doesn't jump between steps, a header, a body that fills the space (its forms push
	// their fields and their buttons down with mt-auto), and a reserved row for the terms.
	let {
		title,
		subtitle,
		onBack,
		children
	}: {
		title: string;
		subtitle?: Snippet;
		onBack?: () => void;
		children: Snippet;
	} = $props();
</script>

<div class="relative flex min-h-164 flex-col sm:min-h-157">
	{#if onBack}
		<Button
			variant="ghost"
			size="icon-sm"
			class="absolute start-4 top-4 z-10 rounded-xl text-muted-foreground hover:text-foreground"
			aria-label="Go back"
			onclick={onBack}><ArrowLeftIcon /></Button
		>
	{/if}

	<header class="shrink-0 px-5 pt-16 [overflow-wrap:anywhere] sm:px-6">
		<div class="flex flex-col gap-y-1.5 text-center">
			<h1 class="text-title-3 font-semibold tracking-tight text-balance text-primary sm:text-2xl">
				{title}
			</h1>
			{#if subtitle}
				<p class="text-sm text-balance text-muted-foreground">{@render subtitle()}</p>
			{/if}
		</div>
	</header>

	<div class="flex flex-1 flex-col px-5 pt-6 sm:px-6">{@render children()}</div>

	<div class="flex h-18 shrink-0 items-end px-5 pb-5 sm:h-16 sm:px-6 sm:pb-6">
		<AuthTerms />
	</div>
</div>
