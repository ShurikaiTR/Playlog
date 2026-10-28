<script lang="ts">
	import type { Game } from "#lib/games.js";

	let { game }: { game: Game } = $props();
</script>

<a
	href="/oyun/{game.slug}"
	class="group/poster focus-visible:ring-ring/50 block min-w-0 rounded-2xl outline-none focus-visible:ring-[3px]"
>
	<div
		class="bg-muted relative aspect-2/3 overflow-hidden rounded-2xl ring-1 ring-black/8 transition-[transform,box-shadow] duration-300 ease-out group-hover/poster:-translate-y-1 group-hover/poster:shadow-[0_24px_55px_-24px_rgb(0_0_0/0.78)] motion-reduce:transition-none motion-reduce:group-hover/poster:translate-y-0 dark:ring-white/10"
	>
		<img
			class="size-full object-cover transition-transform duration-700 ease-out group-hover/poster:scale-[1.035] motion-reduce:transition-none motion-reduce:group-hover/poster:scale-100"
			src={game.cover}
			alt=""
			width="1024"
			height="1536"
			loading="lazy"
			decoding="async"
		/>
		<div
			class="pointer-events-none absolute inset-0 bg-linear-to-t from-black/65 via-black/0 to-black/15 opacity-70 transition-opacity duration-300 group-hover/poster:opacity-100"
		></div>
	</div>

	<div class="px-0.5 pt-3">
		<h3 class="text-primary truncate text-sm leading-5 font-medium tracking-[-0.01em] md:text-[15px]">
			{game.title}
		</h3>
		{#if game.meta}<p class="text-muted-foreground mt-0.5 truncate text-xs">{game.meta}</p>{/if}
	</div>
</a>
