<script lang="ts">
	import type { Game } from './games.js';

	let { game }: { game: Game } = $props();
</script>

<a
	href="/games/{game.slug}"
	class="group/poster block min-w-0 rounded-2xl outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
>
	<!-- Quiet by default: just the artwork. Hover only lifts it a touch, and reduced motion turns that off. -->
	<div
		class="relative aspect-2/3 overflow-hidden rounded-2xl bg-muted shadow-[0_6px_18px_-10px_rgb(0_0_0/0.5)] ring-1 ring-black/8 transition-[transform,box-shadow] duration-200 ease-out group-hover/poster:scale-[1.02] group-hover/poster:shadow-[0_16px_32px_-14px_rgb(0_0_0/0.6)] group-active/poster:scale-[0.99] motion-reduce:transition-none motion-reduce:group-hover/poster:scale-100 motion-reduce:group-active/poster:scale-100 dark:ring-white/10"
	>
		{#if game.cover}
			<img
				class="size-full object-cover"
				src={game.cover}
				alt=""
				width="600"
				height="900"
				loading="lazy"
				decoding="async"
			/>
		{:else}
			<!-- No artwork yet: the title stands in, so the grid keeps its rhythm. -->
			<div
				class="flex size-full items-end p-3 text-footnote font-medium text-muted-foreground"
				aria-hidden="true"
			>
				<span class="line-clamp-3">{game.title}</span>
			</div>
		{/if}
	</div>

	<div class="px-0.5 pt-2.5">
		<h3 class="truncate text-callout font-medium text-primary">
			{game.title}
		</h3>
		{#if game.meta}<p class="truncate text-footnote text-muted-foreground">{game.meta}</p>{/if}
	</div>
</a>
