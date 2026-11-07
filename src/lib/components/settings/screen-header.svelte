<script lang="ts">
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';

	// Title of the screen on show. A screen opened from another one gets a back button labelled
	// with where it goes, as in System Settings.
	let {
		title,
		back,
		description
	}: {
		title: string;
		back?: { label: string; onclick: () => void };
		description?: string;
	} = $props();
</script>

<div class="mb-6">
	{#if back}
		<Button
			variant="ghost"
			class="-ms-3 mb-2 h-11 cursor-pointer gap-0.5 rounded-xl ps-1.5 pe-3 text-callout text-primary"
			onclick={back.onclick}
		>
			<ChevronLeftIcon class="size-5" aria-hidden="true" />{back.label}
		</Button>
	{/if}
	<Dialog.Title class="text-title-3">{title}</Dialog.Title>
	<Dialog.Description class={description ? 'mt-1 text-callout text-muted-foreground' : 'sr-only'}
		>{description ?? title}</Dialog.Description
	>
</div>
