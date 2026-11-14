<script lang="ts">
	import type { Snippet } from 'svelte';

	// The section name is in the site header; this row adds the page's own title (optional),
	// a line of description and the page's main actions.
	let { title, description, actions }: { title?: string; description?: string; actions?: Snippet } =
		$props();
</script>

<div class="flex flex-wrap items-end justify-between gap-3">
	<div class="min-w-0">
		{#if title}<h2 class="text-title-2">{title}</h2>{/if}
		{#if description}
			<p class="text-callout text-muted-foreground {title ? 'mt-1' : ''}">{description}</p>
		{/if}
	</div>
	{#if actions}
		<div class="flex items-center gap-2">{@render actions()}</div>
	{/if}
</div>
