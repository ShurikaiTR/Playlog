<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';
	import type { Named, SeriesEntry } from './games.js';

	let { series, games }: { series: Named; games: SeriesEntry[] } = $props();
</script>

<section aria-labelledby="series-heading" class="w-full px-4 py-8 md:px-8">
	<h2 id="series-heading" class="text-title-2 text-primary">{series.name}</h2>
	<p class="mt-1 mb-4 text-callout text-muted-foreground">Recommended play order</p>
	<ol class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
		{#each games as game, index (game.slug)}
			<li>
				<a
					href={game.current ? undefined : `/games/${game.slug}`}
					aria-current={game.current ? 'page' : undefined}
					class="flex min-h-11 items-center gap-3 rounded-2xl border bg-card p-3 outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 {game.current
						? 'border-primary bg-muted'
						: 'hover:bg-muted'}"
				>
					<span class="w-6 shrink-0 text-center text-callout text-muted-foreground tabular-nums"
						>{index + 1}</span
					>
					{#if game.cover}
						<img
							src={game.cover}
							alt=""
							width="40"
							height="60"
							loading="lazy"
							decoding="async"
							class="h-15 w-10 shrink-0 rounded-lg object-cover"
						/>
					{:else}
						<div class="h-15 w-10 shrink-0 rounded-lg bg-muted" aria-hidden="true"></div>
					{/if}
					<div class="min-w-0 flex-1">
						<p class="truncate text-callout font-medium text-primary">{game.title}</p>
						<p class="truncate text-footnote text-muted-foreground">{game.release}</p>
						{#if game.note}
							<p class="line-clamp-2 text-footnote text-muted-foreground">{game.note}</p>
						{/if}
					</div>
					{#if game.current}<Badge class="shrink-0">You are here</Badge>{/if}
				</a>
			</li>
		{/each}
	</ol>
</section>
