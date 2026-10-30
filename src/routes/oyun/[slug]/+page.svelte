<script lang="ts">
	import * as Card from "#lib/components/ui/card/index.js";
	import { Badge } from "#lib/components/ui/badge/index.js";
	import * as Dialog from "#lib/components/ui/dialog/index.js";
	import { cn } from "#lib/utils.js";
	import * as Tooltip from "#lib/components/ui/tooltip/index.js";
	import { Separator } from "#lib/components/ui/separator/index.js";
	import GameSaveMenu from "#lib/components/game-save-menu.svelte";
	import { toListGame } from "#lib/user-lists.svelte.js";
	import * as Carousel from "#lib/components/ui/carousel/index.js";
	import type { CarouselAPI } from "#lib/components/ui/carousel/context.js";
	import GameRow from "#lib/components/game-row.svelte";
	import { trendingGames } from "#lib/games.js";
	import ExternalLinkIcon from "@lucide/svelte/icons/external-link";
	import XIcon from "@lucide/svelte/icons/x";
	import CalendarIcon from "@lucide/svelte/icons/calendar";
	import GamepadIcon from "@lucide/svelte/icons/gamepad-2";
	import MonitorIcon from "@lucide/svelte/icons/monitor";
	import CodeIcon from "@lucide/svelte/icons/code-xml";
	import ShieldIcon from "@lucide/svelte/icons/shield";
	import StarIcon from "@lucide/svelte/icons/star";
	import TagIcon from "@lucide/svelte/icons/tag";
	import BuildingIcon from "@lucide/svelte/icons/building-2";
	import PlayIcon from "@lucide/svelte/icons/play";
	import { Button } from "#lib/components/ui/button/index.js";

	let { data } = $props();
	const game = $derived(data.game);

	let galleryOpen = $state(false);
	let galleryStart = $state(0);
	let galleryApi = $state<CarouselAPI>();
	function openGallery(index: number) {
		galleryStart = index;
		galleryOpen = true;
	}
	// The carousel mounts with the dialog; jump to the clicked screenshot as soon as it is ready.
	let galleryIndex = $state(0);
	let canPrev = $state(false);
	let canNext = $state(false);
	$effect(() => {
		const api = galleryApi;
		if (!galleryOpen || !api) return;
		api.scrollTo(galleryStart, true);
		galleryIndex = galleryStart;
		const onSelect = () => {
			galleryIndex = api.selectedScrollSnap();
			canPrev = api.canScrollPrev();
			canNext = api.canScrollNext();
		};
		onSelect();
		api.on("select", onSelect);
		return () => api.off("select", onSelect);
	});

	const dateFormat = new Intl.DateTimeFormat("tr-TR", { day: "numeric", month: "long", year: "numeric" });
	const info = $derived([
		{ label: "Geliştirici", value: game.developer, icon: CodeIcon },
		{ label: "Yayıncı", value: game.publisher, icon: BuildingIcon },
		{ label: "Çıkış tarihi", value: dateFormat.format(new Date(game.releaseDate)), icon: CalendarIcon },
		{ label: "Platformlar", values: game.platforms, icon: MonitorIcon },
		{ label: "Oyun modları", values: game.modes, icon: GamepadIcon },
		{ label: "Türler", value: game.genres.join(", "), icon: TagIcon },
		{ label: "Yaş sınırı", value: game.ageRating, icon: ShieldIcon },
	]);
	// Placeholder media and review data until games come from real data.
	const screenshots = $derived(
		game.screenshots?.length ? game.screenshots : trendingGames.slice(0, 5).map((g, i) => trendingGames[(i + 1) % trendingGames.length].cover),
	);
	// Spreads the votes over five score bands around the game's score.
	const distribution = $derived.by(() => {
		const weights = [9.5, 7.5, 5.5, 3.5, 1.5].map((mid) => Math.exp(-((mid - game.score) ** 2) / 6));
		const total = weights.reduce((a, b) => a + b, 0);
		return ["9–10", "7–8", "5–6", "3–4", "1–2"].map((range, i) => ({ range, percent: Math.round((weights[i] / total) * 100) }));
	});
	const featuredReview = {
		text: "Atmosferi ve hikâyesiyle uzun zamandır oynadığım en akılda kalıcı oyunlardan biri. Birkaç küçük aksaklığa rağmen bırakmak çok zor.",
		name: "Deniz Aksoy",
		initials: "DA",
		score: 9,
		date: "2 hafta önce",
		helpful: 128,
	};
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
				<GameSaveMenu game={toListGame(game)} class="w-full min-w-0" />
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

	<!-- Two columns from lg up: game info on the left (as wide as the poster above), the right column is left free for more sections. -->
	<div class="grid w-full min-w-0 items-start gap-8 px-4 md:px-8 lg:grid-cols-[13.5rem_minmax(0,1fr)] lg:gap-x-10">
		<section aria-labelledby="info-title" class="min-w-0">
			<h2 id="info-title" class="text-primary mb-4 text-sm font-semibold tracking-wider uppercase">Oyun bilgileri</h2>
			<Card.Root class="gap-0 px-5 py-2 sm:px-6 lg:px-4">
				<dl class="game-info-grid text-sm">
					{#each info as item (item.label)}
						<div class="grid min-h-14 grid-cols-2 items-center gap-4 py-3 lg:grid-cols-1 lg:gap-1.5">
							<dt class="text-muted-foreground grid grid-cols-[1rem_minmax(0,1fr)] items-center gap-2.5">
								<item.icon class="size-4" aria-hidden="true" />
								{item.label}
							</dt>
							{#if item.values}
								<dd class="flex flex-wrap justify-end gap-1.5 lg:justify-start lg:pl-[1.625rem]">
									{#each item.values as value (value)}
										<Badge variant="secondary">{value}</Badge>
									{/each}
								</dd>
							{:else}
								<dd class="text-primary text-right font-medium lg:text-left lg:pl-[1.625rem]">{item.value}</dd>
							{/if}
						</div>
					{/each}
				</dl>
			</Card.Root>
		</section>

		<!-- Two equal columns: the trailer sits alone in its own row (left half), screenshots in one row, and the score and reviews card at the very bottom. -->
		<div class="grid min-w-0 items-start gap-8 xl:grid-cols-2">
			<section aria-labelledby="trailer-title" class="min-w-0 xl:col-span-2">
				<h2 id="trailer-title" class="text-primary mb-3 text-sm font-semibold tracking-wider uppercase">Fragman</h2>
				<div class="min-w-0 xl:w-1/2 xl:pr-4">
					<div class="bg-muted relative aspect-video w-full overflow-hidden rounded-xl ring-1 ring-black/8 dark:ring-white/10">
						<!-- The artwork sits behind the player, so the box is never empty while the video loads or when there is none. -->
						<img src={game.backdrop} alt="" loading="lazy" class="absolute inset-0 size-full object-cover opacity-60" />
						{#if game.trailer}
							<iframe
								title="{game.title} fragmanı"
								src="https://www.youtube-nocookie.com/embed/{game.trailer}"
								loading="lazy"
								allow="accelerometer; encrypted-media; picture-in-picture"
								allowfullscreen
								class="absolute inset-0 size-full"
							></iframe>
						{:else}
							<div class="absolute inset-0 grid place-items-center">
								<div class="flex flex-col items-center gap-3 text-center">
									<span class="bg-background/90 grid size-14 place-items-center rounded-full shadow-sm">
										<PlayIcon class="text-primary size-6 fill-current" aria-hidden="true" />
									</span>
									<p class="bg-background/90 text-primary rounded-full px-3 py-1 text-xs font-medium">Fragman yakında</p>
								</div>
							</div>
						{/if}
					</div>
				</div>
			</section>

			<section aria-labelledby="shots-title" class="min-w-0 xl:col-span-2">
				<h2 id="shots-title" class="text-primary mb-3 text-sm font-semibold tracking-wider uppercase">Ekran görüntüleri</h2>
				<ul class="flex [scrollbar-width:none] gap-3 overflow-x-auto [&::-webkit-scrollbar]:hidden">
					{#each screenshots as shot, i (i)}
						<li class="min-w-40 flex-1">
							<button
								type="button"
								onclick={() => openGallery(i)}
								aria-label="{game.title} ekran görüntüsü {i + 1}, büyüt"
								class="focus-visible:ring-ring/50 block w-full cursor-pointer rounded-xl outline-none focus-visible:ring-[3px]"
							>
								<img
									src={shot}
									alt=""
									loading="lazy"
									class="bg-muted aspect-video w-full rounded-xl object-cover ring-1 ring-black/8 transition-opacity hover:opacity-90 dark:ring-white/10"
								/>
							</button>
						</li>
					{/each}
				</ul>
			</section>

			<section aria-labelledby="reviews-title" class="min-w-0 xl:col-span-2">
				<h2 id="reviews-title" class="text-primary mb-3 text-sm font-semibold tracking-wider uppercase">Puanlar ve incelemeler</h2>
				<Card.Root class="gap-0 py-0">
					<Card.Content class="grid items-center gap-6 py-6 sm:grid-cols-[auto_1px_minmax(0,1fr)] sm:gap-8">
						<div class="flex items-end gap-4">
							<p class="text-primary text-7xl leading-none font-medium tracking-[-0.075em] tabular-nums" aria-label="10 üzerinden {game.score.toFixed(1)}">
								{game.score.toFixed(1)}
							</p>
							<div class="space-y-1 pb-1">
								<span class="flex items-center gap-0.5" aria-hidden="true">
									{#each [1, 2, 3, 4, 5] as star (star)}
										<span class="relative inline-flex size-5">
											<StarIcon class="text-primary size-5" strokeWidth={1.5} />
											<span class="absolute inset-y-0 left-0 overflow-hidden" style:width="{Math.min(Math.max(game.score / 2 - (star - 1), 0), 1) * 100}%">
												<StarIcon class="fill-primary text-primary size-5" strokeWidth={1.5} />
											</span>
										</span>
									{/each}
								</span>
								<p class="text-muted-foreground text-xs">
									<span class="text-primary font-medium tabular-nums">{game.votes.toLocaleString("tr-TR")}</span> oy
								</p>
							</div>
						</div>
						<Separator orientation="vertical" class="hidden self-stretch! sm:block" />
						<div class="space-y-2">
							{#each distribution as group (group.range)}
								<div class="grid grid-cols-[3.25rem_1fr_2.25rem] items-center gap-3 text-xs">
									<span class="text-primary flex items-center gap-1.5 font-medium tabular-nums">
										<StarIcon class="size-3 fill-current" aria-hidden="true" />
										{group.range}
									</span>
									<div class="bg-muted h-1.5 overflow-hidden rounded-full" role="img" aria-label="{group.range} puan: %{group.percent}">
										<div class="bg-primary h-full rounded-full" style:width="{group.percent}%"></div>
									</div>
									<span class="text-muted-foreground text-right tabular-nums">%{group.percent}</span>
								</div>
							{/each}
						</div>
					</Card.Content>

					<Separator />

					<Card.Content class="py-6">
						<figure>
							<p class="text-muted-foreground mb-3 text-xs font-medium">En faydalı inceleme</p>
							<blockquote class="text-primary text-lg leading-8 font-medium tracking-tight text-pretty sm:text-xl">
								{featuredReview.text}
							</blockquote>
							<figcaption class="mt-5 flex flex-wrap items-center gap-x-3 gap-y-2 text-xs">
								<span class="bg-primary text-primary-foreground grid size-7 place-items-center rounded-full text-[0.65rem] font-semibold">{featuredReview.initials}</span>
								<span class="text-primary text-sm font-semibold">{featuredReview.name}</span>
								<span class="bg-secondary text-secondary-foreground inline-flex items-center gap-1 rounded-full px-2 py-0.5 font-medium">
									<StarIcon class="size-3 fill-current" aria-hidden="true" />
									{featuredReview.score}/10
								</span>
								<span class="text-muted-foreground">{featuredReview.date}</span>
								<span class="text-muted-foreground">· {featuredReview.helpful} kişi faydalı buldu</span>
							</figcaption>
						</figure>
					</Card.Content>

					<Card.Footer class="bg-muted/30 flex-wrap gap-2 border-t py-4">
						<Button type="button" class="cursor-pointer rounded-full px-5">İnceleme yaz</Button>
						<Button type="button" variant="outline" class="cursor-pointer rounded-full px-5">Tüm incelemeler</Button>
					</Card.Footer>
				</Card.Root>
			</section>
		</div>
	</div>

	<!-- Preview: the picture on the dimmed page, round arrows at the screen edges and a small bar underneath. Clicking the empty area closes it. -->
	<Dialog.Root bind:open={galleryOpen}>
		<Dialog.Content
			class="top-0 left-0 grid h-svh w-screen max-w-none translate-x-0 translate-y-0 place-items-center rounded-none bg-transparent p-0 shadow-none ring-0 sm:max-w-none"
			overlayClass="bg-black/80 supports-backdrop-filter:backdrop-blur-md"
			showCloseButton={false}
			aria-describedby={undefined}
			onOpenAutoFocus={(event) => event.preventDefault()}
			onclick={(event) => event.target === event.currentTarget && (galleryOpen = false)}
			onkeydown={(event) => {
				// The arrow buttons handle the keys themselves when focused.
				if (event.defaultPrevented) return;
				if (event.key === "ArrowLeft") galleryApi?.scrollPrev();
				else if (event.key === "ArrowRight") galleryApi?.scrollNext();
			}}
		>
			<Dialog.Title class="sr-only">{game.title} ekran görüntüleri</Dialog.Title>
			<Carousel.Root
				setApi={(api) => (galleryApi = api)}
				opts={{ startIndex: galleryStart }}
				class="w-[min(92vw,calc(78svh*16/9))]"
			>
				<Carousel.Content>
					{#each screenshots as shot, i (i)}
						<Carousel.Item>
							<img
								src={shot}
								alt="{game.title} ekran görüntüsü {i + 1} / {screenshots.length}"
								class="bg-muted aspect-video w-full rounded-xl object-cover shadow-2xl"
							/>
						</Carousel.Item>
					{/each}
				</Carousel.Content>
			</Carousel.Root>
			{#each [{ dir: -1, label: "Önceki görsel", can: canPrev, pos: "start-5" }, { dir: 1, label: "Sonraki görsel", can: canNext, pos: "end-5" }] as arrow (arrow.dir)}
				<Button
					variant="ghost"
					size="icon"
					class="{arrow.pos} group/arrow fixed inset-y-0 my-auto size-10 cursor-pointer rounded-full border-0 bg-black/40 text-white backdrop-blur-sm hover:bg-black/60 hover:text-white active:bg-black/70"
					disabled={!arrow.can}
					aria-label={arrow.label}
					onclick={() => (arrow.dir < 0 ? galleryApi?.scrollPrev() : galleryApi?.scrollNext())}
				>
					<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={cn("size-5", arrow.dir < 0 && "rotate-180")} aria-hidden="true">
						<line x1="5" y1="12" x2="19" y2="12" class="origin-right scale-x-0 [transform-box:fill-box] transition-transform duration-150 group-hover/arrow:scale-x-100" />
						<polyline points="11 8 15 12 11 16" class="transition-transform duration-150 group-hover/arrow:translate-x-1" />
					</svg>
				</Button>
			{/each}
			<div class="bg-popover text-popover-foreground ring-foreground/10 fixed bottom-6 left-1/2 flex -translate-x-1/2 items-center gap-1 rounded-xl p-1 pl-3 shadow-lg ring-1">
				<span class="text-muted-foreground pr-2 text-xs tabular-nums">{galleryIndex + 1} / {screenshots.length}</span>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button {...props} variant="ghost" size="icon-sm" class="hover:bg-foreground/10 cursor-pointer rounded-xl" href={screenshots[galleryIndex]} target="_blank" rel="noreferrer" aria-label="Görseli yeni sekmede aç">
								<ExternalLinkIcon />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="top">Yeni sekmede aç</Tooltip.Content>
				</Tooltip.Root>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props: triggerProps })}
							<Dialog.Close>
								{#snippet child({ props })}
									<Button {...props} {...triggerProps} variant="ghost" size="icon-sm" class="hover:bg-foreground/10 cursor-pointer rounded-xl" aria-label="Önizlemeyi kapat"><XIcon /></Button>
								{/snippet}
							</Dialog.Close>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="top">Önizlemeyi kapat</Tooltip.Content>
				</Tooltip.Root>
			</div>
		</Dialog.Content>
	</Dialog.Root>

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

	.game-info-grid > div {
		border-bottom: 1px solid color-mix(in oklab, var(--border) 70%, transparent);
	}

	.game-info-grid > div:last-child {
		border-bottom: 0;
	}
</style>
