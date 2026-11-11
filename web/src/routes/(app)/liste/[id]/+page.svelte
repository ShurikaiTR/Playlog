<script lang="ts">
	import { page } from "$app/state";
	import UserListPage from "#lib/features/lists/user-list-page.svelte";
	import { findList } from "#lib/features/lists/user-lists.svelte.js";

	const list = $derived(findList(page.params.id ?? ""));
</script>

{#if list}
	<UserListPage {list} />
{:else}
	<p class="text-muted-foreground px-4 pt-24 text-sm md:px-8">Liste bulunamadı.</p>
{/if}
