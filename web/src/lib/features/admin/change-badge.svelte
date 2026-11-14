<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { cn } from '#lib/utils.js';

	// How much the last 30 days changed against the 30 before. With nothing before, any growth
	// reads as "New" rather than an infinite percentage.
	let { added, before }: { added: number; before: number } = $props();

	const percent = $derived(before > 0 ? Math.round(((added - before) / before) * 100) : null);
	const tone = $derived(added > before ? 'up' : added < before ? 'down' : 'flat');
	const text = $derived(
		percent === null ? (added > 0 ? `+${added} new` : '0%') : `${percent > 0 ? '+' : ''}${percent}%`
	);
</script>

<Badge
	variant="secondary"
	class={cn(
		'font-normal tabular-nums',
		tone === 'up' && 'bg-success/10 text-success',
		tone === 'down' && 'bg-destructive/10 text-destructive',
		tone === 'flat' && 'text-muted-foreground'
	)}
	title="Last 30 days against the 30 days before"
>
	{text}
</Badge>
