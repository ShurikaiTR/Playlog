<script lang="ts">
	import { onMount } from "svelte";
	import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
	import type { HeroSlide } from "#lib/hero-slides.js";
	import { Button } from "#lib/components/ui/button/index.js";

	let { slides }: { slides: HeroSlide[] } = $props();
	const SLIDE_DURATION = 8000;

	let currentIndex = $state(0);
	let progress = $state(0);
	let paused = $state(false);
	let elapsed = 0;
	let lastFrame: number | undefined;
	let animationFrame: number;
	const current = $derived(slides[currentIndex]);

	function updateAutoplay(timestamp: number) {
		if (lastFrame === undefined) lastFrame = timestamp;

		if (!paused) {
			elapsed += timestamp - lastFrame;
			progress = Math.min(elapsed / SLIDE_DURATION, 1);
		}

		lastFrame = timestamp;

		if (elapsed >= SLIDE_DURATION) {
			currentIndex = (currentIndex + 1) % slides.length;
			elapsed = 0;
			progress = 0;
		}

		animationFrame = requestAnimationFrame(updateAutoplay);
	}

	function showSlide(index: number) {
		currentIndex = (index + slides.length) % slides.length;
		elapsed = 0;
		progress = 0;
	}

	// Arrow keys step through the slides, unless the user is typing somewhere.
	function onKeydown(event: KeyboardEvent) {
		if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
		if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;
		const target = event.target;
		if (
			target instanceof HTMLElement &&
			target.closest('input, textarea, select, [contenteditable=""], [contenteditable="true"]')
		) {
			return;
		}
		showSlide(currentIndex + (event.key === "ArrowRight" ? 1 : -1));
	}

	// A horizontal swipe does the same on touch screens.
	let touchStartX: number | null = null;
	function onTouchEnd(event: TouchEvent) {
		if (touchStartX === null) return;
		const delta = event.changedTouches[0].clientX - touchStartX;
		touchStartX = null;
		if (Math.abs(delta) > 50) showSlide(currentIndex + (delta < 0 ? 1 : -1));
	}

	onMount(() => {
		animationFrame = requestAnimationFrame(updateAutoplay);
		return () => cancelAnimationFrame(animationFrame);
	});
</script>

<svelte:window onkeydown={onKeydown} />

{#if current}
	<section
		class="hero"
		aria-labelledby="hero-title"
		ontouchstart={(event) => (touchStartX = event.touches[0].clientX)}
		ontouchend={onTouchEnd}
	>
		{#each slides as slide, index (slide.slug)}
			<img
				class={["hero-image", { active: index === currentIndex }]}
				src={slide.image}
				alt=""
				aria-hidden={index !== currentIndex}
				width="1600"
				height="900"
				fetchpriority={index === 0 ? "high" : "auto"}
			/>
		{/each}
		<div class="hero-overlay"></div>

		{#key current.slug}
			<div
				class="hero-copy"
				role="group"
				aria-label="Öne çıkan oyun"
				onpointerenter={() => (paused = true)}
				onpointerleave={() => (paused = false)}
			>
				<div class="game-logo" id="hero-title">
					{#if current.logo}
						<img src={current.logo} alt={current.title} class="hero-logo" />
					{:else}
						<h1 class="text-display">{current.title}</h1>
					{/if}
				</div>
				<div class="metadata" aria-label="Oyun bilgileri">
					{#if current.rating}
						<span class="rating">{current.rating}</span>
					{/if}
					{#each current.genres as genre (genre)}
						<span class="genre">{genre}</span>
					{/each}
				</div>
				{#if current.description}
					<p class="description">{current.description}</p>
				{/if}
				<div class="actions">
					<Button href={current.href} size="lg" class="h-12 px-6 text-base">
						İncele
						<ArrowRightIcon data-icon="inline-end" class="size-4.5" />
					</Button>
				</div>
			</div>
		{/key}

		<nav class="slider-controls" aria-label="Öne çıkan oyunlar">
			<p class="counter numeric" aria-live="off">
				<span>{String(currentIndex + 1).padStart(2, "0")}</span>
				<span class="counter-total">/ {String(slides.length).padStart(2, "0")}</span>
			</p>
			<div class="segments">
				{#each slides as slide, index (slide.slug)}
					<button
						type="button"
						class={{ active: index === currentIndex }}
						aria-label={`${index + 1}. oyun: ${slide.title}`}
						aria-current={index === currentIndex ? "true" : undefined}
						onclick={() => showSlide(index)}
					>
						<span
							class="segment-fill"
							style:transform={`scaleX(${index === currentIndex ? (paused ? 1 : progress) : 0})`}
						></span>
					</button>
				{/each}
			</div>
		</nav>
	</section>
{/if}

<style>
	/* Full-bleed: slides up under the fixed header (4rem tall). */
	/* Full-bleed: slides under the header and the floating sidebar, copy stays in the content area. */
	.hero {
		--offset: var(--sidebar-offset, 0px);
		position: relative;
		isolation: isolate;
		display: flex;
		min-height: clamp(34rem, 78vh, 48rem);
		margin-top: -4rem;
		margin-left: calc(-1 * var(--offset));
		padding-top: 4rem;
		padding-left: var(--offset);
		overflow: hidden;
		color: var(--foreground);
	}

	.hero-image {
		position: absolute;
		inset: 0;
		z-index: -2;
		height: 100%;
		width: 100%;
		object-fit: cover;
		object-position: center 30%;
		opacity: 0;
		/* Fade the artwork out at the bottom so no hard edge shows against the page. */
		mask-image: linear-gradient(to bottom, black 55%, transparent 100%);
		transform: scale(1.035);
		transition:
			opacity 0.8s ease,
			transform 1.2s cubic-bezier(0.22, 1, 0.36, 1);
	}

	.hero-image.active {
		opacity: 1;
		transform: scale(1.015);
	}

	/* Washes the artwork into the page background on every edge but the right. */
	.hero-overlay {
		position: absolute;
		inset: 0;
		z-index: -1;
		background:
			linear-gradient(
				0deg,
				var(--background) 0%,
				color-mix(in oklch, var(--background) 70%, transparent) 22%,
				transparent 55%
			),
			linear-gradient(
				90deg,
				color-mix(in oklch, var(--background) 88%, transparent) 0%,
				color-mix(in oklch, var(--background) 62%, transparent) 40%,
				color-mix(in oklch, var(--background) 30%, transparent) 100%
			),
			linear-gradient(180deg, color-mix(in oklch, var(--background) 55%, transparent) 0%, transparent 22%);
	}

	/* Fills the content area, same gutters as the rows below. */
	.hero-copy {
		align-self: center;
		width: 100%;
		padding: 3rem 1rem 6rem;
	}

	.hero-copy > * {
		max-width: 40rem;
	}

	.game-logo {
		display: inline-flex;
		flex-direction: column;
		align-items: flex-start;
		padding: 0.25rem 0 0.65rem;
	}

	.hero-logo {
		max-height: clamp(5rem, 14vw, 9rem);
		max-width: min(26rem, 100%);
		width: auto;
		object-fit: contain;
		object-position: left bottom;
	}

	.metadata {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.65rem;
		margin-top: 1rem;
		font-size: 0.75rem;
		font-weight: 500;
		letter-spacing: 0.04em;
		color: color-mix(in oklch, var(--foreground) 72%, transparent);
	}

	.rating {
		border: 1px solid color-mix(in oklch, var(--foreground) 25%, transparent);
		border-radius: 0.375rem;
		background: color-mix(in oklch, var(--background) 40%, transparent);
		padding: 0.125rem 0.375rem;
		color: color-mix(in oklch, var(--foreground) 85%, transparent);
	}

	.genre:not(:first-child)::before,
	.rating + .genre::before {
		margin-right: 0.65rem;
		color: color-mix(in oklch, var(--foreground) 28%, transparent);
		content: "•";
	}

	.rating + .genre::before {
		content: none;
	}

	.description {
		display: -webkit-box;
		overflow: hidden;
		-webkit-line-clamp: 4;
		line-clamp: 4;
		-webkit-box-orient: vertical;
		max-width: 32rem;
		margin-top: 1.5rem;
		font-size: clamp(0.875rem, 0.8rem + 0.3vw, 1rem);
		line-height: 1.7;
		color: color-mix(in oklch, var(--foreground) 78%, transparent);
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.75rem;
		margin-top: 2rem;
	}

	/* Lined up with the copy above: a counter and one thin bar per slide, the active one filling. */
	.slider-controls {
		position: absolute;
		bottom: 2.5rem;
		left: calc(var(--offset) + 1rem);
		display: flex;
		align-items: center;
		gap: 1.25rem;
	}

	.counter {
		display: flex;
		align-items: baseline;
		gap: 0.35rem;
		font-size: 0.8125rem;
		font-weight: 600;
		letter-spacing: 0.08em;
	}

	.counter-total {
		font-weight: 500;
		color: color-mix(in oklch, var(--foreground) 45%, transparent);
	}

	.segments {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	/* The button is a tall hit area; the thin bar is drawn inside it. */
	.segments button {
		position: relative;
		display: grid;
		height: 1.5rem;
		width: 2.75rem;
		cursor: pointer;
		place-items: center;
		border-radius: 999px;
		outline: none;
		transition: width 0.3s ease;
	}

	.segments button:focus-visible {
		box-shadow: 0 0 0 3px color-mix(in oklch, var(--ring) 50%, transparent);
	}

	.segments button::before {
		content: "";
		grid-area: 1 / 1;
		height: 3px;
		width: 100%;
		border-radius: 999px;
		background: color-mix(in oklch, var(--foreground) 22%, transparent);
		transition: background-color 0.2s ease;
	}

	.segments button:hover::before {
		background: color-mix(in oklch, var(--foreground) 40%, transparent);
	}

	.segments button.active {
		width: 4.5rem;
	}

	.segment-fill {
		grid-area: 1 / 1;
		height: 3px;
		width: 100%;
		border-radius: 999px;
		background: var(--foreground);
		transform-origin: left;
	}

	@media (min-width: 48rem) {
		.hero-copy {
			padding-inline: 2rem;
		}

		.slider-controls {
			left: calc(var(--offset) + 2rem);
		}
	}

	@media (max-width: 47.5rem) {
		.hero {
			min-height: 40rem;
		}

		.hero-image {
			object-position: 66% center;
		}

		.hero-copy {
			align-self: flex-end;
			padding: 8rem 1rem 6rem;
		}

		.slider-controls {
			bottom: 1.5rem;
		}
	}

	@media (prefers-reduced-motion: no-preference) {
		.hero-copy {
			animation: rise 0.65s cubic-bezier(0.22, 1, 0.36, 1) both;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.hero-image,
		.segments button,
		.segments button::before {
			transition: none;
		}
	}

	@keyframes rise {
		from {
			opacity: 0;
			transform: translateY(14px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
