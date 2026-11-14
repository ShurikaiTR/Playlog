<script lang="ts">
	import { page } from '$app/state';
	import UserListPage from '#lib/features/lists/user-list-page.svelte';
	import { findList } from '#lib/features/lists/user-lists.svelte.js';

	const list = $derived(findList(page.params.id ?? ''));
</script>

{#if list}
	<UserListPage {list} />
{:else}
	<p class="px-4 pt-24 text-sm text-muted-foreground md:px-8">List not found.</p>
{/if}
