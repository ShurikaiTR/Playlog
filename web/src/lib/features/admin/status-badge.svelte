<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';

	let { status }: { status: string } = $props();

	const variants = { published: 'default', draft: 'secondary', hidden: 'outline' } as const;
	const variant = $derived(variants[status as keyof typeof variants] ?? 'outline');
</script>

<Badge {variant} class="capitalize">{status}</Badge>
