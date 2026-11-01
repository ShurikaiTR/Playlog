<script lang="ts">
	import * as Card from "#lib/components/ui/card/index.js";
	import * as ButtonGroup from "#lib/components/ui/button-group/index.js";
	import * as Alert from "#lib/components/ui/alert/index.js";
	import * as Avatar from "#lib/components/ui/avatar/index.js";
	import { Label } from "#lib/components/ui/label/index.js";
	import { Progress } from "#lib/components/ui/progress/index.js";
	import { Checkbox } from "#lib/components/ui/checkbox/index.js";
	import { ScrollArea } from "#lib/components/ui/scroll-area/index.js";
	import * as Select from "#lib/components/ui/select/index.js";
	import { AspectRatio } from "#lib/components/ui/aspect-ratio/index.js";
	import * as Empty from "#lib/components/ui/empty/index.js";
	import * as Item from "#lib/components/ui/item/index.js";
	import * as Sheet from "#lib/components/ui/sheet/index.js";
	import * as Tabs from "#lib/components/ui/tabs/index.js";
	import { Textarea } from "#lib/components/ui/textarea/index.js";
	import { toast } from "svelte-sonner";
	import { Badge } from "#lib/components/ui/badge/index.js";
	import * as Dialog from "#lib/components/ui/dialog/index.js";
	import { cn } from "#lib/utils.js";
	import * as Tooltip from "#lib/components/ui/tooltip/index.js";
	import * as Table from "#lib/components/ui/table/index.js";
	import { Separator } from "#lib/components/ui/separator/index.js";
	import GameSaveMenu from "#lib/components/game-save-menu.svelte";
	import { gameRating, myReviews, toListGame } from "#lib/user-lists.svelte.js";
	import * as Carousel from "#lib/components/ui/carousel/index.js";
	import type { CarouselAPI } from "#lib/components/ui/carousel/context.js";
	import GameRow from "#lib/components/game-row.svelte";
	import { ageDescription, allGameDetails, trendingGames } from "#lib/games.js";
	import FlagIcon from "@lucide/svelte/icons/flag";
	import ExternalLinkIcon from "@lucide/svelte/icons/external-link";
	import XIcon from "@lucide/svelte/icons/x";
	import ThumbsUpIcon from "@lucide/svelte/icons/thumbs-up";
	import EyeOffIcon from "@lucide/svelte/icons/eye-off";
	import CalendarIcon from "@lucide/svelte/icons/calendar";
	import GamepadIcon from "@lucide/svelte/icons/gamepad-2";
	import MonitorIcon from "@lucide/svelte/icons/monitor";
	import CodeIcon from "@lucide/svelte/icons/code-xml";
	import ShieldIcon from "@lucide/svelte/icons/shield";
	import StarIcon from "@lucide/svelte/icons/star";
	import TagIcon from "@lucide/svelte/icons/tag";
	import CheckIcon from "@lucide/svelte/icons/check";
	import MinusIcon from "@lucide/svelte/icons/minus";
	import Volume2Icon from "@lucide/svelte/icons/volume-2";
	import CaptionsIcon from "@lucide/svelte/icons/captions";
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

	const languageColumns = [
		{ key: "interface", label: "Arayüz", icon: MonitorIcon },
		{ key: "audio", label: "Seslendirme", icon: Volume2Icon },
		{ key: "subtitles", label: "Altyazı", icon: CaptionsIcon },
	] as const;
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
	const requirementRows = [
		{ key: "os", label: "İşletim sistemi" },
		{ key: "cpu", label: "İşlemci" },
		{ key: "gpu", label: "Ekran kartı" },
		{ key: "ram", label: "Bellek" },
		{ key: "storage", label: "Depolama" },
	] as const;
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
	// Placeholder reviews until they come from real data, newest first.
	type Review = { key: string; text: string; name: string; initials: string; score?: number; date: string; helpful: number; spoiler: boolean; mine?: boolean };
	const sampleReviews: Review[] = [
		{ key: "mert", name: "Mert Yıldız", initials: "MY", score: 7, date: "3 hafta önce", helpful: 41, spoiler: false, text: "Görsel tasarım harika ama orta kısımlar biraz uzamış. Yine de müziklerle birlikte keyifli bir deneyim." },
		{ key: "deniz", name: "Deniz Aksoy", initials: "DA", score: 9, date: "2 hafta önce", helpful: 128, spoiler: false, text: "Atmosferi ve hikâyesiyle uzun zamandır oynadığım en akılda kalıcı oyunlardan biri. Birkaç küçük aksaklığa rağmen bırakmak çok zor." },
		{ key: "elif", name: "Elif Karaca", initials: "EK", score: 8, date: "1 ay önce", helpful: 74, spoiler: true, text: "Final bölümündeki seçim beni gerçekten düşündürdü. Hangi sonu seçersen seç, bedelini ödüyorsun ve bu çok iyi işlenmiş." },
		{ key: "zeynep", name: "Zeynep Arslan", initials: "ZA", score: 6, date: "2 ay önce", helpful: 19, spoiler: false, text: "Fikir çok iyi ama kontroller bazen cevap vermiyor. Yama sonrası daha iyi olabilir." },
	];

	const rating = $derived(gameRating[game.slug]);
	const myReview = $derived(myReviews[game.slug]);
	const reviews = $derived<Review[]>([
		...(myReview ? [{ key: "mine", name: "Sen", initials: "S", score: rating, date: "Az önce", helpful: 0, spoiler: myReview.spoiler, text: myReview.text, mine: true }] : []),
		...sampleReviews,
	]);
	let reviewSort = $state("helpful");
	let revealed = $state<Record<string, boolean>>({});
	let voted = $state<Record<string, boolean>>({});
	const votes = (review: Review) => review.helpful + (voted[review.key] ? 1 : 0);
	const sortedReviews = $derived(reviewSort === "helpful" ? [...reviews].sort((a, b) => votes(b) - votes(a)) : reviews);
	// The featured review never contains spoilers, since it shows without a click.
	const featuredReview = $derived([...reviews].filter((review) => !review.spoiler).sort((a, b) => votes(b) - votes(a))[0]);


	function scrollToReviews(event: MouseEvent) {
		if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
		const heading = document.getElementById("reviews-title");
		if (!heading) return;
		event.preventDefault();
		heading.focus({ preventScroll: true });
		heading.scrollIntoView({ behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "instant" : "smooth", block: "start" });
	}

	let reportOpen = $state(false);
	let reportReason = $state("");
	let reportDetails = $state("");
	let reportNotice = $state("");

	function submitReport(event: SubmitEvent) {
		event.preventDefault();
		reportNotice = "Bildirimler şu anda kullanılamıyor. Bildirimin gönderilmedi.";
	}

	let reviewsSheetOpen = $state(false);
	let reviewSheetOpen = $state(false);
	let draftText = $state("");
	let draftSpoiler = $state(false);
	let reviewError = $state("");

	function openReview() {
		draftText = myReview?.text ?? "";
		draftSpoiler = myReview?.spoiler ?? false;
		reviewError = "";
		reviewsSheetOpen = false;
		reviewSheetOpen = true;
	}

	function saveReview(event: SubmitEvent) {
		event.preventDefault();
		if (!rating) {
			reviewError = "Önce 1 ile 10 arasında bir puan seç.";
			return;
		}
		const text = draftText.trim();
		if (text.length < 10) {
			reviewError = "İncelemen en az 10 karakter olsun.";
			return;
		}
		myReviews[game.slug] = { text, spoiler: draftSpoiler };
		reviewSheetOpen = false;
		toast("İncelemen kaydedildi");
	}
	// Where the game is sold, by platform. Search links until real store ids exist.
	const storeLinks = $derived.by(() => {
		const q = encodeURIComponent(game.title);
		const all = [
			{ platform: "PC", name: "Steam", href: `https://store.steampowered.com/search/?term=${q}` },
			{ platform: "PC", name: "Epic Games Store", href: `https://store.epicgames.com/browse?q=${q}` },
			{ platform: "PlayStation", name: "PlayStation Store", href: `https://store.playstation.com/search/${q}` },
			{ platform: "Xbox", name: "Xbox Store", href: `https://www.xbox.com/search?q=${q}` },
			{ platform: "Switch", name: "Nintendo eShop", href: `https://www.nintendo.com/search/?q=${q}` },
			{ platform: "iOS", name: "App Store", href: `https://apps.apple.com/search?term=${q}` },
			{ platform: "Android", name: "Google Play", href: `https://play.google.com/store/search?q=${q}&c=apps` },
		];
		return all.filter((store) => (game.platforms as string[]).includes(store.platform));
	});
	const seriesGames = $derived(
		(game.series ?? []).flatMap((slug) => {
			const other = allGameDetails.find((g) => g.slug === slug);
			return other ? [{ slug: other.slug, title: other.title, cover: other.cover, meta: "Aynı seriden" }] : [];
		}),
	);
	const similar = $derived(trendingGames.filter((g) => g.slug !== game.slug).slice(0, 6));
</script>

{#snippet heading(id: string, title: string, meta?: string)}
	<div class="mb-3 flex items-baseline justify-between gap-3">
		<h2 {id} tabindex="-1" class="text-primary scroll-m-20 text-xl font-semibold tracking-tight outline-none">{title}</h2>
		{#if meta}<span class="text-muted-foreground text-xs tabular-nums">{meta}</span>{/if}
	</div>
{/snippet}

{#snippet reviewCard(review: Review)}
	<Card.Root size="sm">
		<Card.Content>
			<div class="flex items-center gap-3">
				<Avatar.Root class="size-9">
					<Avatar.Fallback class="bg-foreground text-background text-xs font-semibold">{review.initials}</Avatar.Fallback>
				</Avatar.Root>
				<div class="min-w-0 flex-1">
					<p class="text-sm font-semibold">{review.name}</p>
					<p class="text-muted-foreground text-[0.68rem]">{review.date}</p>
				</div>
				{#if review.score}
					<Badge variant="secondary"><StarIcon data-icon="inline-start" class="fill-current" />{review.score}/10</Badge>
				{/if}
			</div>
			{#if review.spoiler && !revealed[review.key]}
				<Alert.Root role="note" class="mt-3">
					<EyeOffIcon aria-hidden="true" />
					<Alert.Title>Bu inceleme spoiler içeriyor</Alert.Title>
					<Alert.Action>
						<Button type="button" variant="outline" size="xs" class="cursor-pointer" onclick={() => (revealed[review.key] = true)}>Göster</Button>
					</Alert.Action>
				</Alert.Root>
			{:else}
				<p class="text-foreground/80 animate-in fade-in mt-3 text-sm leading-6 duration-300">{review.text}</p>
			{/if}
			<div class="mt-3 -ml-2 flex gap-1">
				<Button
					type="button"
					variant="ghost"
					size="sm"
					class="text-muted-foreground hover:text-foreground h-auto cursor-pointer rounded-full px-2.5 py-1 text-xs"
					aria-pressed={!!voted[review.key]}
					onclick={() => (voted[review.key] = !voted[review.key])}
				>
					<ThumbsUpIcon data-icon="inline-start" class={cn("size-3.5", voted[review.key] && "fill-current")} />
					Faydalı · {votes(review)}
				</Button>
				{#if review.spoiler && revealed[review.key]}
					<Button type="button" variant="ghost" size="sm" class="text-muted-foreground hover:text-foreground h-auto cursor-pointer rounded-full px-2.5 py-1 text-xs" onclick={() => (revealed[review.key] = false)}>
						<EyeOffIcon data-icon="inline-start" class="size-3.5" />
						Gizle
					</Button>
				{/if}
			</div>
		</Card.Content>
	</Card.Root>
{/snippet}

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
			<div class="detail-poster">
				<AspectRatio ratio={2 / 3} class="bg-muted overflow-hidden rounded-2xl shadow-[0_20px_50px_-18px_rgb(0_0_0/0.5)] ring-1 ring-white/20">
					<img src={game.cover} alt="{game.title} kapağı" width="1024" height="1536" class="size-full object-cover" />
				</AspectRatio>
			</div>

			<div class="detail-heading min-w-0">
				<h1 id="game-title" class="detail-title text-[2rem] font-medium tracking-[-0.025em] text-primary md:text-[2.5rem]">
					{#if game.logo}
						<img src={game.logo} alt={game.title} class="max-h-28 w-full max-w-md object-contain object-left" />
					{:else}
						{game.title}
					{/if}
				</h1>
				<ul class="mt-4 flex flex-wrap gap-2" aria-label="Türler">
					{#each game.genres as genre (genre)}
						<li><Badge variant="outline" class="bg-background text-primary h-7 px-3 text-[0.8125rem]">{genre}</Badge></li>
					{/each}
				</ul>
			</div>

			<div class="detail-actions">
				<GameSaveMenu game={toListGame(game)} class="w-full min-w-0" />
			</div>

			<div class="detail-body @container min-w-0">
				<ButtonGroup.Root class="w-full max-w-md">
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									href="#reviews-title"
									onclick={scrollToReviews}
									variant="ghost"
									class="h-20 min-w-0 flex-1 cursor-pointer flex-col gap-2 rounded-l-4xl px-1"
									aria-label="{game.votes.toLocaleString('tr-TR')} oya göre 10 üzerinden {game.score.toFixed(1)}. Puanlama bölümüne git"
								>
									<span class="flex h-8 items-center gap-2">
										<span class="text-primary text-3xl font-semibold tracking-tight tabular-nums">{game.score.toFixed(1)}</span>
										<StarIcon class="size-4 fill-amber-500 text-amber-500" aria-hidden="true" />
									</span>
									<span class="text-muted-foreground text-[0.65rem] font-normal @[28rem]:text-xs">{game.votes.toLocaleString("tr-TR")} oy</span>
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>Puan ver ve incelemeleri gör</Tooltip.Content>
					</Tooltip.Root>
					<ButtonGroup.Separator class="bg-foreground/15" />
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button {...props} variant="ghost" class="h-20 min-w-0 flex-1 flex-col gap-2 px-1" aria-label={game.ageRating}>
									<span class="flex h-8 items-center">
										<Badge variant="outline" class="bg-background h-7 rounded-md px-2.5 font-semibold">{game.ageRating}</Badge>
									</span>
									<span class="text-muted-foreground text-[0.65rem] font-normal @[28rem]:text-xs">Yaş sınırı</span>
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content class="w-72 max-w-none items-start gap-3 rounded-xl p-3">
							{@const info = ageDescription(game.ageRating)}
							<span class="bg-background/15 flex size-10 shrink-0 items-center justify-center rounded-lg text-sm font-semibold">{game.ageRating}</span>
							<div class="min-w-0 space-y-0.5 text-left">
								<p class="text-sm font-medium">{info?.title ?? "Yaş sınırı"}</p>
								{#if info}<p class="text-xs leading-snug opacity-70">{info.text}</p>{/if}
							</div>
						</Tooltip.Content>
					</Tooltip.Root>
					<ButtonGroup.Separator class="bg-foreground/15" />
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									variant="ghost"
									class="h-20 min-w-0 flex-1 cursor-pointer flex-col gap-2 px-1"
									onclick={() => {
										reportNotice = "";
										reportOpen = true;
									}}
								>
									<span class="flex h-8 items-center"><FlagIcon class="size-5" /></span>
									<span class="text-muted-foreground text-[0.65rem] font-normal @[28rem]:text-xs">Sorun bildir</span>
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>Oyun bilgilerindeki bir sorunu bildir</Tooltip.Content>
					</Tooltip.Root>
				</ButtonGroup.Root>
				<p class="text-muted-foreground mt-6 max-w-2xl text-sm leading-7 whitespace-pre-line">{game.description}</p>
			</div>
		</div>
	</section>

	<!-- Two columns from lg up: game info on the left (as wide as the poster above), the right column is left free for more sections. -->
	<div class="grid w-full min-w-0 items-start gap-8 px-4 md:px-8 lg:grid-cols-[13.5rem_minmax(0,1fr)] lg:gap-x-10">
		<div class="flex min-w-0 flex-col gap-10">
		<section aria-labelledby="info-title" class="min-w-0">
			{@render heading("info-title", "Oyun bilgileri")}
			<dl class="divide-y">
				{#each info as row (row.label)}
					<div class="py-3 first:pt-0">
						<dt class="text-muted-foreground text-xs">{row.label}</dt>
						<dd class="text-primary mt-0.5 text-sm font-medium">{row.values ? row.values.join(", ") : row.value}</dd>
					</div>
				{/each}
			</dl>
		</section>

		{#if storeLinks.length}
			<section aria-labelledby="stores-title" class="min-w-0">
				{@render heading("stores-title", "Mağazalar")}
				<ul class="divide-y">
					{#each storeLinks as store (store.name)}
						<li>
							<a href={store.href} target="_blank" rel="noreferrer" class="text-primary hover:text-muted-foreground flex items-center justify-between gap-2 py-2.5 text-sm font-medium transition-colors first:pt-0">
								{store.name}
								<ExternalLinkIcon class="text-muted-foreground size-3.5" aria-hidden="true" />
								<span class="sr-only">(yeni sekmede açılır)</span>
							</a>
						</li>
					{/each}
				</ul>
			</section>
		{/if}
		</div>

		<!-- Two equal columns: the trailer sits alone in its own row (left half), screenshots in one row, and the score and reviews card at the very bottom. -->
		<div class="grid min-w-0 items-start gap-x-6 gap-y-10 xl:grid-cols-2">
			<section aria-labelledby="trailer-title" class="min-w-0 xl:col-span-2">
				{@render heading("trailer-title", "Fragman")}
				<div class="min-w-0 xl:w-1/2 xl:pr-3">
					<AspectRatio ratio={16 / 9} class="bg-muted overflow-hidden rounded-2xl ring-1 ring-black/8 dark:ring-white/10">
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
							<Empty.Root class="bg-background/60 absolute inset-0 rounded-none border-0 backdrop-blur-sm">
								<Empty.Header>
									<Empty.Media variant="icon"><PlayIcon class="fill-current" /></Empty.Media>
									<Empty.Title>Fragman yakında</Empty.Title>
								</Empty.Header>
							</Empty.Root>
						{/if}
					</AspectRatio>
				</div>
			</section>

			<section aria-labelledby="shots-title" class="min-w-0 xl:col-span-2">
				{@render heading("shots-title", "Ekran görüntüleri", `${screenshots.length} görsel`)}
				<ScrollArea orientation="horizontal" class="w-full">
					<ul class="flex w-max min-w-full gap-3 pb-3">
						{#each screenshots as shot, i (i)}
							<li class="min-w-40 flex-1">
								<button
									type="button"
									onclick={() => openGallery(i)}
									aria-label="{game.title} ekran görüntüsü {i + 1}, büyüt"
									class="focus-visible:ring-ring/50 block w-full cursor-pointer rounded-2xl outline-none focus-visible:ring-[3px]"
								>
									<AspectRatio ratio={16 / 9} class="bg-muted overflow-hidden rounded-2xl ring-1 ring-black/8 transition-opacity hover:opacity-90 dark:ring-white/10">
										<img src={shot} alt="" loading="lazy" class="size-full object-cover" />
									</AspectRatio>
								</button>
							</li>
						{/each}
					</ul>
				</ScrollArea>
			</section>

			<section aria-label="Diller ve sistem gereksinimleri" class="grid min-w-0 gap-x-12 gap-y-10 sm:grid-cols-2 xl:col-span-2">
				<div class="min-w-0">
					{@render heading("languages-title", "Desteklenen diller", `${game.languages.length} dil`)}
					<Table.Root>
					<Table.Header>
						<Table.Row class="hover:bg-transparent">
							<Table.Head class="text-muted-foreground h-9 pl-0 text-xs font-normal">Dil</Table.Head>
							{#each languageColumns as column (column.key)}
								<Table.Head class="h-9 w-7 px-0 text-center">
									<Tooltip.Root>
										<Tooltip.Trigger>
											{#snippet child({ props })}
												<span {...props} class="text-muted-foreground inline-flex"><column.icon class="size-3.5" aria-hidden="true" /><span class="sr-only">{column.label}</span></span>
											{/snippet}
										</Tooltip.Trigger>
										<Tooltip.Content>{column.label}</Tooltip.Content>
									</Tooltip.Root>
								</Table.Head>
							{/each}
							<Table.Head class="w-2 p-0" aria-hidden="true"></Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each game.languages as language (language.name)}
							<Table.Row>
								<Table.Cell class="py-2 pl-0 text-[0.8125rem] font-medium">{language.name}</Table.Cell>
								{#each languageColumns as column (column.key)}
									<Table.Cell class="px-0 py-2 text-center">
										{#if language[column.key]}
											<CheckIcon class="text-primary mx-auto size-3.5" aria-hidden="true" /><span class="sr-only">{column.label} var</span>
										{:else}
											<MinusIcon class="text-muted-foreground/40 mx-auto size-3.5" aria-hidden="true" /><span class="sr-only">{column.label} yok</span>
										{/if}
									</Table.Cell>
								{/each}
								<Table.Cell class="p-0" aria-hidden="true"></Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
				</div>
				{#if game.requirements}
					{@const requirements = game.requirements}
					<div class="min-w-0">
						{@render heading("requirements-title", "Sistem gereksinimleri")}
						<Table.Root>
							<Table.Header>
								<Table.Row class="hover:bg-transparent">
									<Table.Head class="h-9 pl-0"><span class="sr-only">Bileşen</span></Table.Head>
									<Table.Head class="text-muted-foreground h-9 text-xs font-normal">Minimum</Table.Head>
									<Table.Head class="text-muted-foreground h-9 pr-0 text-xs font-normal">Önerilen</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each requirementRows as row (row.key)}
									<Table.Row>
										<Table.Cell class="text-muted-foreground py-2.5 pl-0 text-xs">{row.label}</Table.Cell>
										<Table.Cell class="py-2.5 text-[0.8125rem] font-medium whitespace-normal">{requirements.minimum[row.key]}</Table.Cell>
										<Table.Cell class="py-2.5 pr-0 text-[0.8125rem] font-medium whitespace-normal">{requirements.recommended[row.key]}</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					</div>
				{/if}
			</section>

			<section aria-labelledby="reviews-title" class="min-w-0 xl:col-span-2">
				{@render heading("reviews-title", "Puanlar ve incelemeler", `${reviews.length} inceleme`)}
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
									<Progress value={group.percent} aria-label="{group.range} puan: %{group.percent}" />
									<span class="text-muted-foreground text-right tabular-nums">%{group.percent}</span>
								</div>
							{/each}
						</div>
					</Card.Content>

					<Separator />

					<Card.Content class="py-6">
						{#if featuredReview}
							<Item.Root variant="muted" class="flex-col items-stretch gap-4 p-5 sm:p-6">
								<p class="text-muted-foreground text-xs font-medium">En faydalı inceleme</p>
								<Item.Content>
									<Item.Description class="text-primary line-clamp-none text-lg leading-8 font-medium tracking-tight text-pretty sm:text-xl">{featuredReview.text}</Item.Description>
								</Item.Content>
								<Item.Header class="justify-start gap-3 text-xs">
									<Avatar.Root class="size-7">
										<Avatar.Fallback class="bg-primary text-primary-foreground text-[0.65rem] font-semibold">{featuredReview.initials}</Avatar.Fallback>
									</Avatar.Root>
									<span class="text-primary text-sm font-semibold">{featuredReview.name}</span>
									{#if featuredReview.score}<Badge variant="secondary"><StarIcon data-icon="inline-start" class="fill-current" />{featuredReview.score}/10</Badge>{/if}
									<span class="text-muted-foreground">{featuredReview.date}</span>
									<span class="text-muted-foreground">· {votes(featuredReview)} kişi faydalı buldu</span>
								</Item.Header>
							</Item.Root>
						{:else}
							<p class="text-muted-foreground text-sm">Henüz inceleme yok. İlk incelemeyi sen yaz.</p>
						{/if}
					</Card.Content>

					<Card.Footer class="bg-muted/30 flex-wrap gap-2 border-t py-4">
						<Button type="button" class="cursor-pointer rounded-full px-5" onclick={openReview}>{myReview ? "İncelemeni düzenle" : "İnceleme yaz"}</Button>
						<Button type="button" variant="outline" class="cursor-pointer rounded-full px-5" onclick={() => (reviewsSheetOpen = true)}>Tüm incelemeler <span class="text-muted-foreground tabular-nums">({reviews.length})</span></Button>
					</Card.Footer>
				</Card.Root>
			</section>
		</div>
	</div>

	<!-- Preview: the picture on the dimmed page, round arrows at the screen edges and a small bar underneath. Clicking the empty area closes it. -->
	<Dialog.Root bind:open={galleryOpen}>
		<Dialog.Content
			class="top-0 left-0 grid h-svh max-h-none w-screen max-w-none translate-x-0 translate-y-0 place-items-center overflow-visible rounded-none bg-transparent p-0 shadow-none ring-0 sm:max-w-none dark:ring-0"
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
							<AspectRatio ratio={16 / 9} class="bg-muted overflow-hidden rounded-xl shadow-2xl">
								<img src={shot} alt="{game.title} ekran görüntüsü {i + 1} / {screenshots.length}" class="size-full object-cover" />
							</AspectRatio>
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

	<!-- All reviews, like the anime page: a side drawer with a sort tab and one card per review. -->
	<Sheet.Root bind:open={reviewsSheetOpen}>
		<Sheet.Content class="w-full gap-0 sm:max-w-md [&_[data-slot=sheet-close]]:cursor-pointer">
			<Sheet.Header class="border-b">
				<Sheet.Title class="text-primary text-lg font-medium tracking-tight">İncelemeler</Sheet.Title>
				<Sheet.Description>{game.title} için {reviews.length} inceleme · Ortalama {game.score.toFixed(1).replace(".", ",")}</Sheet.Description>
				<Tabs.Root bind:value={reviewSort} class="mt-2">
					<Tabs.List class="rounded-xl" aria-label="İncelemeleri sırala">
						<Tabs.Trigger value="helpful" class="cursor-pointer rounded-lg px-3 text-xs">En faydalı</Tabs.Trigger>
						<Tabs.Trigger value="newest" class="cursor-pointer rounded-lg px-3 text-xs">En yeni</Tabs.Trigger>
					</Tabs.List>
				</Tabs.Root>
			</Sheet.Header>
			<ScrollArea class="min-h-0 flex-1">
				<div class="space-y-3 p-4">
					{#each sortedReviews as review (review.key)}
						{@render reviewCard(review)}
					{/each}
				</div>
			</ScrollArea>
			<Sheet.Footer class="border-t">
				<Button type="button" class="cursor-pointer rounded-full" onclick={openReview}>{myReview ? "İncelemeni düzenle" : "İnceleme yaz"}</Button>
			</Sheet.Footer>
		</Sheet.Content>
	</Sheet.Root>

	<Dialog.Root bind:open={reportOpen}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Sorun bildir</Dialog.Title>
				<Dialog.Description>{game.title} bilgilerinde neyin düzeltilmesi gerektiğini bize anlat.</Dialog.Description>
			</Dialog.Header>
			<form class="space-y-4" onsubmit={submitReport}>
				<div class="space-y-2">
					<Label for="report-reason">Neden</Label>
					<Select.Root type="single" bind:value={reportReason}>
						<Select.Trigger id="report-reason" class="h-10 w-full">
							<span data-slot="select-value">{reportReason || "Bir neden seç"}</span>
						</Select.Trigger>
						<Select.Content class="rounded-xl">
							{#each ["Yanlış bilgi", "Yanlış görsel", "Eksik bilgi", "Diğer"] as reason (reason)}
								<Select.Item value={reason} class="rounded-lg">{reason}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
				<div class="space-y-2">
					<Label for="report-details">Ayrıntılar</Label>
					<Textarea id="report-details" bind:value={reportDetails} placeholder="Sorunu anlat…" class="min-h-24 resize-y" />
				</div>
				{#if reportNotice}<p class="text-muted-foreground text-xs leading-5" role="status">{reportNotice}</p>{/if}
				<Dialog.Footer>
					<Button type="button" variant="outline" class="cursor-pointer" onclick={() => (reportOpen = false)}>Vazgeç</Button>
					<Button type="submit" class="cursor-pointer">Bildirimi gönder</Button>
				</Dialog.Footer>
			</form>
		</Dialog.Content>
	</Dialog.Root>

	<Dialog.Root bind:open={reviewSheetOpen}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>{game.title} için puan ver</Dialog.Title>
				<Dialog.Description>Bir puan seç ve düşüncelerini paylaş.</Dialog.Description>
			</Dialog.Header>
			<form class="space-y-5" onsubmit={saveReview}>
				<div class="space-y-2">
					<div class="flex items-baseline justify-between">
						<Label>Puanın</Label>
						{#if rating}<span class="text-xl font-semibold tabular-nums">{rating}<span class="text-muted-foreground text-sm font-normal">/10</span></span>{/if}
					</div>
					<div class="grid grid-cols-5 gap-1.5 sm:grid-cols-10">
						{#each Array.from({ length: 10 }, (_, index) => index + 1) as n (n)}
							<Button
								type="button"
								variant={rating === n ? "default" : "outline"}
								class="aspect-square h-auto w-full cursor-pointer rounded-xl p-0 text-xs font-semibold {rating && n < rating ? 'border-foreground/30 bg-foreground/5' : ''}"
								onclick={() => (gameRating[game.slug] = n)}
								aria-pressed={rating === n}
							>
								{n}
							</Button>
						{/each}
					</div>
				</div>
				<div class="space-y-2">
					<Label for="review">İncelemen</Label>
					<Textarea id="review" bind:value={draftText} placeholder="{game.title} hakkında ne düşünüyorsun?" class="min-h-28 resize-none rounded-xl" />
				</div>
				<div class="flex items-center gap-2">
					<Checkbox id="spoiler" bind:checked={draftSpoiler} />
					<Label for="spoiler" class="text-muted-foreground cursor-pointer text-xs">Spoiler içeriyor</Label>
				</div>
				{#if reviewError}<p class="text-muted-foreground text-xs leading-5" role="status">{reviewError}</p>{/if}
				<Dialog.Footer>
					<Button variant="outline" class="cursor-pointer" onclick={() => (reviewSheetOpen = false)}>Vazgeç</Button>
					<Button type="submit" class="cursor-pointer">İncelemeyi gönder</Button>
				</Dialog.Footer>
			</form>
		</Dialog.Content>
	</Dialog.Root>

	{#if seriesGames.length}
		<GameRow title="Aynı seriden" href="/oyunlar" games={seriesGames} />
	{/if}

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


</style>
