<script lang="ts">
	import AppShell from '#lib/components/layout/app-shell.svelte';
	import { hydrateLists } from '#lib/features/lists/user-lists.svelte.js';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();

	// Browser only on purpose: the lists belong to this visitor and must not sit in server memory.
	$effect.pre(() => hydrateLists(data.lists));
</script>

<AppShell>{@render children()}</AppShell>
