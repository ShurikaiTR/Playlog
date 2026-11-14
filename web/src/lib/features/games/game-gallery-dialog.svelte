<script lang="ts">
	import { AspectRatio } from '#lib/components/ui/aspect-ratio/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Carousel from '#lib/components/ui/carousel/index.js';
	import type { CarouselAPI } from '#lib/components/ui/carousel/context.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { cn } from '#lib/utils.js';

	// Preview: the picture on the dimmed page, round arrows at the screen edges. Clicking the empty area or pressing Esc closes it.
	type Props = {
		/** The game's title, for the labels. */
		title: string;
		/** The trailer (a YouTube id) first when there is one, then the screenshots. */
		items: { kind: 'video' | 'image'; src: string }[];
		/** The item to show when the dialog opens. */
		start: number;
		open: boolean;
	};
	let { title, items, start, open = $bindable(false) }: Props = $props();

	const videoCount = $derived(items.filter((item) => item.kind === 'video').length);
	const imageCount = $derived(items.length - videoCount);

	let api = $state<CarouselAPI>();
	let index = $state(0);
	let canPrev = $state(false);
	let canNext = $state(false);
	// The carousel mounts with the dialog; jump to the clicked item as soon as it is ready.
	$effect(() => {
		const carousel = api;
		if (!open || !carousel) return;
		carousel.scrollTo(start, true);
		index = start;
		const onSelect = () => {
			index = carousel.selectedScrollSnap();
			canPrev = carousel.canScrollPrev();
			canNext = carousel.canScrollNext();
		};
		onSelect();
		carousel.on('select', onSelect);
		return () => carousel.off('select', onSelect);
	});
</script>

<Dialog.Root bind:open>
	<Dialog.Content
		class="top-0 left-0 grid h-svh max-h-none w-screen max-w-none translate-x-0 translate-y-0 place-items-center overflow-visible rounded-none bg-transparent p-0 shadow-none ring-0 sm:max-w-none dark:ring-0"
		overlayClass="bg-black/80 supports-backdrop-filter:backdrop-blur-md"
		showCloseButton={false}
		aria-describedby={undefined}
		onOpenAutoFocus={(event) => event.preventDefault()}
		onclick={(event) => event.target === event.currentTarget && (open = false)}
		onkeydown={(event) => {
			// The arrow buttons handle the keys themselves when focused.
			if (event.defaultPrevented) return;
			if (event.key === 'ArrowLeft') api?.scrollPrev();
			else if (event.key === 'ArrowRight') api?.scrollNext();
		}}
	>
		<Dialog.Title class="sr-only">{title} screenshots</Dialog.Title>
		<Carousel.Root
			setApi={(carousel) => (api = carousel)}
			opts={{ startIndex: start }}
			class="w-[min(92vw,calc(78svh*16/9))]"
		>
			<Carousel.Content>
				{#each items as item, i (i)}
					<Carousel.Item>
						<AspectRatio ratio={16 / 9} class="overflow-hidden rounded-xl bg-muted shadow-2xl">
							{#if item.kind === 'video'}
								<!-- Only the slide that is showing holds the player, so leaving it stops the sound. -->
								{#if open && index === i}
									<iframe
										title="{title} trailer"
										src="https://www.youtube-nocookie.com/embed/{item.src}?autoplay=1&rel=0"
										allow="accelerometer; autoplay; encrypted-media; picture-in-picture"
										allowfullscreen
										class="size-full"
									></iframe>
								{/if}
							{:else}
								<img
									src={item.src}
									alt="{title} screenshot {i + 1 - videoCount} of {imageCount}"
									class="size-full object-cover"
								/>
							{/if}
						</AspectRatio>
					</Carousel.Item>
				{/each}
			</Carousel.Content>
		</Carousel.Root>
		{#each [{ dir: -1, label: 'Previous image', can: canPrev, pos: 'start-5' }, { dir: 1, label: 'Next image', can: canNext, pos: 'end-5' }] as arrow (arrow.dir)}
			<Button
				variant="ghost"
				size="icon"
				class="{arrow.pos} group/arrow fixed inset-y-0 my-auto size-10 cursor-pointer rounded-full border-0 bg-black/40 text-white backdrop-blur-sm hover:bg-black/60 hover:text-white active:bg-black/70"
				disabled={!arrow.can}
				aria-label={arrow.label}
				onclick={() => (arrow.dir < 0 ? api?.scrollPrev() : api?.scrollNext())}
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class={cn('size-5', arrow.dir < 0 && 'rotate-180')}
					aria-hidden="true"
				>
					<line
						x1="5"
						y1="12"
						x2="19"
						y2="12"
						class="origin-right scale-x-0 transition-transform duration-150 [transform-box:fill-box] group-hover/arrow:scale-x-100"
					/>
					<polyline
						points="11 8 15 12 11 16"
						class="transition-transform duration-150 group-hover/arrow:translate-x-1"
					/>
				</svg>
			</Button>
		{/each}
	</Dialog.Content>
</Dialog.Root>
