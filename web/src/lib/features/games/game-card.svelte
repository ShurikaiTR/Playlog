<script lang="ts">
	import type { Game } from "./games.js";

	let { game }: { game: Game } = $props();
</script>

<a
	href="/oyun/{game.slug}"
	class="group/poster focus-visible:ring-ring/50 block min-w-0 rounded-2xl outline-none focus-visible:ring-[3px]"
>
	<!-- Quiet by default: just the artwork. Hover only lifts it a touch, and reduced motion turns that off. -->
	<div
		class="bg-muted relative aspect-2/3 overflow-hidden rounded-2xl ring-1 ring-black/8 shadow-[0_6px_18px_-10px_rgb(0_0_0/0.5)] transition-[transform,box-shadow] duration-200 ease-out group-hover/poster:scale-[1.02] group-hover/poster:shadow-[0_16px_32px_-14px_rgb(0_0_0/0.6)] group-active/poster:scale-[0.99] motion-reduce:transition-none motion-reduce:group-hover/poster:scale-100 motion-reduce:group-active/poster:scale-100 dark:ring-white/10"
	>
		<img
			class="size-full object-cover"
			src={game.cover}
			alt=""
			width="1024"
			height="1536"
			loading="lazy"
			decoding="async"
		/>
	</div>

	<div class="px-0.5 pt-2.5">
		<h3 class="text-primary text-callout truncate font-medium">
			{game.title}
		</h3>
		{#if game.meta}<p class="text-muted-foreground text-footnote truncate">{game.meta}</p>{/if}
	</div>
</a>
