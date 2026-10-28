<script lang="ts">
	import HomeHero from '#lib/components/home-hero.svelte';
	import GameRow from '#lib/components/game-row.svelte';
	import { heroSlides } from '#lib/hero-slides.js';
	import { trendingGames, upcomingGames } from '#lib/games.js';
</script>

<HomeHero slides={heroSlides} />
<GameRow title="Şu an trend" href="/kesfet/populer" games={trendingGames} />
<GameRow title="Yakında çıkacaklar" href="/kesfet/yeni" games={upcomingGames} />
