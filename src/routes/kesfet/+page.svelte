<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import { cn } from '#lib/utils.js';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ShuffleIcon from '@lucide/svelte/icons/shuffle';
	import Volume2Icon from '@lucide/svelte/icons/volume-2';
	import VolumeXIcon from '@lucide/svelte/icons/volume-x';
	import { MediaQuery } from 'svelte/reactivity';

	let { data } = $props();

	// The mood showcase: the chosen game in a big card, the rest lined up after it.
	let index = $state(0);
	const items = $derived(data.mood.items);
	const featured = $derived(items[index]);
	$effect.pre(() => {
		void data.mood.key;
		index = 0;
	});

	// The trailer plays muted in the big card; the button toggles the sound through the player API.
	let player = $state<HTMLIFrameElement>();
	let soundSlug = $state<string | null>(null);
	// Which game's trailer has loaded; until then the artwork stays, then the video fades in over it.
	let readySlug = $state<string | null>(null);
	const muted = $derived(soundSlug !== featured?.slug);
	const reducedMotion = new MediaQuery('(prefers-reduced-motion: reduce)');
	function toggleSound() {
		soundSlug = muted ? featured.slug : null;
		player?.contentWindow?.postMessage(
			JSON.stringify({ event: 'command', func: muted ? 'mute' : 'unMute', args: [] }),
			'*'
		);
	}

	// The gallery moves on by itself with a thin progress bar; the first move the user makes stops it for good.
	const SLIDE_DURATION = 12000;
	let progress = $state(0);
	let hovering = false;
	let userStopped = $state(false);
	$effect(() => {
		void index;
		progress = 0;
		if (userStopped || reducedMotion.current) return;
		let elapsed = 0;
		let last: number | undefined;
		let frame = 0;
		const tick = (time: number) => {
			last ??= time;
			if (!hovering) elapsed += time - last;
			last = time;
			progress = Math.min(elapsed / SLIDE_DURATION, 1);
			if (elapsed >= SLIDE_DURATION) {
				index = (index + 1) % items.length;
				return;
			}
			frame = requestAnimationFrame(tick);
		};
		frame = requestAnimationFrame(tick);
		return () => cancelAnimationFrame(frame);
	});

	function show(next: number) {
		userStopped = true;
		if (next >= 0 && next < items.length) index = next;
	}

	// The strip is a native scroller, so trackpad and touch scrolling keep the system's own momentum.
	// Scrolling only moves the strip; a card grows when it is chosen. The first scroll stops the autoplay.
	let scroller = $state<HTMLElement>();
	let stripHeight = $state(0);
	const GAP = 16;
	$effect(() => {
		if (!scroller || !stripHeight) return;
		// Posters are 2:3 and as tall as the big card; the previous one stays peeking in at the left.
		const left = Math.max(0, index - 1) * ((stripHeight * 2) / 3 + GAP);
		scroller.scrollTo({ left, behavior: reducedMotion.current ? 'auto' : 'smooth' });
	});
	function onWheel(event: WheelEvent) {
		if (Math.abs(event.deltaX) > Math.abs(event.deltaY)) userStopped = true;
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
		if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;
		if (
			event.target instanceof HTMLElement &&
			event.target.closest('input, textarea, select, [contenteditable]')
		)
			return;
		show(index + (event.key === 'ArrowRight' ? 1 : -1));
	}
</script>

<svelte:head>
	<title>Keşfet | Playlog</title>
	<meta
		name="description"
		content="Ruh haline göre oyun keşfet: macera, gerilim, hikâye odaklı oyunlar ve daha fazlası."
	/>
</svelte:head>

<svelte:window onkeydown={onKeydown} />

<div lang="tr" class="min-w-0 px-4 pb-20 md:px-8 md:pb-24">
	<header class="flex flex-wrap items-end justify-between gap-6 pt-2 pb-6 md:pt-4">
		<div class="max-w-2xl">
			<h1
				class="text-[2rem] leading-tight font-medium tracking-[-0.025em] text-primary md:text-[2.5rem]"
			>
				Keşfet
			</h1>
			<p class="mt-2 text-sm leading-[1.6] text-muted-foreground">
				<span class="font-medium text-primary">{data.mood.title}</span> · {data.mood.subtitle}
			</p>
		</div>
		<Button
			href="/kesfet?ruh={data.surprise}"
			variant="outline"
			size="lg"
			class="h-11 px-5 text-[0.9375rem]"
		>
			<ShuffleIcon data-icon="inline-start" aria-hidden="true" />
			Rastgele bir oyun
		</Button>
	</header>

	{#if featured}
		<section
			aria-label={data.mood.title}
			class="@container min-w-0 select-none"
			onpointerenter={() => (hovering = true)}
			onpointerleave={() => (hovering = false)}
		>
			<div class="relative [--fw:100cqw] md:[--fw:72cqw] lg:[--fw:62cqw]">
				<div
					role="group"
					aria-label="Oyunlar"
					bind:this={scroller}
					bind:clientHeight={stripHeight}
					onwheel={onWheel}
					ontouchmove={() => (userStopped = true)}
					class="flex [scrollbar-width:none] items-start gap-4 overflow-x-auto overscroll-x-contain [&::-webkit-scrollbar]:hidden"
				>
					{#each items as item, position (item.slug)}
						<div
							class="h-[calc(var(--fw)*9/16)] shrink-0 transition-[width] duration-500 ease-[cubic-bezier(0.22,1,0.36,1)] motion-reduce:transition-none {position ===
							index
								? 'w-(--fw)'
								: 'w-[calc(var(--fw)*9/16*2/3)]'}"
						>
							{#if position === index}
								<div
									class="relative size-full overflow-hidden rounded-2xl bg-muted ring-1 ring-foreground/10"
								>
									<img
										src={item.backdrop}
										alt=""
										draggable="false"
										width="1672"
										height="941"
										class="size-full object-cover"
									/>
									{#if item.trailer && !reducedMotion.current}
										{#key item.slug}
											<iframe
												bind:this={player}
												title="{item.title} fragmanı"
												src="https://www.youtube-nocookie.com/embed/{item.trailer}?autoplay=1&mute=1&controls=0&loop=1&playlist={item.trailer}&playsinline=1&modestbranding=1&rel=0&enablejsapi=1"
												allow="autoplay; encrypted-media"
												referrerpolicy="strict-origin-when-cross-origin"
												onload={() => (readySlug = item.slug)}
												class={cn(
													'pointer-events-none absolute inset-0 size-full scale-[1.35] border-0 opacity-0 transition-opacity duration-700 motion-reduce:transition-none',
													readySlug === item.slug && 'opacity-100'
												)}
											></iframe>
										{/key}
									{/if}
									<a
										href="/oyun/{item.slug}"
										class="absolute inset-0 outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:ring-inset"
										aria-label="{item.title} sayfasına git"
									></a>
									<div
										class="pointer-events-none absolute inset-x-0 bottom-0 bg-linear-to-t from-black/70 via-black/25 to-transparent p-4 pt-20 pr-16 md:p-6 md:pt-24 md:pr-20"
									>
										{#if item.logo}
											<img
												src={item.logo}
												alt={item.title}
												draggable="false"
												class="h-12 w-auto max-w-[60%] object-contain object-left drop-shadow-[0_2px_12px_rgb(0_0_0/0.5)] md:h-16"
											/>
										{:else}
											<span
												class="block text-xl leading-tight font-medium tracking-[-0.02em] text-white md:text-3xl"
												>{item.title}</span
											>
										{/if}
									</div>
									{#if item.trailer && !reducedMotion.current}
										<Button
											variant="ghost"
											size="icon-lg"
											class="absolute right-3 bottom-3 z-10 rounded-full bg-background/80 shadow-sm backdrop-blur-md hover:bg-background"
											aria-label={muted ? 'Sesi aç' : 'Sesi kapat'}
											onclick={toggleSound}
										>
											{#if muted}<VolumeXIcon aria-hidden="true" />{:else}<Volume2Icon
													aria-hidden="true"
												/>{/if}
										</Button>
									{/if}
								</div>
							{:else}
								<button
									type="button"
									onclick={() => show(position)}
									aria-label="{item.title} oyununu göster"
									class="relative size-full cursor-pointer overflow-hidden rounded-2xl bg-muted ring-1 ring-foreground/10 transition-shadow outline-none hover:ring-foreground/30 focus-visible:ring-[3px] focus-visible:ring-ring/50 motion-reduce:transition-none"
								>
									<img
										src={item.cover}
										alt=""
										draggable="false"
										width="1024"
										height="1536"
										loading="lazy"
										class="size-full object-cover"
									/>
								</button>
							{/if}
						</div>
					{/each}
				</div>
				{#if index > 0}
					<Button
						variant="outline"
						size="icon"
						class="absolute top-1/2 left-3 z-10 size-8 -translate-y-1/2 rounded-full bg-background hover:bg-muted"
						aria-label="Önceki oyun"
						onclick={() => show(index - 1)}
					>
						<ChevronLeftIcon aria-hidden="true" />
					</Button>
				{/if}
				{#if index < items.length - 1}
					<Button
						variant="outline"
						size="icon"
						class="absolute top-1/2 right-3 z-10 size-8 -translate-y-1/2 rounded-full bg-background hover:bg-muted"
						aria-label="Sonraki oyun"
						onclick={() => show(index + 1)}
					>
						<ChevronRightIcon aria-hidden="true" />
					</Button>
				{/if}
			</div>

			{#if !userStopped && !reducedMotion.current}
				<div
					class="mt-4 h-[3px] max-w-xs overflow-hidden rounded-full bg-foreground/10"
					aria-hidden="true"
				>
					<div class="h-full origin-left bg-foreground" style:transform="scaleX({progress})"></div>
				</div>
			{/if}

			{#key featured.slug}
				<div class="mt-6 flex max-w-2xl flex-col items-start gap-3" aria-live="polite">
					<p
						class="text-xs font-medium tracking-wider text-muted-foreground uppercase tabular-nums"
					>
						{index + 1} / {items.length}
					</p>
					<h2 class="-mt-1 text-2xl font-medium tracking-[-0.02em] text-primary md:text-3xl">
						{featured.title}
					</h2>
					<p class="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
						<span
							class="rounded-md border bg-background px-2 py-0.5 text-xs font-semibold text-primary"
							>{featured.ageRating}</span
						>
						{#each featured.genres.slice(0, 3) as genre (genre)}
							<span aria-hidden="true" class="text-foreground/30">•</span>
							<span>{genre}</span>
						{/each}
					</p>
					<p class="line-clamp-3 text-sm leading-relaxed text-muted-foreground">
						{featured.description}
					</p>
					<Button href="/oyun/{featured.slug}" size="lg" class="mt-1 h-11 px-5 text-[0.9375rem]">
						İncele
						<ArrowRightIcon data-icon="inline-end" aria-hidden="true" />
					</Button>
				</div>
			{/key}
		</section>
	{/if}
</div>
