<script lang="ts">
	let { username, displayName }: { username: string; displayName: string | null } = $props();
</script>

<a href={`/@${username}`} class="group block min-w-0" target="_blank" rel="noopener">
	<span class="block max-w-64 truncate font-medium group-hover:underline"
		>{displayName || username}</span
	>
	<span class="block max-w-64 truncate text-footnote text-muted-foreground">@{username}</span>
</a>
