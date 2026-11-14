<script lang="ts">
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import { AspectRatio } from '#lib/components/ui/aspect-ratio/index.js';
	import * as Card from '#lib/components/ui/card/index.js';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import ClockIcon from '@lucide/svelte/icons/clock-3';
	import GamepadIcon from '@lucide/svelte/icons/gamepad-2';

	// The card a list wears on the profile and in the library: a cover, one round icon badge, then
	// the name and "N games · description".
	let {
		name,
		kind,
		cover,
		gameCount,
		description,
		href
	}: {
		name: string;
		kind: 'played' | 'play_later' | 'custom';
		cover: string;
		gameCount: number;
		description?: string;
		href: string | null;
	} = $props();

	const Icon = $derived(
		{ played: CircleCheckIcon, play_later: ClockIcon, custom: GamepadIcon }[kind]
	);
</script>

{#snippet body()}
	<div class="relative">
		<AspectRatio ratio={2.6} class="bg-muted">
			{#if cover}
				<img
					src={cover}
					alt=""
					width="1600"
					height="560"
					loading="lazy"
					class="size-full object-cover"
				/>
			{/if}
		</AspectRatio>
		<Avatar.Root class="absolute -bottom-7 left-(--card-spacing) size-14 ring-4 ring-card">
			<Avatar.Fallback><Icon class="size-5 text-muted-foreground" /></Avatar.Fallback>
		</Avatar.Root>
	</div>
	<Card.Header class="pt-10">
		<Card.Title class="truncate">{name}</Card.Title>
		<Card.Description class="truncate"
			>{gameCount}
			{gameCount === 1 ? 'game' : 'games'}{description ? ` · ${description}` : ''}</Card.Description
		>
	</Card.Header>
{/snippet}

<Card.Root
	class="group/list h-full gap-0 pt-0 pb-5 {href
		? 'transition-transform duration-300 hover:-translate-y-1'
		: ''}"
>
	{#if href}
		<a
			{href}
			class="flex flex-col rounded-2xl outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
			aria-label="View {name}">{@render body()}</a
		>
	{:else}
		<div class="flex flex-col">{@render body()}</div>
	{/if}
</Card.Root>
