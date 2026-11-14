<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';

	let { role }: { role: string } = $props();

	const variants = { admin: 'default', moderator: 'secondary', user: 'outline' } as const;
	const variant = $derived(variants[role as keyof typeof variants] ?? 'outline');
</script>

<Badge {variant} class="capitalize">{role}</Badge>
