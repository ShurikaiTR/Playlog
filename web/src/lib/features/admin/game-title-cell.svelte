<script lang="ts">
	let { slug, title, cover }: { slug: string; title: string; cover: string | null } = $props();
</script>

<a href={`/admin/games/${slug}`} class="group flex min-w-0 items-center gap-3">
	<span class="size-11 shrink-0 overflow-hidden rounded-lg bg-muted">
		{#if cover}
			<img src={cover} alt="" loading="lazy" class="size-full object-cover" />
		{/if}
	</span>
	<span class="min-w-0">
		<span class="block max-w-72 truncate font-medium group-hover:underline">{title}</span>
		<span class="block max-w-72 truncate text-footnote text-muted-foreground">{slug}</span>
	</span>
</a>
