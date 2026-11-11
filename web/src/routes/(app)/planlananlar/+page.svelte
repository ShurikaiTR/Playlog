<script lang="ts">
	import UserListPage from "#lib/features/lists/user-list-page.svelte";
	import { listByKind } from "#lib/features/lists/user-lists.svelte.js";
</script>

<UserListPage list={listByKind("play_later")} />
