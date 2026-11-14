<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	let {
		slug,
		published,
		canDelete,
		onDelete
	}: { slug: string; published: boolean; canDelete: boolean; onDelete: () => void } = $props();
</script>

<div class="flex items-center justify-end gap-1">
	<Button
		href={`/admin/games/${slug}`}
		variant="ghost"
		size="icon-sm"
		class="pointer-coarse:size-11"
		aria-label="Edit game"><PencilIcon /></Button
	>
	{#if published}
		<Button
			href={`/games/${slug}`}
			variant="ghost"
			size="icon-sm"
			class="pointer-coarse:size-11"
			aria-label="View on Playthru"
			target="_blank"
			rel="noopener"><ExternalLinkIcon /></Button
		>
	{/if}
	{#if canDelete}
		<Button
			variant="ghost"
			size="icon-sm"
			class="text-destructive hover:text-destructive pointer-coarse:size-11"
			aria-label="Delete game"
			onclick={onDelete}><Trash2Icon /></Button
		>
	{/if}
</div>
