<script lang="ts">
	import HomeHero from '#lib/features/home/home-hero.svelte';
	import GameRow from '#lib/features/games/game-row.svelte';
	import { heroSlides } from '#lib/features/home/hero-slides.js';
	import { trendingGames, upcomingGames } from '#lib/features/games/games.js';
</script>

<HomeHero slides={heroSlides} />
<GameRow title="Şu an trend" href="/kesfet/populer" games={trendingGames} />
<GameRow title="Yakında çıkacaklar" href="/kesfet/yeni" games={upcomingGames} />
