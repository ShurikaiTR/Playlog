<script lang="ts">
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import * as Card from '#lib/components/ui/card/index.js';
	import * as Alert from '#lib/components/ui/alert/index.js';
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Switch } from '#lib/components/ui/switch/index.js';
	import { ScrollArea } from '#lib/components/ui/scroll-area/index.js';
	import { AspectRatio } from '#lib/components/ui/aspect-ratio/index.js';
	import * as ToggleGroup from '#lib/components/ui/toggle-group/index.js';
	import * as Item from '#lib/components/ui/item/index.js';
	import ResponsivePanel from '#lib/components/layout/responsive-panel.svelte';
	import * as Collapsible from '#lib/components/ui/collapsible/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import { toast } from 'svelte-sonner';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { compactCount, initials, type SocialReview } from '#lib/features/games/game-social.js';
	import { invalidate } from '$app/navigation';
	import { promptSignIn, socialRequest, failToast } from '#lib/features/lists/social-api.js';
	import { cn } from '#lib/utils.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import * as Table from '#lib/components/ui/table/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';
	import GameSaveMenu from '#lib/features/lists/game-save-menu.svelte';
	import { toListGame } from '#lib/features/lists/user-lists.svelte.js';
	import GameGalleryDialog from '#lib/features/games/game-gallery-dialog.svelte';
	import GameReportDialog from '#lib/features/games/game-report-dialog.svelte';
	import GameRow from '#lib/features/games/game-row.svelte';
	import { formatDayMonth, formatRelease } from '#lib/features/games/games.js';
	import FlagIcon from '@lucide/svelte/icons/flag';
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

	// Steam does not say which languages have subtitles, so that column only shows when known.
	const languageColumns = $derived(
		(
			[
				{ key: 'interface', label: 'Interface', icon: MonitorIcon },
				{ key: 'audio', label: 'Audio', icon: Volume2Icon },
				{ key: 'subtitles', label: 'Subtitles', icon: CaptionsIcon }
			] as const
		).filter((column) => game.languages.some((language) => language[column.key]))
	);
	const names = (items: { name: string }[]) => items.map((item) => item.name).join(', ');
	// Only the facts the catalog has for this game.
	const info = $derived(
		[
			{ label: 'Developer', value: names(game.developers), icon: CodeIcon },
			{ label: 'Publisher', value: names(game.publishers), icon: BuildingIcon },
			{
				label: 'Release date',
				value: game.releaseDate ? formatRelease(game.releaseDate, game.releasePrecision) : '',
				icon: CalendarIcon
			},
			{ label: 'Platforms', value: names(game.platforms), icon: MonitorIcon },
			{ label: 'Game modes', value: names(game.modes), icon: GamepadIcon },
			{ label: 'Genres', value: names(game.genres), icon: TagIcon },
			{ label: 'Age rating', value: game.ageRating?.label ?? '', icon: UsersRoundIcon }
		].filter((row) => row.value)
	);
	const requirementRows = [
		{ key: 'os', label: 'Operating system' },
		{ key: 'cpu', label: 'Processor' },
		{ key: 'gpu', label: 'Graphics' },
		{ key: 'ram', label: 'Memory' },
		{ key: 'storage', label: 'Storage' }
	] as const;
	const screenshots = $derived(game.screenshots);
	// The gallery shows the trailer first (when there is one), then the screenshots.
	const trailerCount = $derived(game.trailer ? 1 : 0);
	const galleryItems = $derived<{ kind: 'video' | 'image'; src: string }[]>([
		...(game.trailer ? [{ kind: 'video' as const, src: game.trailer }] : []),
		...screenshots.map((src) => ({ kind: 'image' as const, src }))
	]);
	type Review = SocialReview;

	const social = $derived(data.social);
	const myRating = $derived(social.mine?.rating ?? null);
	const myReview = $derived(social.mine?.review ?? null);
	// The server sends the viewer's own review first, then the newest.
	const reviews = $derived<Review[]>(social.reviews);
	// The featured review never contains spoilers, since it shows without a click.
	const featuredReview = $derived(reviews.find((review) => !review.spoiler));
	let revealed = $state<Record<string, boolean>>({});
	const scoreWords = [
		'Awful',
		'Very bad',
		'Bad',
		'Weak',
		'Mediocre',
		'Decent',
		'Good',
		'Very good',
		'Great',
		'Masterpiece'
	];

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
	let draftRating = $state<number | null>(null);
	let draftText = $state('');
	let draftSpoiler = $state(false);
	let reviewError = $state('');
	let saving = $state(false);

	// Unsent writing survives closing the dialog by accident, per game, until it is sent.
	let drafts = $state<Record<string, { rating: number | null; text: string; spoiler: boolean }>>(
		{}
	);
	$effect(() => {
		if (!reviewSheetOpen) return;
		drafts[game.slug] = { rating: draftRating, text: draftText, spoiler: draftSpoiler };
	});

	function openReview() {
		if (!data.user) {
			promptSignIn('rate and review games');
			return;
		}
		const draft = drafts[game.slug];
		draftRating = draft?.rating ?? myRating;
		draftText = draft?.text ?? myReview?.text ?? '';
		draftSpoiler = draft?.spoiler ?? myReview?.spoiler ?? false;
		reviewError = '';
		reviewsSheetOpen = false;
		reviewSheetOpen = true;
	}

	async function saveReview(event: SubmitEvent) {
		event.preventDefault();
		if (!draftRating) {
			reviewError = 'Pick a score from 1 to 10 first.';
			return;
		}
		const text = draftText.trim();
		if (text && text.length < 10) {
			reviewError = 'Write at least 10 characters, or leave it empty.';
			return;
		}
		const slug = game.slug;
		saving = true;
		let result = await socialRequest('PUT', `games/${slug}/rating`, { score: draftRating });
		if (result.ok && text) {
			result = await socialRequest('PUT', `games/${slug}/review`, {
				text,
				spoiler: draftSpoiler
			});
		} else if (result.ok && myReview) {
			// The text was cleared: the rating stays, the review goes.
			result = await socialRequest('DELETE', `games/${slug}/review`);
		}
		saving = false;
		if (!result.ok) {
			failToast(result.error);
			return;
		}
		reviewSheetOpen = false;
		delete drafts[slug];
		await invalidate('app:social');
		toast(text ? 'Your review was saved' : 'Your rating was saved');
	}
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
					{#if review.image}<Avatar.Image src={review.image} alt="" />{/if}
					<Avatar.Fallback class="bg-foreground text-xs font-semibold text-background"
						>{initials(review.name)}</Avatar.Fallback
					>
				</Avatar.Root>
				<div class="min-w-0 flex-1">
					<p class="text-sm font-semibold">{review.mine ? 'You' : review.name}</p>
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
					<Alert.Title>This review contains spoilers</Alert.Title>
					<Alert.Action>
						<Button
							type="button"
							variant="outline"
							size="xs"
							class="cursor-pointer"
							onclick={() => (revealed[review.key] = true)}>Show</Button
						>
					</Alert.Action>
				</Alert.Root>
			{:else}
				<p class="mt-3 animate-in text-sm leading-6 text-foreground/80 duration-300 fade-in">
					{review.text}
				</p>
			{/if}
			<div class="mt-3 -ml-2 flex gap-1">
				{#if review.spoiler && revealed[review.key]}
					<Button
						type="button"
						variant="ghost"
						size="sm"
						class="h-auto cursor-pointer rounded-full px-2.5 py-1 text-xs text-muted-foreground hover:text-foreground"
						onclick={() => (revealed[review.key] = false)}
					>
						<EyeOffIcon data-icon="inline-start" class="size-3.5.5" />
						Hide
					</Button>
				{/if}
			</div>
		</Card.Content>
	</Card.Root>
{/snippet}

<svelte:head>
	<title>{game.title} | Playthru</title>
	{#if game.summary}<meta name="description" content={game.summary} />{/if}
</svelte:head>

<div class="flex min-w-0 flex-col gap-16 pb-12">
	<section aria-labelledby="game-title" class="detail-hero relative isolate min-w-0">
		<!-- Full-bleed: slides under the header and the floating sidebar. -->
		<div
			class="pointer-events-none absolute inset-0 -z-10 overflow-hidden"
			aria-hidden="true"
			style="mask-image: linear-gradient(to bottom, black 55%, transparent 100%)"
		>
			{#if game.backdrop}
				<img
					src={game.backdrop}
					alt=""
					width="1600"
					height="900"
					fetchpriority="high"
					class="size-full object-cover object-[center_45%]"
				/>
			{/if}
			<div class="detail-scrim absolute inset-0"></div>
		</div>

		<div class="detail-intro w-full min-w-0 px-4 md:px-8">
			<div class="flex items-start gap-4 sm:gap-6">
				<div class="w-28 shrink-0 sm:w-36 lg:w-40">
					<AspectRatio
						ratio={2 / 3}
						class="overflow-hidden rounded-2xl bg-muted shadow-[0_20px_50px_-18px_rgb(0_0_0/0.5)] ring-1 ring-white/20"
					>
						{#if game.cover}
							<img
								src={game.cover}
								alt="{game.title} cover"
								width="600"
								height="900"
								class="size-full object-cover"
							/>
						{/if}
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
						{#if game.parent}
							<p class="mt-3 text-footnote text-muted-foreground">
								{game.kind === 'dlc' || game.kind === 'expansion' ? 'Add-on for' : 'Based on'}
								<a
									href="/games/{game.parent.slug}"
									class="font-medium text-primary underline-offset-4 hover:underline"
									>{game.parent.title}</a
								>
							</p>
						{/if}
						{#if game.genres.length}
							<ul class="mt-4 flex flex-wrap gap-1.5" aria-label="Genres">
								{#each game.genres as genre (genre.slug)}
									<li>
										<Badge variant="secondary" class="px-2.5 py-1" href="/games?genre={genre.slug}"
											>{genre.name}</Badge
										>
									</li>
								{/each}
							</ul>
						{/if}
						{#if social.played || social.playLater}
							<div class="mt-3 flex items-center gap-3">
								{#if social.people.length}
									<Avatar.Group aria-hidden="true">
										{#each social.people as person (person.username)}
											<Avatar.Root class="size-7">
												{#if person.image}<Avatar.Image src={person.image} alt="" />{/if}
												<Avatar.Fallback
													class="bg-foreground text-[0.625rem] font-semibold text-background"
													>{initials(person.name)}</Avatar.Fallback
												>
											</Avatar.Root>
										{/each}
									</Avatar.Group>
								{/if}
								<p class="text-footnote text-muted-foreground">
									{#if social.played}<span class="tabular-nums">{compactCount(social.played)}</span>
										played{/if}{#if social.played && social.playLater}
										·
									{/if}{#if social.playLater}<span class="tabular-nums"
											>{compactCount(social.playLater)}</span
										>
										play later{/if}
								</p>
							</div>
						{/if}
					</div>
					<div class="flex items-center gap-2">
						<GameSaveMenu game={toListGame(game)} />
						<DropdownMenu.Root>
							<DropdownMenu.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="ghost"
										size="icon"
										class="cursor-pointer rounded-full"
										aria-label="More actions"
									>
										<EllipsisIcon />
									</Button>
								{/snippet}
							</DropdownMenu.Trigger>
							<DropdownMenu.Content align="start">
								<DropdownMenu.Item onSelect={() => (reportOpen = true)}>
									<FlagIcon /> Report a problem
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
						aria-label={social.score === null
							? 'No ratings yet. Go to the ratings section'
							: `${social.score.toFixed(1)} out of 10, ${social.ratings.toLocaleString('en')} ${social.ratings === 1 ? 'rating' : 'ratings'}. Go to the ratings section`}
					>
						<span class="stat-label text-muted-foreground">Rating</span>
						<span class="stat-value tabular-nums">
							{#if social.score === null}
								<span class="text-muted-foreground">–</span>
							{:else}
								{social.score.toFixed(1)}<span
									class="ms-0.5 self-end pb-1 text-xs font-medium text-muted-foreground">/10</span
								>
							{/if}
						</span>
						<span class="stat-caption flex items-center gap-1.5 text-muted-foreground">
							{#if social.score === null}
								Be the first to rate
							{:else}
								<span class="flex gap-px" aria-hidden="true">
									{#each [0, 1, 2, 3, 4] as i (i)}
										<StarIcon
											class={cn(
												'size-3',
												i < Math.round(social.score / 2) ? 'fill-current' : 'opacity-30'
											)}
										/>
									{/each}
								</span>
								<span class="tabular-nums">{compactCount(social.ratings)}</span>
							{/if}
						</span>
					</Button>
				</div>
				{#if game.playtime}
					<Separator orientation="vertical" class="my-1 h-auto self-stretch first:hidden" />
					<div class="stat">
						<dt>Time to beat</dt>
						<dd class="stat-value tabular-nums">{game.playtime}</dd>
						<dd class="stat-caption">{game.playtime === 1 ? 'hour' : 'hours'}, main story</dd>
					</div>
				{/if}
				{#if game.releaseDate}
					<Separator orientation="vertical" class="my-1 h-auto self-stretch first:hidden" />
					<div class="stat">
						<dt>Release</dt>
						<dd class="stat-value tabular-nums">{game.releaseDate.slice(0, 4)}</dd>
						<dd class="stat-caption">
							{game.releasePrecision === 'day'
								? formatDayMonth(game.releaseDate)
								: formatRelease(game.releaseDate, game.releasePrecision)}
						</dd>
					</div>
				{/if}
				{#if game.ageRating}
					<Separator orientation="vertical" class="my-1 h-auto self-stretch first:hidden" />
					<div class="stat stat-wrap">
						<Button
							href="#info-title"
							onclick={(event) => scrollToSection(event, 'info-title')}
							variant="ghost"
							class="h-auto w-full cursor-pointer flex-col gap-1 rounded-2xl px-2 py-0 font-normal whitespace-normal"
							aria-label="Age rating {game.ageRating.label}. Go to the info section"
						>
							<span class="stat-label text-muted-foreground">Age</span>
							<span class="stat-value">{game.ageRating.label.split(' ')[1]}</span>
							<span class="stat-caption text-muted-foreground">{game.ageRating.title}</span>
						</Button>
					</div>
				{/if}
				{#if game.modes.length}
					<Separator orientation="vertical" class="my-1 h-auto self-stretch first:hidden" />
					<div class="stat stat-wrap">
						<Button
							href="#info-title"
							onclick={(event) => scrollToSection(event, 'info-title')}
							variant="ghost"
							class="h-auto w-full cursor-pointer flex-col gap-1 rounded-2xl px-2 py-0 font-normal whitespace-normal"
							aria-label="Game modes: {names(game.modes)}. Go to the info section"
						>
							<span class="stat-label text-muted-foreground">Mode</span>
							<span class="stat-value"><GamepadIcon aria-hidden="true" /></span>
							<span class="stat-caption text-muted-foreground"
								>{game.modes.map((mode) => mode.name).join(' · ')}</span
							>
						</Button>
					</div>
				{/if}
				{#if game.developers.length}
					<Separator orientation="vertical" class="my-1 h-auto self-stretch first:hidden" />
					<div class="stat">
						<dt>Developer</dt>
						<dd class="stat-value"><CodeIcon aria-hidden="true" /></dd>
						<dd class="stat-caption">{names(game.developers)}</dd>
					</div>
				{/if}
				{#if game.languages.length}
					<Separator orientation="vertical" class="my-1 h-auto self-stretch first:hidden" />
					<div class="stat stat-wrap">
						<Button
							type="button"
							variant="ghost"
							class="h-auto w-full cursor-pointer flex-col gap-1 rounded-2xl px-2 py-0 font-normal whitespace-normal"
							onclick={() => (languagesOpen = true)}
							aria-haspopup="dialog"
						>
							<span class="stat-label text-muted-foreground">Languages</span>
							<span class="stat-value tabular-nums">{game.languages.length}</span>
							<span class="stat-caption text-muted-foreground">Supported</span>
						</Button>
					</div>
				{/if}
			</dl>
			{#if game.requirements}
				{@const requirements = game.requirements}
				<Dialog.Root bind:open={requirementsOpen}>
					<Dialog.Content
						class="max-h-[85svh] overflow-y-auto sm:max-w-xl"
						onOpenAutoFocus={(event) => event.preventDefault()}
					>
						<Dialog.Header>
							<Dialog.Title>System requirements</Dialog.Title>
							<Dialog.Description
								>Minimum and recommended PC hardware for {game.title}.</Dialog.Description
							>
						</Dialog.Header>
						<Table.Root>
							<Table.Header>
								<Table.Row class="hover:bg-transparent">
									<Table.Head class="h-9 pl-0"><span class="sr-only">Component</span></Table.Head>
									<Table.Head class="h-9 text-xs font-normal text-muted-foreground"
										>Minimum</Table.Head
									>
									<Table.Head class="h-9 pr-0 text-xs font-normal text-muted-foreground"
										>Recommended</Table.Head
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
											>{requirements.minimum?.[row.key] ?? '–'}</Table.Cell
										>
										<Table.Cell class="py-2.5 pr-0 text-[0.8125rem] font-medium whitespace-normal"
											>{requirements.recommended?.[row.key] ?? '–'}</Table.Cell
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
						<Dialog.Title>Supported languages</Dialog.Title>
						<Dialog.Description
							>{game.languages.length}
							{game.languages.length === 1 ? 'language' : 'languages'} for {game.title}.</Dialog.Description
						>
					</Dialog.Header>
					<Table.Root>
						<Table.Header>
							<Table.Row class="hover:bg-transparent">
								<Table.Head class="h-9 pl-0 text-xs font-normal text-muted-foreground"
									>Language</Table.Head
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
							{#each game.languages as language (language.code)}
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
												/><span class="sr-only">{column.label}: yes</span>
											{:else}
												<MinusIcon
													class="size-3.5.5 mx-auto text-muted-foreground/40"
													aria-hidden="true"
												/><span class="sr-only">{column.label}: no</span>
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
			{#if galleryItems.length}
				<section aria-label="Preview" class="min-w-0 xl:col-span-2">
					<ScrollArea
						orientation="horizontal"
						class="-mx-4 w-[calc(100%+2rem)] md:-mx-8 md:w-[calc(100%+4rem)]"
					>
						<ul class="flex w-max gap-3 px-4 pb-3 md:px-8">
							{#if game.trailer}
								<li class="aspect-video w-[clamp(16rem,24vw,22rem)] shrink-0">
									<button
										type="button"
										onclick={() => openGallery(0)}
										aria-label="Play the {game.title} trailer"
										class="group relative block size-full cursor-pointer overflow-hidden rounded-2xl bg-muted ring-1 ring-black/8 outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:ring-white/10"
									>
										{#if game.backdrop}
											<img
												src={game.backdrop}
												alt=""
												loading="lazy"
												class="size-full object-cover transition-opacity group-hover:opacity-90"
											/>
										{/if}
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
								</li>
							{/if}
							{#each screenshots as shot, i (i)}
								<li class="aspect-video w-[clamp(16rem,24vw,22rem)] shrink-0">
									<button
										type="button"
										onclick={() => openGallery(i + trailerCount)}
										aria-label="Enlarge {game.title} screenshot {i + 1}"
										class="block size-full cursor-pointer overflow-hidden rounded-2xl bg-muted ring-1 ring-black/8 transition-opacity outline-none hover:opacity-90 focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:ring-white/10"
									>
										<img src={shot} alt="" loading="lazy" class="size-full object-cover" />
									</button>
								</li>
							{/each}
						</ul>
					</ScrollArea>
				</section>
			{/if}

			{#if game.description ?? game.summary}
				<section aria-label="About" class="min-w-0 xl:col-span-2">
					<p class="text-callout whitespace-pre-line measure">
						{game.description ?? game.summary}
					</p>
				</section>
			{/if}

			<section aria-labelledby="reviews-title" class="min-w-0 xl:col-span-2">
				<Separator class="mb-8" />
				{@render heading('reviews-title', 'Ratings and reviews')}
				<Card.Root class="gap-0 py-0">
					<Card.Content class="py-6">
						{#if featuredReview}
							<Item.Root variant="muted" class="flex-col items-stretch gap-4 p-5 sm:p-6">
								<p class="text-xs font-medium text-muted-foreground">
									{featuredReview.mine ? 'Your review' : 'Latest review'}
								</p>
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
											>{initials(featuredReview.name)}</Avatar.Fallback
										>
									</Avatar.Root>
									<span class="text-sm font-semibold text-primary"
										>{featuredReview.mine ? 'You' : featuredReview.name}</span
									>
									{#if featuredReview.score}<Badge variant="secondary"
											><StarIcon
												data-icon="inline-start"
												class="fill-current"
											/>{featuredReview.score}/10</Badge
										>{/if}
									<span class="text-muted-foreground">{featuredReview.date}</span>
								</Item.Header>
							</Item.Root>
						{:else}
							<div class="flex flex-col items-center gap-1 py-4 text-center">
								<!-- The drawing mixes light and dark ink, so it sits on a light disc in dark mode. -->
								<img
									src="/mascot/confused.svg"
									alt=""
									width="160"
									height="160"
									loading="lazy"
									class="size-40 rounded-full object-contain dark:bg-zinc-200"
								/>
								<p class="mt-3 text-title-3 text-primary">No reviews yet</p>
								<p class="text-callout text-muted-foreground">Be the first to review it!</p>
								<Button
									type="button"
									class="mt-4 cursor-pointer rounded-full px-5"
									onclick={openReview}>Write a review</Button
								>
							</div>
						{/if}
					</Card.Content>

					{#if reviews.length}
						<Card.Footer class="flex-wrap gap-2 border-t bg-muted/30 py-4">
							<Button type="button" class="cursor-pointer rounded-full px-5" onclick={openReview}
								>{myReview ? 'Edit your review' : 'Write a review'}</Button
							>
							{#if reviews.length}
								<Button
									type="button"
									variant="outline"
									class="cursor-pointer rounded-full px-5"
									onclick={() => (reviewsSheetOpen = true)}
									>All reviews <span class="text-muted-foreground tabular-nums"
										>({reviews.length})</span
									></Button
								>
							{/if}
						</Card.Footer>
					{/if}
				</Card.Root>
			</section>

			<section aria-labelledby="info-title" class="min-w-0 xl:col-span-2">
				<Separator class="mb-8" />
				{@render heading('info-title', 'Information')}
				<dl
					class="grid items-start gap-x-8 gap-y-6 text-sm sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
				>
					{#each info as row (row.label)}
						<div class="min-w-0">
							<dt class="flex items-center gap-1.5 text-footnote text-muted-foreground">
								<row.icon class="size-4 shrink-0" aria-hidden="true" />{row.label}
							</dt>
							{#if row.label === 'Age rating'}
								{@const age = game.ageRating}
								<!-- Opens in place, like a disclosure row: the age label stays, the explanation unfolds under it. -->
								<Collapsible.Root>
									<dd class="mt-1">
										<Collapsible.Trigger
											class="-mx-2 inline-flex max-w-full cursor-pointer items-center gap-1 rounded-xl px-2 py-0.5 text-left font-medium transition-colors hover:bg-muted [&[data-state=open]>svg]:rotate-90"
										>
											<span class="truncate">{row.value}</span>
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
								<dd class="mt-1 font-medium">{row.value}</dd>
							{/if}
						</div>
					{/each}
					{#if game.languages.length}
						<div class="min-w-0">
							<dt class="flex items-center gap-1.5 text-footnote text-muted-foreground">
								<LanguagesIcon class="size-4 shrink-0" aria-hidden="true" />Languages
							</dt>
							<dd class="mt-1">
								<button
									type="button"
									onclick={() => (languagesOpen = true)}
									aria-haspopup="dialog"
									class="-mx-2 inline-flex max-w-full cursor-pointer items-center gap-1 rounded-xl px-2 py-0.5 text-left font-medium transition-colors hover:bg-muted"
								>
									<span class="truncate"
										>{game.languages[0].name}{game.languages.length > 1
											? ` and ${game.languages.length - 1} more`
											: ''}</span
									>
									<ChevronRightIcon
										class="size-4 shrink-0 text-muted-foreground"
										aria-hidden="true"
									/>
								</button>
							</dd>
						</div>
					{/if}
					{#if game.requirements}
						<div class="min-w-0">
							<dt class="flex items-center gap-1.5 text-footnote text-muted-foreground">
								<CpuIcon class="size-4 shrink-0" aria-hidden="true" />System requirements
							</dt>
							<dd class="mt-1">
								<button
									type="button"
									onclick={() => (requirementsOpen = true)}
									aria-haspopup="dialog"
									class="-mx-2 inline-flex max-w-full cursor-pointer items-center gap-1 rounded-xl px-2 py-0.5 text-left font-medium transition-colors hover:bg-muted"
								>
									<span class="truncate">View details</span>
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

	<!-- All reviews: a side panel (a bottom drawer on phones) with a sort tab and one card per review. -->
	<ResponsivePanel
		bind:open={reviewsSheetOpen}
		title="Reviews"
		description="{reviews.length} {reviews.length === 1 ? 'review' : 'reviews'} of {game.title}"
		sheetClass="w-full sm:max-w-md [&_[data-slot=sheet-close]]:cursor-pointer"
		headerClass="border-b"
		titleClass="text-lg font-medium tracking-tight text-primary"
		footerClass="border-t"
	>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<div class="space-y-3 p-4">
				{#each reviews as review (review.key)}
					{@render reviewCard(review)}
				{/each}
			</div>
		</div>
		{#snippet footer()}
			<Button type="button" class="cursor-pointer rounded-full" onclick={openReview}
				>{myReview ? 'Edit your review' : 'Write a review'}</Button
			>
		{/snippet}
	</ResponsivePanel>

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
					<Dialog.Title>Write your review</Dialog.Title>
					<Dialog.Description class="truncate">{game.title}</Dialog.Description>
				</div>
			</Dialog.Header>
			<form class="space-y-4" onsubmit={saveReview}>
				<div class="space-y-4 rounded-2xl bg-muted/50 p-4">
					<div class="text-center" aria-live="polite">
						<p class="text-large-title tabular-nums">
							{draftRating ?? '–'}<span class="text-title-3 font-normal text-muted-foreground"
								>/10</span
							>
						</p>
						<p class="mt-1 text-callout text-muted-foreground">
							{draftRating ? scoreWords[draftRating - 1] : 'Pick a score'}
						</p>
					</div>
					<ToggleGroup.Root
						type="single"
						variant="outline"
						spacing={1}
						value={draftRating ? String(draftRating) : ''}
						onValueChange={(value) => value && (draftRating = Number(value))}
						aria-label="Your score, from 1 to 10"
						class="grid w-full grid-cols-5 gap-1.5 sm:grid-cols-10"
					>
						{#each Array.from({ length: 10 }, (_, index) => index + 1) as n (n)}
							<ToggleGroup.Item
								value={String(n)}
								aria-label="Score {n}"
								class="aspect-square h-auto w-full cursor-pointer rounded-xl bg-background p-0 text-xs font-semibold data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
							>
								{n}
							</ToggleGroup.Item>
						{/each}
					</ToggleGroup.Root>
				</div>
				<div class="space-y-2">
					<Label for="review"
						>Your thoughts <span class="font-normal text-muted-foreground">(optional)</span></Label
					>
					<Textarea
						id="review"
						bind:value={draftText}
						placeholder="What did you think of {game.title}?"
						class="min-h-28 resize-none rounded-xl"
					/>
				</div>
				<div class="flex items-center justify-between gap-4">
					<div class="min-w-0">
						<Label for="spoiler" class="cursor-pointer">Contains spoilers</Label>
						<p class="text-footnote text-muted-foreground">
							Your review stays hidden until a reader chooses to see it.
						</p>
					</div>
					<Switch id="spoiler" bind:checked={draftSpoiler} />
				</div>
				{#if reviewError}<p class="text-footnote text-destructive" role="alert">
						{reviewError}
					</p>{/if}
				<Dialog.Footer>
					<Button variant="outline" class="cursor-pointer" onclick={() => (reviewSheetOpen = false)}
						>Cancel</Button
					>
					<Button type="submit" class="cursor-pointer" disabled={saving}
						>{draftText.trim() ? 'Post review' : 'Save rating'}</Button
					>
				</Dialog.Footer>
			</form>
		</Dialog.Content>
	</Dialog.Root>

	{#if game.related.length}
		<GameRow title="Add-ons and editions" href="/games" games={game.related} />
	{/if}
	{#if game.series && game.seriesGames.length}
		<GameRow title="More from {game.series.name}" href="/games" games={game.seriesGames} />
	{/if}
	{#if game.similar.length}
		<GameRow title="Similar games" href="/games" games={game.similar} />
	{/if}
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
