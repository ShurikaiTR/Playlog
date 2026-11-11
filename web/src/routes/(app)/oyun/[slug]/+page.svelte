<script lang="ts">
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import * as Card from '#lib/components/ui/card/index.js';
	import * as Alert from '#lib/components/ui/alert/index.js';
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Progress } from '#lib/components/ui/progress/index.js';
	import { Switch } from '#lib/components/ui/switch/index.js';
	import { ScrollArea } from '#lib/components/ui/scroll-area/index.js';
	import { AspectRatio } from '#lib/components/ui/aspect-ratio/index.js';
	import * as Empty from '#lib/components/ui/empty/index.js';
	import * as ToggleGroup from '#lib/components/ui/toggle-group/index.js';
	import * as Item from '#lib/components/ui/item/index.js';
	import * as Sheet from '#lib/components/ui/sheet/index.js';
	import * as Collapsible from '#lib/components/ui/collapsible/index.js';
	import * as Tabs from '#lib/components/ui/tabs/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import { toast } from 'svelte-sonner';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { cn } from '#lib/utils.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import * as Table from '#lib/components/ui/table/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';
	import GameSaveMenu from '#lib/features/lists/game-save-menu.svelte';
	import { gameRating, myReviews, toListGame } from '#lib/features/lists/user-lists.svelte.js';
	import GameGalleryDialog from '#lib/features/games/game-gallery-dialog.svelte';
	import GameReportDialog from '#lib/features/games/game-report-dialog.svelte';
	import GameRow from '#lib/features/games/game-row.svelte';
	import { ageDescription, allGameDetails, trendingGames } from '#lib/features/games/games.js';
	import FlagIcon from '@lucide/svelte/icons/flag';
	import ThumbsUpIcon from '@lucide/svelte/icons/thumbs-up';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import GamepadIcon from '@lucide/svelte/icons/gamepad-2';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import CodeIcon from '@lucide/svelte/icons/code-xml';
	import UsersRoundIcon from '@lucide/svelte/icons/users-round';
	import StarIcon from '@lucide/svelte/icons/star';
	import TagIcon from '@lucide/svelte/icons/tag';
	import CheckIcon from '@lucide/svelte/icons/check';
	import MinusIcon from '@lucide/svelte/icons/minus';
	import Volume2Icon from '@lucide/svelte/icons/volume-2';
	import CaptionsIcon from '@lucide/svelte/icons/captions';
	import BuildingIcon from '@lucide/svelte/icons/building-2';
	import CpuIcon from '@lucide/svelte/icons/cpu';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import PlayIcon from '@lucide/svelte/icons/play';
	import { Button } from '#lib/components/ui/button/index.js';

	let { data } = $props();
	const game = $derived(data.game);

	let galleryOpen = $state(false);
	let galleryStart = $state(0);
	function openGallery(index: number) {
		galleryStart = index;
		galleryOpen = true;
	}

	const languageColumns = [
		{ key: 'interface', label: 'Arayüz', icon: MonitorIcon },
		{ key: 'audio', label: 'Seslendirme', icon: Volume2Icon },
		{ key: 'subtitles', label: 'Altyazı', icon: CaptionsIcon }
	] as const;
	const dateFormat = new Intl.DateTimeFormat('tr-TR', {
		day: 'numeric',
		month: 'long',
		year: 'numeric'
	});
	const info = $derived([
		{ label: 'Geliştirici', value: game.developer, icon: CodeIcon },
		{ label: 'Yayıncı', value: game.publisher, icon: BuildingIcon },
		{
			label: 'Çıkış tarihi',
			value: dateFormat.format(new Date(game.releaseDate)),
			icon: CalendarIcon
		},
		{ label: 'Platformlar', values: game.platforms, icon: MonitorIcon },
		{ label: 'Oyun modları', values: game.modes, icon: GamepadIcon },
		{ label: 'Türler', value: game.genres.join(', '), icon: TagIcon },
		{ label: 'Yaş sınırı', value: game.ageRating, icon: UsersRoundIcon }
	]);
	const requirementRows = [
		{ key: 'os', label: 'İşletim sistemi' },
		{ key: 'cpu', label: 'İşlemci' },
		{ key: 'gpu', label: 'Ekran kartı' },
		{ key: 'ram', label: 'Bellek' },
		{ key: 'storage', label: 'Depolama' }
	] as const;
	// Placeholder media and review data until games come from real data.
	const screenshots = $derived(
		game.screenshots?.length
			? game.screenshots
			: trendingGames.slice(0, 5).map((g, i) => trendingGames[(i + 1) % trendingGames.length].cover)
	);
	// The gallery shows the trailer first (when there is one), then the screenshots.
	const trailerCount = $derived(game.trailer ? 1 : 0);
	const galleryItems = $derived<{ kind: 'video' | 'image'; src: string }[]>([
		...(game.trailer ? [{ kind: 'video' as const, src: game.trailer }] : []),
		...screenshots.map((src) => ({ kind: 'image' as const, src }))
	]);
	// Spreads the votes over five score bands around the game's score.
	const distribution = $derived.by(() => {
		const weights = [9.5, 7.5, 5.5, 3.5, 1.5].map((mid) =>
			Math.exp(-((mid - game.score) ** 2) / 6)
		);
		const total = weights.reduce((a, b) => a + b, 0);
		return ['9–10', '7–8', '5–6', '3–4', '1–2'].map((range, i) => ({
			range,
			percent: Math.round((weights[i] / total) * 100)
		}));
	});
	// Placeholder reviews until they come from real data, newest first.
	type Review = {
		key: string;
		text: string;
		name: string;
		initials: string;
		score?: number;
		date: string;
		helpful: number;
		spoiler: boolean;
		mine?: boolean;
	};
	const sampleReviews: Review[] = [
		{
			key: 'mert',
			name: 'Mert Yıldız',
			initials: 'MY',
			score: 7,
			date: '3 hafta önce',
			helpful: 41,
			spoiler: false,
			text: 'Görsel tasarım harika ama orta kısımlar biraz uzamış. Yine de müziklerle birlikte keyifli bir deneyim.'
		},
		{
			key: 'deniz',
			name: 'Deniz Aksoy',
			initials: 'DA',
			score: 9,
			date: '2 hafta önce',
			helpful: 128,
			spoiler: false,
			text: 'Atmosferi ve hikâyesiyle uzun zamandır oynadığım en akılda kalıcı oyunlardan biri. Birkaç küçük aksaklığa rağmen bırakmak çok zor.'
		},
		{
			key: 'elif',
			name: 'Elif Karaca',
			initials: 'EK',
			score: 8,
			date: '1 ay önce',
			helpful: 74,
			spoiler: true,
			text: 'Final bölümündeki seçim beni gerçekten düşündürdü. Hangi sonu seçersen seç, bedelini ödüyorsun ve bu çok iyi işlenmiş.'
		},
		{
			key: 'zeynep',
			name: 'Zeynep Arslan',
			initials: 'ZA',
			score: 6,
			date: '2 ay önce',
			helpful: 19,
			spoiler: false,
			text: 'Fikir çok iyi ama kontroller bazen cevap vermiyor. Yama sonrası daha iyi olabilir.'
		}
	];

	const rating = $derived(gameRating[game.slug]);
	const scoreWords = [
		'Berbat',
		'Çok kötü',
		'Kötü',
		'Zayıf',
		'Vasat',
		'Fena değil',
		'İyi',
		'Çok iyi',
		'Harika',
		'Başyapıt'
	];
	const myReview = $derived(myReviews[game.slug]);
	const reviews = $derived<Review[]>([
		...(myReview
			? [
					{
						key: 'mine',
						name: 'Sen',
						initials: 'S',
						score: rating,
						date: 'Az önce',
						helpful: 0,
						spoiler: myReview.spoiler,
						text: myReview.text,
						mine: true
					}
				]
			: []),
		...sampleReviews
	]);
	let reviewSort = $state('helpful');
	let revealed = $state<Record<string, boolean>>({});
	let voted = $state<Record<string, boolean>>({});
	const votes = (review: Review) => review.helpful + (voted[review.key] ? 1 : 0);
	const sortedReviews = $derived(
		reviewSort === 'helpful' ? [...reviews].sort((a, b) => votes(b) - votes(a)) : reviews
	);
	// The featured review never contains spoilers, since it shows without a click.
	const featuredReview = $derived(
		[...reviews].filter((review) => !review.spoiler).sort((a, b) => votes(b) - votes(a))[0]
	);

	function scrollToSection(event: MouseEvent, id: string) {
		if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey)
			return;
		const heading = document.getElementById(id);
		if (!heading) return;
		event.preventDefault();
		heading.focus({ preventScroll: true });
		heading.scrollIntoView({
			behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth',
			block: 'start'
		});
	}

	let languagesOpen = $state(false);
	let requirementsOpen = $state(false);
	let reportOpen = $state(false);

	let reviewsSheetOpen = $state(false);
	let reviewSheetOpen = $state(false);
	let draftText = $state('');
	let draftSpoiler = $state(false);
	let reviewError = $state('');

	// Unsent writing survives closing the dialog by accident, per game, until it is sent.
	let drafts = $state<Record<string, { text: string; spoiler: boolean }>>({});
	$effect(() => {
		if (!reviewSheetOpen) return;
		drafts[game.slug] = { text: draftText, spoiler: draftSpoiler };
	});

	function openReview() {
		const draft = drafts[game.slug];
		draftText = draft?.text ?? myReview?.text ?? '';
		draftSpoiler = draft?.spoiler ?? myReview?.spoiler ?? false;
		reviewError = '';
		reviewsSheetOpen = false;
		reviewSheetOpen = true;
	}

	function saveReview(event: SubmitEvent) {
		event.preventDefault();
		if (!rating) {
			reviewError = 'Önce 1 ile 10 arasında bir puan seç.';
			return;
		}
		const text = draftText.trim();
		const slug = game.slug;
		if (text && text.length < 10) {
			reviewError = 'İncelemen en az 10 karakter olsun ya da boş bırak.';
			return;
		}
		reviewSheetOpen = false;
		delete drafts[slug];
		if (!text) {
			toast('Puanın kaydedildi');
			return;
		}
		const previous = myReviews[slug];
		myReviews[slug] = { text, spoiler: draftSpoiler };
		toast('İncelemen kaydedildi', {
			action: { label: 'Geri al', onClick: () => (myReviews[slug] = previous) }
		});
	}
	const seriesGames = $derived(
		(game.series ?? []).flatMap((slug) => {
			const other = allGameDetails.find((g) => g.slug === slug);
			return other
				? [{ slug: other.slug, title: other.title, cover: other.cover, meta: 'Aynı seriden' }]
				: [];
		})
	);
	const similar = $derived(trendingGames.filter((g) => g.slug !== game.slug).slice(0, 6));
</script>

{#snippet heading(id: string, title: string, meta?: string)}
	<div class="mb-3 flex items-baseline justify-between gap-3">
		<h2 {id} tabindex="-1" class="scroll-m-20 text-title-3 text-primary outline-none">{title}</h2>
		{#if meta}<span class="text-xs text-muted-foreground tabular-nums">{meta}</span>{/if}
	</div>
{/snippet}

{#snippet reviewCard(review: Review)}
	<Card.Root size="sm">
		<Card.Content>
			<div class="flex items-center gap-3">
				<Avatar.Root class="size-9">
					<Avatar.Fallback class="bg-foreground text-xs font-semibold text-background"
						>{review.initials}</Avatar.Fallback
					>
				</Avatar.Root>
				<div class="min-w-0 flex-1">
					<p class="text-sm font-semibold">{review.name}</p>
					<p class="text-[0.68rem] text-muted-foreground">{review.date}</p>
				</div>
				{#if review.score}
					<Badge variant="secondary"
						><StarIcon data-icon="inline-start" class="fill-current" />{review.score}/10</Badge
					>
				{/if}
			</div>
			{#if review.spoiler && !revealed[review.key]}
				<Alert.Root role="note" class="mt-3">
					<EyeOffIcon aria-hidden="true" />
					<Alert.Title>Bu inceleme spoiler içeriyor</Alert.Title>
					<Alert.Action>
						<Button
							type="button"
							variant="outline"
							size="xs"
							class="cursor-pointer"
							onclick={() => (revealed[review.key] = true)}>Göster</Button
						>
					</Alert.Action>
				</Alert.Root>
			{:else}
				<p class="mt-3 animate-in text-sm leading-6 text-foreground/80 duration-300 fade-in">
					{review.text}
				</p>
			{/if}
			<div class="mt-3 -ml-2 flex gap-1">
				<Button
					type="button"
					variant="ghost"
					size="sm"
					class="h-auto cursor-pointer rounded-full px-2.5 py-1 text-xs text-muted-foreground hover:text-foreground"
					aria-pressed={!!voted[review.key]}
					onclick={() => (voted[review.key] = !voted[review.key])}
				>
					<ThumbsUpIcon
						data-icon="inline-start"
						class={cn('size-3.5.5', voted[review.key] && 'fill-current')}
					/>
					Faydalı · {votes(review)}
				</Button>
				{#if review.spoiler && revealed[review.key]}
					<Button
						type="button"
						variant="ghost"
						size="sm"
						class="h-auto cursor-pointer rounded-full px-2.5 py-1 text-xs text-muted-foreground hover:text-foreground"
						onclick={() => (revealed[review.key] = false)}
					>
						<EyeOffIcon data-icon="inline-start" class="size-3.5.5" />
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
		<div
			class="pointer-events-none absolute inset-0 -z-10 overflow-hidden"
			aria-hidden="true"
			style="mask-image: linear-gradient(to bottom, black 55%, transparent 100%)"
		>
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

		<div class="detail-intro w-full min-w-0 px-4 md:px-8">
			<div class="flex items-start gap-4 sm:gap-6">
				<div class="w-28 shrink-0 sm:w-36 lg:w-40">
					<AspectRatio
						ratio={2 / 3}
						class="overflow-hidden rounded-2xl bg-muted shadow-[0_20px_50px_-18px_rgb(0_0_0/0.5)] ring-1 ring-white/20"
					>
						<img
							src={game.cover}
							alt="{game.title} kapağı"
							width="1024"
							height="1536"
							class="size-full object-cover"
						/>
					</AspectRatio>
				</div>

				<div class="flex min-w-0 flex-1 flex-col justify-end gap-5 self-stretch">
					<div class="min-w-0">
						<h1 id="game-title" class="detail-title text-large-title text-primary">
							{#if game.logo}
								<img
									src={game.logo}
									alt={game.title}
									class="h-auto max-h-24 w-auto max-w-full object-contain object-left-bottom drop-shadow-[0_4px_18px_rgb(0_0_0/0.55)] sm:max-h-28 sm:max-w-[26rem]"
								/>
							{:else}
								{game.title}
							{/if}
						</h1>
						<ul class="mt-4 flex flex-wrap gap-1.5" aria-label="Türler">
							{#each game.genres as genre (genre)}
								<li><Badge variant="secondary" class="px-2.5 py-1">{genre}</Badge></li>
							{/each}
						</ul>
					</div>
					<div class="flex items-center gap-2">
						<GameSaveMenu game={toListGame(game)} class="min-w-0 sm:w-64" />
						<DropdownMenu.Root>
							<DropdownMenu.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="ghost"
										size="icon"
										class="cursor-pointer rounded-full"
										aria-label="Diğer işlemler"
									>
										<EllipsisIcon />
									</Button>
								{/snippet}
							</DropdownMenu.Trigger>
							<DropdownMenu.Content align="start">
								<DropdownMenu.Item onSelect={() => (reportOpen = true)}>
									<FlagIcon /> Sorun bildir
								</DropdownMenu.Item>
							</DropdownMenu.Content>
						</DropdownMenu.Root>
					</div>
				</div>
			</div>

			<!-- Stat strip: one fact per tile, label above, value large, caption below. -->
			<dl class="stat-strip -mx-4 mt-6 flex snap-x overflow-x-auto px-4 py-4 md:mx-0 md:px-0">
				<div class="stat stat-wrap">
					<Button
						href="#reviews-title"
						onclick={(event) => scrollToSection(event, 'reviews-title')}
						variant="ghost"
						class="h-auto w-full cursor-pointer flex-col gap-1 rounded-2xl px-2 py-0 font-normal whitespace-normal"
						aria-label="10 üzerinden {game.score.toFixed(1)}, {game.votes.toLocaleString(
							'tr-TR'
						)} oy. Puanlar bölümüne git"
					>
						<span class="stat-label text-muted-foreground"
							>{game.votes.toLocaleString('tr-TR')} oy</span
						>
						<span class="stat-value tabular-nums">{game.score.toFixed(1)}</span>
						<span class="stat-caption flex gap-px text-muted-foreground" aria-hidden="true">
							{#each [0, 1, 2, 3, 4] as i (i)}
								<StarIcon
									class={cn(
										'size-3',
										i < Math.round(game.score / 2) ? 'fill-current' : 'opacity-30'
									)}
								/>
							{/each}
						</span>
					</Button>
				</div>
				<Separator orientation="vertical" class="my-1 h-auto self-stretch" />
				<div class="stat stat-wrap">
					<Button
						href="#info-title"
						onclick={(event) => scrollToSection(event, 'info-title')}
						variant="ghost"
						class="h-auto w-full cursor-pointer flex-col gap-1 rounded-2xl px-2 py-0 font-normal whitespace-normal"
						aria-label="Yaş sınırı {game.ageRating}. Bilgi bölümüne git"
					>
						<span class="stat-label text-muted-foreground">Yaş</span>
						<span class="stat-value">{game.ageRating}</span>
						<span class="stat-caption text-muted-foreground"
							>{ageDescription(game.ageRating)?.title ?? 'Yaş sınırı'}</span
						>
					</Button>
				</div>
				<Separator orientation="vertical" class="my-1 h-auto self-stretch" />
				<div class="stat stat-wrap">
					<Button
						href="#info-title"
						onclick={(event) => scrollToSection(event, 'info-title')}
						variant="ghost"
						class="h-auto w-full cursor-pointer flex-col gap-1 rounded-2xl px-2 py-0 font-normal whitespace-normal"
						aria-label="Oyun modları: {game.modes.join(', ')}. Bilgi bölümüne git"
					>
						<span class="stat-label text-muted-foreground">Mod</span>
						<span class="stat-value"><GamepadIcon aria-hidden="true" /></span>
						<span class="stat-caption text-muted-foreground">{game.modes.join(' · ')}</span>
					</Button>
				</div>
				<Separator orientation="vertical" class="my-1 h-auto self-stretch" />
				<div class="stat">
					<dt>Çıkış</dt>
					<dd class="stat-value tabular-nums">{new Date(game.releaseDate).getFullYear()}</dd>
					<dd class="stat-caption">
						{new Intl.DateTimeFormat('tr-TR', { day: 'numeric', month: 'short' }).format(
							new Date(game.releaseDate)
						)}
					</dd>
				</div>
				<Separator orientation="vertical" class="my-1 h-auto self-stretch" />
				<div class="stat">
					<dt>Geliştirici</dt>
					<dd class="stat-value"><CodeIcon aria-hidden="true" /></dd>
					<dd class="stat-caption">{game.developer}</dd>
				</div>
				<Separator orientation="vertical" class="my-1 h-auto self-stretch" />
				<div class="stat stat-wrap">
					<Button
						type="button"
						variant="ghost"
						class="h-auto w-full cursor-pointer flex-col gap-1 rounded-2xl px-2 py-0 font-normal whitespace-normal"
						onclick={() => (languagesOpen = true)}
						aria-haspopup="dialog"
					>
						<span class="stat-label text-muted-foreground">Dil</span>
						<span class="stat-value tabular-nums">{game.languages.length}</span>
						<span class="stat-caption text-muted-foreground">Dil desteği</span>
					</Button>
				</div>
			</dl>
			{#if game.requirements}
				{@const requirements = game.requirements}
				<Dialog.Root bind:open={requirementsOpen}>
					<Dialog.Content
						class="max-h-[85svh] overflow-y-auto sm:max-w-xl"
						onOpenAutoFocus={(event) => event.preventDefault()}
					>
						<Dialog.Header>
							<Dialog.Title>Sistem gereksinimleri</Dialog.Title>
							<Dialog.Description
								>{game.title} için PC'de en az ve önerilen donanım.</Dialog.Description
							>
						</Dialog.Header>
						<Table.Root>
							<Table.Header>
								<Table.Row class="hover:bg-transparent">
									<Table.Head class="h-9 pl-0"><span class="sr-only">Bileşen</span></Table.Head>
									<Table.Head class="h-9 text-xs font-normal text-muted-foreground"
										>Minimum</Table.Head
									>
									<Table.Head class="h-9 pr-0 text-xs font-normal text-muted-foreground"
										>Önerilen</Table.Head
									>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each requirementRows as row (row.key)}
									<Table.Row>
										<Table.Cell class="py-2.5 pl-0 text-xs text-muted-foreground"
											>{row.label}</Table.Cell
										>
										<Table.Cell class="py-2.5 text-[0.8125rem] font-medium whitespace-normal"
											>{requirements.minimum[row.key]}</Table.Cell
										>
										<Table.Cell class="py-2.5 pr-0 text-[0.8125rem] font-medium whitespace-normal"
											>{requirements.recommended[row.key]}</Table.Cell
										>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					</Dialog.Content>
				</Dialog.Root>
			{/if}

			<Dialog.Root bind:open={languagesOpen}>
				<Dialog.Content
					class="max-h-[85svh] overflow-y-auto"
					onOpenAutoFocus={(event) => event.preventDefault()}
				>
					<Dialog.Header>
						<Dialog.Title>Desteklenen diller</Dialog.Title>
						<Dialog.Description
							>{game.title} için {game.languages.length} dil: arayüz, seslendirme ve altyazı.</Dialog.Description
						>
					</Dialog.Header>
					<Table.Root>
						<Table.Header>
							<Table.Row class="hover:bg-transparent">
								<Table.Head class="h-9 pl-0 text-xs font-normal text-muted-foreground"
									>Dil</Table.Head
								>
								{#each languageColumns as column (column.key)}
									<Table.Head class="h-9 w-7 px-0 text-center">
										<Tooltip.Root>
											<Tooltip.Trigger>
												{#snippet child({ props })}
													<span {...props} class="inline-flex text-muted-foreground"
														><column.icon class="size-3.5.5" aria-hidden="true" /><span
															class="sr-only">{column.label}</span
														></span
													>
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
									<Table.Cell class="py-2 pl-0 text-[0.8125rem] font-medium"
										>{language.name}</Table.Cell
									>
									{#each languageColumns as column (column.key)}
										<Table.Cell class="px-0 py-2 text-center">
											{#if language[column.key]}
												<CheckIcon
													class="size-3.5.5 mx-auto text-primary"
													aria-hidden="true"
												/><span class="sr-only">{column.label} var</span>
											{:else}
												<MinusIcon
													class="size-3.5.5 mx-auto text-muted-foreground/40"
													aria-hidden="true"
												/><span class="sr-only">{column.label} yok</span>
											{/if}
										</Table.Cell>
									{/each}
									<Table.Cell class="p-0" aria-hidden="true"></Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				</Dialog.Content>
			</Dialog.Root>
		</div>
	</section>

	<!-- Two columns from lg up: game info on the left (as wide as the poster above), the right column is left free for more sections. -->
	<div class="grid w-full min-w-0 items-start gap-8 px-4 md:px-8">
		<!-- Two equal columns: the trailer sits alone in its own row (left half), screenshots in one row, and the score and reviews card at the very bottom. -->
		<div class="grid min-w-0 items-start gap-x-6 gap-y-10 xl:grid-cols-2">
			<!-- Like the App Store preview row: trailer first, then the screenshots, one height, one scrolling row that runs to the edge of the screen. -->
			<section aria-label="Önizleme" class="min-w-0 xl:col-span-2">
				<ScrollArea
					orientation="horizontal"
					class="-mx-4 w-[calc(100%+2rem)] md:-mx-8 md:w-[calc(100%+4rem)]"
				>
					<ul class="flex w-max gap-3 px-4 pb-3 md:px-8">
						<li class="aspect-video w-[clamp(16rem,24vw,22rem)] shrink-0">
							{#if game.trailer}
								<button
									type="button"
									onclick={() => openGallery(0)}
									aria-label="{game.title} fragmanını oynat"
									class="group relative block size-full cursor-pointer overflow-hidden rounded-2xl bg-muted ring-1 ring-black/8 outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:ring-white/10"
								>
									<img
										src={game.backdrop}
										alt=""
										loading="lazy"
										class="size-full object-cover transition-opacity group-hover:opacity-90"
									/>
									<span
										class="absolute inset-0 flex items-center justify-center"
										aria-hidden="true"
									>
										<span
											class="flex size-12 items-center justify-center rounded-full bg-black/45 text-white backdrop-blur-sm transition-transform group-hover:scale-105"
											><PlayIcon class="size-5 fill-current" /></span
										>
									</span>
								</button>
							{:else}
								<div
									class="relative size-full overflow-hidden rounded-2xl bg-muted ring-1 ring-black/8 dark:ring-white/10"
								>
									<img
										src={game.backdrop}
										alt=""
										loading="lazy"
										class="absolute inset-0 size-full object-cover opacity-60"
									/>
									<Empty.Root
										class="absolute inset-0 rounded-none border-0 bg-background/60 backdrop-blur-sm"
									>
										<Empty.Header>
											<Empty.Media variant="icon"><PlayIcon class="fill-current" /></Empty.Media>
											<Empty.Title>Fragman yakında</Empty.Title>
										</Empty.Header>
									</Empty.Root>
								</div>
							{/if}
						</li>
						{#each screenshots as shot, i (i)}
							<li class="aspect-video w-[clamp(16rem,24vw,22rem)] shrink-0">
								<button
									type="button"
									onclick={() => openGallery(i + trailerCount)}
									aria-label="{game.title} ekran görüntüsü {i + 1}, büyüt"
									class="block size-full cursor-pointer overflow-hidden rounded-2xl bg-muted ring-1 ring-black/8 transition-opacity outline-none hover:opacity-90 focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:ring-white/10"
								>
									<img src={shot} alt="" loading="lazy" class="size-full object-cover" />
								</button>
							</li>
						{/each}
					</ul>
				</ScrollArea>
			</section>

			<section aria-label="Açıklama" class="min-w-0 xl:col-span-2">
				<p class="text-callout whitespace-pre-line measure">{game.description}</p>
			</section>

			<section aria-labelledby="reviews-title" class="min-w-0 xl:col-span-2">
				<Separator class="mb-8" />
				{@render heading('reviews-title', 'Puanlar ve incelemeler', `${reviews.length} inceleme`)}
				<Card.Root class="gap-0 py-0">
					<Card.Content
						class="grid items-center gap-6 py-6 sm:grid-cols-[auto_1px_minmax(0,1fr)] sm:gap-8"
					>
						<div class="flex items-end gap-4">
							<p
								class="text-7xl leading-none font-medium tracking-[-0.075em] text-primary tabular-nums"
								aria-label="10 üzerinden {game.score.toFixed(1)}"
							>
								{game.score.toFixed(1)}
							</p>
							<div class="space-y-1 pb-1">
								<span class="flex items-center gap-0.5" aria-hidden="true">
									{#each [1, 2, 3, 4, 5] as star (star)}
										<span class="relative inline-flex size-5">
											<StarIcon class="size-5 text-primary" />
											<span
												class="absolute inset-y-0 left-0 overflow-hidden"
												style:width="{Math.min(Math.max(game.score / 2 - (star - 1), 0), 1) * 100}%"
											>
												<StarIcon class="size-5 fill-primary text-primary" />
											</span>
										</span>
									{/each}
								</span>
								<p class="text-xs text-muted-foreground">
									<span class="font-medium text-primary tabular-nums"
										>{game.votes.toLocaleString('tr-TR')}</span
									> oy
								</p>
							</div>
						</div>
						<Separator orientation="vertical" class="hidden self-stretch! sm:block" />
						<div class="space-y-2">
							{#each distribution as group (group.range)}
								<div class="grid grid-cols-[3.25rem_1fr_2.25rem] items-center gap-3 text-xs">
									<span class="flex items-center gap-1.5 font-medium text-primary tabular-nums">
										<StarIcon class="size-3.5 fill-current" aria-hidden="true" />
										{group.range}
									</span>
									<Progress
										value={group.percent}
										aria-label="{group.range} puan: %{group.percent}"
									/>
									<span class="text-right text-muted-foreground tabular-nums">%{group.percent}</span
									>
								</div>
							{/each}
						</div>
					</Card.Content>

					<Separator />

					<Card.Content class="py-6">
						{#if featuredReview}
							<Item.Root variant="muted" class="flex-col items-stretch gap-4 p-5 sm:p-6">
								<p class="text-xs font-medium text-muted-foreground">En faydalı inceleme</p>
								<Item.Content>
									<Item.Description
										class="line-clamp-none text-lg leading-8 font-medium tracking-tight text-pretty text-primary sm:text-xl"
										>{featuredReview.text}</Item.Description
									>
								</Item.Content>
								<Item.Header class="justify-start gap-3 text-xs">
									<Avatar.Root class="size-7">
										<Avatar.Fallback
											class="bg-primary text-[0.65rem] font-semibold text-primary-foreground"
											>{featuredReview.initials}</Avatar.Fallback
										>
									</Avatar.Root>
									<span class="text-sm font-semibold text-primary">{featuredReview.name}</span>
									{#if featuredReview.score}<Badge variant="secondary"
											><StarIcon
												data-icon="inline-start"
												class="fill-current"
											/>{featuredReview.score}/10</Badge
										>{/if}
									<span class="text-muted-foreground">{featuredReview.date}</span>
									<span class="text-muted-foreground"
										>· {votes(featuredReview)} kişi faydalı buldu</span
									>
								</Item.Header>
							</Item.Root>
						{:else}
							<p class="text-sm text-muted-foreground">
								Henüz inceleme yok. İlk incelemeyi sen yaz.
							</p>
						{/if}
					</Card.Content>

					<Card.Footer class="flex-wrap gap-2 border-t bg-muted/30 py-4">
						<Button type="button" class="cursor-pointer rounded-full px-5" onclick={openReview}
							>{myReview ? 'İncelemeni düzenle' : 'İnceleme yaz'}</Button
						>
						<Button
							type="button"
							variant="outline"
							class="cursor-pointer rounded-full px-5"
							onclick={() => (reviewsSheetOpen = true)}
							>Tüm incelemeler <span class="text-muted-foreground tabular-nums"
								>({reviews.length})</span
							></Button
						>
					</Card.Footer>
				</Card.Root>
			</section>

			<section aria-labelledby="info-title" class="min-w-0 xl:col-span-2">
				<Separator class="mb-8" />
				{@render heading('info-title', 'Bilgi')}
				<dl
					class="grid items-start gap-x-8 gap-y-6 text-sm sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
				>
					{#each info as row (row.label)}
						<div class="min-w-0">
							<dt class="flex items-center gap-1.5 text-footnote text-muted-foreground">
								<row.icon class="size-4 shrink-0" aria-hidden="true" />{row.label}
							</dt>
							{#if row.label === 'Yaş sınırı'}
								{@const age = ageDescription(game.ageRating)}
								<!-- Opens in place, like a disclosure row: the age label stays, the explanation unfolds under it. -->
								<Collapsible.Root>
									<dd class="mt-1">
										<Collapsible.Trigger
											class="-mx-2 inline-flex max-w-full cursor-pointer items-center gap-1 rounded-xl px-2 py-0.5 text-left font-medium transition-colors hover:bg-muted [&[data-state=open]>svg]:rotate-90"
										>
											<span class="truncate">{game.ageRating}</span>
											<ChevronRightIcon
												class="size-4 shrink-0 text-muted-foreground transition-transform"
												aria-hidden="true"
											/>
										</Collapsible.Trigger>
									</dd>
									{#if age}
										<Collapsible.Content>
											<dd class="mt-1 max-w-xs text-footnote text-muted-foreground">{age.text}</dd>
										</Collapsible.Content>
									{/if}
								</Collapsible.Root>
							{:else}
								<dd class="mt-1 font-medium">{row.values ? row.values.join(', ') : row.value}</dd>
							{/if}
						</div>
					{/each}
					<div class="min-w-0">
						<dt class="flex items-center gap-1.5 text-footnote text-muted-foreground">
							<LanguagesIcon class="size-4 shrink-0" aria-hidden="true" />Diller
						</dt>
						<dd class="mt-1">
							<button
								type="button"
								onclick={() => (languagesOpen = true)}
								aria-haspopup="dialog"
								class="-mx-2 inline-flex max-w-full cursor-pointer items-center gap-1 rounded-xl px-2 py-0.5 text-left font-medium transition-colors hover:bg-muted"
							>
								<span class="truncate"
									>{game.languages[0]?.name}{game.languages.length > 1
										? ` ve ${game.languages.length - 1} tane daha`
										: ''}</span
								>
								<ChevronRightIcon
									class="size-4 shrink-0 text-muted-foreground"
									aria-hidden="true"
								/>
							</button>
						</dd>
					</div>
					{#if game.requirements}
						<div class="min-w-0">
							<dt class="flex items-center gap-1.5 text-footnote text-muted-foreground">
								<CpuIcon class="size-4 shrink-0" aria-hidden="true" />Sistem gereksinimleri
							</dt>
							<dd class="mt-1">
								<button
									type="button"
									onclick={() => (requirementsOpen = true)}
									aria-haspopup="dialog"
									class="-mx-2 inline-flex max-w-full cursor-pointer items-center gap-1 rounded-xl px-2 py-0.5 text-left font-medium transition-colors hover:bg-muted"
								>
									<span class="truncate">Minimum ve önerilen</span>
									<ChevronRightIcon
										class="size-4 shrink-0 text-muted-foreground"
										aria-hidden="true"
									/>
								</button>
							</dd>
						</div>
					{/if}
				</dl>
			</section>
		</div>
	</div>

	<GameGalleryDialog
		title={game.title}
		items={galleryItems}
		start={galleryStart}
		bind:open={galleryOpen}
	/>

	<!-- All reviews, like the anime page: a side drawer with a sort tab and one card per review. -->
	<Sheet.Root bind:open={reviewsSheetOpen}>
		<Sheet.Content class="w-full gap-0 sm:max-w-md [&_[data-slot=sheet-close]]:cursor-pointer">
			<Sheet.Header class="border-b">
				<Sheet.Title class="text-lg font-medium tracking-tight text-primary"
					>İncelemeler</Sheet.Title
				>
				<Sheet.Description
					>{game.title} için {reviews.length} inceleme · Ortalama {game.score
						.toFixed(1)
						.replace('.', ',')}</Sheet.Description
				>
				<Tabs.Root bind:value={reviewSort} class="mt-2">
					<Tabs.List class="rounded-xl" aria-label="İncelemeleri sırala">
						<Tabs.Trigger value="helpful" class="cursor-pointer rounded-lg px-3 text-xs"
							>En faydalı</Tabs.Trigger
						>
						<Tabs.Trigger value="newest" class="cursor-pointer rounded-lg px-3 text-xs"
							>En yeni</Tabs.Trigger
						>
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
				<Button type="button" class="cursor-pointer rounded-full" onclick={openReview}
					>{myReview ? 'İncelemeni düzenle' : 'İnceleme yaz'}</Button
				>
			</Sheet.Footer>
		</Sheet.Content>
	</Sheet.Root>

	<GameReportDialog title={game.title} bind:open={reportOpen} />

	<Dialog.Root bind:open={reviewSheetOpen}>
		<Dialog.Content onOpenAutoFocus={(event) => event.preventDefault()}>
			<Dialog.Header class="flex-row items-center gap-3 pe-10">
				<img
					src={game.cover}
					alt=""
					width="40"
					height="60"
					class="h-[3.75rem] w-10 shrink-0 rounded-lg bg-muted object-cover ring-1 ring-black/10 dark:ring-white/15"
				/>
				<div class="min-w-0">
					<Dialog.Title>İncelemeni yaz</Dialog.Title>
					<Dialog.Description class="truncate">{game.title}</Dialog.Description>
				</div>
			</Dialog.Header>
			<form class="space-y-4" onsubmit={saveReview}>
				<div class="space-y-4 rounded-2xl bg-muted/50 p-4">
					<div class="text-center" aria-live="polite">
						<p class="text-large-title tabular-nums">
							{rating ?? '–'}<span class="text-title-3 font-normal text-muted-foreground">/10</span>
						</p>
						<p class="mt-1 text-callout text-muted-foreground">
							{rating ? scoreWords[rating - 1] : 'Bir puan seç'}
						</p>
					</div>
					<ToggleGroup.Root
						type="single"
						variant="outline"
						spacing={1}
						value={rating ? String(rating) : ''}
						onValueChange={(value) => value && (gameRating[game.slug] = Number(value))}
						aria-label="Puanın, 1 ile 10 arasında"
						class="grid w-full grid-cols-5 gap-1.5 sm:grid-cols-10"
					>
						{#each Array.from({ length: 10 }, (_, index) => index + 1) as n (n)}
							<ToggleGroup.Item
								value={String(n)}
								aria-label="{n} puan"
								class="aspect-square h-auto w-full cursor-pointer rounded-xl bg-background p-0 text-xs font-semibold data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
							>
								{n}
							</ToggleGroup.Item>
						{/each}
					</ToggleGroup.Root>
				</div>
				<div class="space-y-2">
					<Label for="review"
						>Düşüncelerin <span class="font-normal text-muted-foreground">(isteğe bağlı)</span
						></Label
					>
					<Textarea
						id="review"
						bind:value={draftText}
						placeholder="{game.title} hakkında ne düşünüyorsun?"
						class="min-h-28 resize-none rounded-xl"
					/>
				</div>
				<div class="flex items-center justify-between gap-4">
					<div class="min-w-0">
						<Label for="spoiler" class="cursor-pointer">Spoiler içeriyor</Label>
						<p class="text-footnote text-muted-foreground">
							Yazın varsayılan olarak gizli gösterilir.
						</p>
					</div>
					<Switch id="spoiler" bind:checked={draftSpoiler} />
				</div>
				{#if reviewError}<p class="text-footnote text-destructive" role="alert">
						{reviewError}
					</p>{/if}
				<Dialog.Footer>
					<Button variant="outline" class="cursor-pointer" onclick={() => (reviewSheetOpen = false)}
						>Vazgeç</Button
					>
					<Button type="submit" class="cursor-pointer"
						>{draftText.trim() ? 'İncelemeyi gönder' : 'Puanı kaydet'}</Button
					>
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
			linear-gradient(
				0deg,
				var(--background) 0%,
				color-mix(in oklch, var(--background) 70%, transparent) 30%,
				transparent 65%
			),
			linear-gradient(
				90deg,
				color-mix(in oklch, var(--background) 92%, transparent) 0%,
				color-mix(in oklch, var(--background) 78%, transparent) 45%,
				color-mix(in oklch, var(--background) 45%, transparent) 100%
			),
			linear-gradient(
				180deg,
				color-mix(in oklch, var(--background) 55%, transparent) 0%,
				transparent 22%
			);
	}

	.detail-intro {
		padding-top: 7rem;
		padding-bottom: 1rem;
	}

	.detail-title {
		line-height: 1.05;
		overflow-wrap: anywhere;
	}

	.stat {
		display: flex;
		flex: 1 0 6.5rem;
		scroll-snap-align: start;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
		padding-inline: 0.75rem;
		text-align: center;
		color: var(--muted-foreground);
	}
	.stat dt {
		font-size: 0.6875rem;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}
	.stat-value {
		display: flex;
		height: 2rem;
		align-items: center;
		font-size: 1.375rem;
		font-weight: 650;
		color: var(--foreground);
	}
	.stat-value :global(svg) {
		width: 1.5rem;
		height: 1.5rem;
	}
	.stat-wrap {
		padding-inline: 0.25rem;
	}
	.stat-label {
		font-size: 0.6875rem;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	.stat-link:hover {
		text-decoration: underline;
	}
	.stat-caption {
		font-size: 0.8125rem;
		line-height: 1.2;
		max-width: 9rem;
		overflow-wrap: anywhere;
	}
</style>
