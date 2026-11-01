<script lang="ts">
	import * as Item from "#lib/components/ui/item/index.js";
	import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
	import TagIcon from "@lucide/svelte/icons/tag";

	let { data } = $props();
</script>

<svelte:head>
	<title>Türler | Playlog</title>
	<meta name="description" content="Oyunlara türlerine göre göz at." />
</svelte:head>

<div lang="tr" class="min-w-0 px-4 pb-20 md:px-8 md:pb-24">
	<header class="pt-2 pb-6 md:pt-4">
		<h1 class="text-primary text-[2rem] leading-tight font-medium tracking-[-0.025em] md:text-[2.5rem]">Türler</h1>
		<p class="text-muted-foreground mt-2 text-sm leading-[1.6]">{data.genres.length} tür</p>
	</header>

	<Item.Group class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
		{#each data.genres as genre (genre.name)}
			<Item.Root variant="outline">
				{#snippet child({ props })}
					<a {...props} href="/oyunlar?tur={encodeURIComponent(genre.name)}">
						<Item.Media variant="icon"><TagIcon aria-hidden="true" /></Item.Media>
						<Item.Content>
							<Item.Title>{genre.name}</Item.Title>
							<Item.Description>{genre.count} oyun</Item.Description>
						</Item.Content>
						<Item.Actions><ChevronRightIcon class="text-muted-foreground size-4" aria-hidden="true" /></Item.Actions>
					</a>
				{/snippet}
			</Item.Root>
		{/each}
	</Item.Group>
</div>
