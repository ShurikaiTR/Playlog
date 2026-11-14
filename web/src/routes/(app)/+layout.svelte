<script lang="ts">
	import { untrack } from 'svelte';
	import AppShell from '#lib/components/layout/app-shell.svelte';
	import { hydrateLists } from '#lib/features/lists/user-lists.svelte.js';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();

	// Browser only on purpose: the lists belong to this visitor and must not sit in server memory.
	$effect.pre(() => {
		const lists = data.lists;
		// hydrateLists reads and rewrites the store; only the incoming data may re-run this.
		untrack(() => hydrateLists(lists));
	});
</script>

<AppShell>{@render children()}</AppShell>
