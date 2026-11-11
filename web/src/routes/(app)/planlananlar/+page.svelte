<script lang="ts">
	import UserListPage from "#lib/components/user-list-page.svelte";
	import { listByKind } from "#lib/user-lists.svelte.js";
</script>

<UserListPage list={listByKind("play_later")} />
