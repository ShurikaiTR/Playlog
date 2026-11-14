<script lang="ts">
	import type { Snippet } from 'svelte';

	let { title, description, actions }: { title: string; description?: string; actions?: Snippet } =
		$props();
</script>

<div class="mb-6 flex flex-wrap items-end justify-between gap-3">
	<div class="min-w-0">
		<h1 class="text-large-title">{title}</h1>
		{#if description}
			<p class="mt-1 text-callout text-muted-foreground">{description}</p>
		{/if}
	</div>
	{#if actions}
		<div class="flex items-center gap-2">{@render actions()}</div>
	{/if}
</div>
