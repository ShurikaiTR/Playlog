<script lang="ts">
	import * as Item from '#lib/components/ui/item/index.js';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import TagIcon from '@lucide/svelte/icons/tag';

	let { data } = $props();
</script>

<svelte:head>
	<title>Genres | Playthru</title>
	<meta name="description" content="Browse games by genre." />
</svelte:head>

<div class="min-w-0 px-4 pb-20 md:px-8 md:pb-24">
	<header class="pt-2 pb-6 md:pt-4">
		<h1
			class="text-[2rem] leading-tight font-medium tracking-[-0.025em] text-primary md:text-[2.5rem]"
		>
			Genres
		</h1>
		<p class="mt-2 text-sm leading-[1.6] text-muted-foreground">
			{data.genres.length}
			{data.genres.length === 1 ? 'genre' : 'genres'}
		</p>
	</header>

	<Item.Group class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
		{#each data.genres as genre (genre.slug)}
			<Item.Root variant="outline">
				{#snippet child({ props })}
					<a {...props} href="/games?genre={genre.slug}">
						<Item.Media variant="icon"><TagIcon aria-hidden="true" /></Item.Media>
						<Item.Content>
							<Item.Title>{genre.name}</Item.Title>
							<Item.Description
								>{genre.count} {genre.count === 1 ? 'game' : 'games'}</Item.Description
							>
						</Item.Content>
						<Item.Actions
							><ChevronRightIcon
								class="size-4 text-muted-foreground"
								aria-hidden="true"
							/></Item.Actions
						>
					</a>
				{/snippet}
			</Item.Root>
		{/each}
	</Item.Group>
</div>
