<script lang="ts">
	import * as Card from "#lib/components/ui/card/index.js";
	import { Separator } from "#lib/components/ui/separator/index.js";
	import GameSaveMenu, { type GameList } from "#lib/components/game-save-menu.svelte";
	import GameRow from "#lib/components/game-row.svelte";
	import { trendingGames } from "#lib/games.js";
	import CalendarIcon from "@lucide/svelte/icons/calendar";
	import CodeIcon from "@lucide/svelte/icons/code-xml";
	import ShieldIcon from "@lucide/svelte/icons/shield";
	import StarIcon from "@lucide/svelte/icons/star";
	import TagIcon from "@lucide/svelte/icons/tag";
	import BuildingIcon from "@lucide/svelte/icons/building-2";

	let { data } = $props();
	const game = $derived(data.game);

	let status = $state<string>("");
	let favorite = $state(false);
	let lists = $state<GameList[]>([]);

	const dateFormat = new Intl.DateTimeFormat("tr-TR", { day: "numeric", month: "long", year: "numeric" });
	const info = $derived([
		{ label: "Geliştirici", value: game.developer, icon: CodeIcon },
		{ label: "Yayıncı", value: game.publisher, icon: BuildingIcon },
		{ label: "Çıkış tarihi", value: dateFormat.format(new Date(game.releaseDate)), icon: CalendarIcon },
		{ label: "Türler", value: game.genres.join(", "), icon: TagIcon },
		{ label: "Yaş sınırı", value: game.ageRating, icon: ShieldIcon },
	]);
	const similar = $derived(trendingGames.filter((g) => g.slug !== game.slug).slice(0, 6));
</script>

<svelte:head>
	<title>{game.title} | Playlog</title>
	<meta name="description" content={game.description} />
</svelte:head>

<div lang="tr" class="flex min-w-0 flex-col gap-16 pb-12">
	<section aria-labelledby="game-title" class="detail-hero relative isolate min-w-0">
		<!-- Full-bleed: slides under the header and the floating sidebar. -->
		<div class="pointer-events-none absolute inset-0 -z-10 overflow-hidden" aria-hidden="true" style="mask-image: linear-gradient(to bottom, black 55%, transparent 100%)">
			<img
				src={game.backdrop}
				alt=""
				width="1600"
				height="900"
				fetchpriority="high"
				class="size-full object-cover object-[center_45%]"
			/>
			<div class="detail-scrim absolute inset-0"></div>
		</div>

		<div class="detail-intro grid min-w-0 w-full items-start px-4 md:px-8">
			<img
				src={game.cover}
				alt="{game.title} kapağı"
				width="1024"
				height="1536"
				class="detail-poster bg-muted aspect-2/3 w-full rounded-2xl object-cover shadow-[0_20px_50px_-18px_rgb(0_0_0/0.5)] ring-1 ring-white/20"
			/>

			<div class="detail-heading min-w-0">
				<h1 id="game-title" class="detail-title text-[2rem] font-medium tracking-[-0.025em] text-primary md:text-[2.5rem]">
					{game.title}
				</h1>
				<ul class="mt-4 flex flex-wrap gap-2" aria-label="Türler">
					{#each game.genres as genre (genre)}
						<li class="bg-background text-primary rounded-full border px-3 py-1.5 text-[0.8125rem]">{genre}</li>
					{/each}
				</ul>
			</div>

			<div class="detail-actions">
				<GameSaveMenu title={game.title} bind:status bind:favorite bind:lists class="w-full min-w-0" />
			</div>

			<div class="detail-body @container min-w-0">
				<div class="grid w-full max-w-md grid-cols-[1.4fr_1px_1fr_1px_1fr] items-center gap-2">
					<div class="flex h-20 min-w-0 flex-col items-center justify-center gap-2 px-1">
						<span class="flex h-8 items-center gap-2">
							<span class="text-primary text-3xl font-semibold tracking-tight tabular-nums">{game.score.toFixed(1)}</span>
							<StarIcon class="size-4 fill-amber-500 text-amber-500" aria-hidden="true" />
						</span>
						<span class="text-muted-foreground text-[0.65rem] @[28rem]:text-xs">{game.votes.toLocaleString("tr-TR")} oy</span>
					</div>
					<Separator orientation="vertical" class="bg-foreground/15 h-12 self-center" />
					<div class="flex h-20 min-w-0 flex-col items-center justify-center gap-2 px-1">
						<span class="flex h-8 items-center">
							<span class="bg-background text-primary rounded-md border px-2.5 py-1 text-sm font-semibold">{game.ageRating}</span>
						</span>
						<span class="text-muted-foreground text-[0.65rem] @[28rem]:text-xs">Yaş sınırı</span>
					</div>
					<Separator orientation="vertical" class="bg-foreground/15 h-12 self-center" />
					<div class="flex h-20 min-w-0 flex-col items-center justify-center gap-2 px-1">
						<span class="text-primary flex h-8 items-center text-lg font-semibold tabular-nums">{new Date(game.releaseDate).getFullYear()}</span>
						<span class="text-muted-foreground text-[0.65rem] @[28rem]:text-xs">Çıkış yılı</span>
					</div>
				</div>
				<p class="text-muted-foreground mt-6 max-w-2xl text-sm leading-7 whitespace-pre-line">{game.description}</p>
			</div>
		</div>
	</section>

	<section aria-labelledby="info-title" class="w-full min-w-0 px-4 md:px-8">
		<h2 id="info-title" class="text-primary mb-4 text-sm font-semibold tracking-wider uppercase">Oyun bilgileri</h2>
		<Card.Root class="gap-0 px-5 py-2 sm:px-6">
			<dl class="game-info-grid grid gap-x-8 text-sm">
				{#each info as item (item.label)}
					<div class="grid min-h-14 grid-cols-2 items-center gap-4 py-3">
						<dt class="text-muted-foreground grid grid-cols-[1rem_minmax(0,1fr)] items-center gap-2.5">
							<item.icon class="size-4" aria-hidden="true" />
							{item.label}
						</dt>
						<dd class="text-primary text-right font-medium">{item.value}</dd>
					</div>
				{/each}
			</dl>
		</Card.Root>
	</section>

	<GameRow title="Benzer oyunlar" href="/kesfet/populer" games={similar} />
</div>

<style>
	.detail-hero {
		--offset: var(--sidebar-offset, 0px);
		margin-top: -4rem;
		margin-left: calc(-1 * var(--offset));
		padding-left: var(--offset);
	}

	/* Same wash as the home hero: fades into the page on top, left and bottom. */
	.detail-scrim {
		background:
			linear-gradient(0deg, var(--background) 0%, color-mix(in oklch, var(--background) 70%, transparent) 30%, transparent 65%),
			linear-gradient(
				90deg,
				color-mix(in oklch, var(--background) 92%, transparent) 0%,
				color-mix(in oklch, var(--background) 78%, transparent) 45%,
				color-mix(in oklch, var(--background) 45%, transparent) 100%
			),
			linear-gradient(180deg, color-mix(in oklch, var(--background) 55%, transparent) 0%, transparent 22%);
	}

	.detail-intro {
		grid-template-columns: 6.5rem minmax(0, 1fr);
		gap: 1.5rem 1rem;
		padding-top: 9rem;
	}

	.detail-title {
		line-height: 1.05;
		overflow-wrap: anywhere;
	}

	.detail-heading {
		padding-top: 0.5rem;
	}

	.detail-actions,
	.detail-body {
		grid-column: 1 / -1;
	}

	@media (min-width: 640px) {
		.detail-intro {
			grid-template-columns: 10rem minmax(0, 1fr);
			column-gap: 1.75rem;
		}
	}

	@media (min-width: 1024px) {
		.detail-intro {
			grid-template-columns: 13.5rem minmax(0, 1fr);
			gap: 1.5rem 2.5rem;
			padding-top: 8rem;
			padding-bottom: 2rem;
		}

		.detail-poster {
			grid-row: span 2;
		}

		.detail-actions {
			grid-column: 1;
			grid-row: 3;
			grid-template-columns: minmax(0, 1fr);
		}

		.detail-body {
			grid-column: 2;
			grid-row: 2 / 4;
		}
	}

	.game-info-grid {
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 18rem), 1fr));
	}

	.game-info-grid > div {
		border-bottom: 1px solid color-mix(in oklab, var(--border) 70%, transparent);
	}
</style>
