<script lang="ts">
	import type { Game } from "#lib/games.js";
	import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
	import GameCard from "#lib/components/game-card.svelte";

	let { title, href, games }: { title: string; href: string; games: Game[] } = $props();
	const headingId = $derived(`row-${href.replace(/\W+/g, "-")}`);
</script>

<section aria-labelledby={headingId} class="w-full px-4 py-8 md:px-8">
	<h2 id={headingId} class="mb-4">
		<a
			href={href}
			class="text-primary focus-visible:ring-ring/50 group/title hover:bg-muted -mx-3 -my-1.5 inline-flex items-center gap-1 rounded-2xl px-3 py-1.5 scroll-m-20 text-2xl font-semibold tracking-tight outline-none focus-visible:ring-[3px]"
		>
			{title}
			<ChevronRightIcon
				class="size-4 transition-transform duration-200 group-hover/title:translate-x-0.5 motion-reduce:transition-none"
				aria-hidden="true"
			/>
		</a>
	</h2>
	<ul class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
		{#each games as game (game.slug)}
			<li><GameCard {game} /></li>
		{/each}
	</ul>
</section>
