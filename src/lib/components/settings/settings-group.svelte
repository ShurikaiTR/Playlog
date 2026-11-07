<script lang="ts">
	import type { Snippet } from 'svelte';
	import * as Item from '#lib/components/ui/item/index.js';

	// A grouped inset list, as in System Settings: a rounded card of rows, with an optional
	// small heading above it and an explanation below it.
	let {
		title,
		footer,
		children
	}: { title?: string; footer?: string | Snippet; children: Snippet } = $props();
</script>

<section class="space-y-1.5">
	{#if title}
		<h3 class="px-4 text-footnote font-normal text-muted-foreground">{title}</h3>
	{/if}
	<Item.Group
		class="gap-0 overflow-hidden rounded-2xl bg-muted/50 *:not-first:rounded-none *:not-first:border-t *:not-first:border-t-border/60 *:first:rounded-none"
	>
		{@render children()}
	</Item.Group>
	{#if footer}
		<div class="px-4 text-footnote text-muted-foreground">
			{#if typeof footer === 'string'}{footer}{:else}{@render footer()}{/if}
		</div>
	{/if}
</section>
