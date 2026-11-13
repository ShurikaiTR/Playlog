import { toHeroSlides } from '#lib/features/home/hero-slides.js';
import { cardMeta, listGames, releaseMeta } from '#lib/server/catalog.js';
import type { PageServerLoad } from './$types';

// Twelve fills every row at each breakpoint of the six, four, three and two column grids.
const ROW = 12;

export const load: PageServerLoad = async (event) => {
	const [released, upcoming] = await Promise.all([
		listGames(event, { window: 'released', sort: 'newest', limit: 24 }),
		listGames(event, { window: 'upcoming', sort: 'soonest', limit: ROW })
	]);
	return {
		slides: toHeroSlides(released.games),
		newReleases: released.games.slice(0, ROW).map((game) => ({ ...game, meta: cardMeta(game) })),
		upcoming: upcoming.games.map((game) => ({ ...game, meta: releaseMeta(game) })),
		unavailable: !released.ok
	};
};
