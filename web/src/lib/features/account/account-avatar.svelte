<script lang="ts">
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import { cn } from '#lib/utils.js';

	let {
		name,
		src = null,
		class: className
	}: { name: string; src?: string | null; class?: string } = $props();

	const initial = $derived(name.charAt(0).toLocaleUpperCase('tr'));
</script>

<Avatar.Root class={cn('shrink-0', className)}>
	{#if src}<Avatar.Image {src} alt="" />{/if}
	<Avatar.Fallback class="bg-foreground font-semibold text-background">{initial}</Avatar.Fallback>
</Avatar.Root>
