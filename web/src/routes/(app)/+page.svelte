<script lang="ts">
	import HomeHero from '#lib/features/home/home-hero.svelte';
	import GameRow from '#lib/features/games/game-row.svelte';

	let { data } = $props();
</script>

<svelte:head>
	<title>Playthru: track the games you play</title>
	<meta
		name="description"
		content="Discover games, keep track of what you play and see what is coming next."
	/>
</svelte:head>

{#if data.slides.length}
	<HomeHero slides={data.slides} />
{/if}

{#if data.newReleases.length}
	<GameRow title="New releases" href="/games" games={data.newReleases} />
{/if}
{#if data.upcoming.length}
	<GameRow title="Coming soon" href="/games?upcoming=1" games={data.upcoming} />
{/if}

{#if !data.newReleases.length && !data.upcoming.length}
	<div class="flex min-h-[60svh] flex-col items-center justify-center gap-2 px-6 text-center">
		<h1 class="text-title-2 text-primary">
			{data.unavailable ? 'We can’t reach the catalog right now' : 'The catalog is empty'}
		</h1>
		<p class="text-sm text-muted-foreground">
			{data.unavailable ? 'Try again in a moment.' : 'Games will show up here once they are added.'}
		</p>
	</div>
{/if}
