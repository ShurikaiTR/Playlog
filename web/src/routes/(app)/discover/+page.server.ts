import { moods } from '#lib/features/games/games.js';
import { listGames, listTags } from '#lib/server/catalog.js';
import type { PageServerLoad } from './$types';

const MOOD_SIZE = 10;

// Without a chosen mood the page opens on a random one, picked once per request on the server so
// hydration shows the same mood. Only moods the catalog has games for are offered.
export const load: PageServerLoad = async (event) => {
	const genres = new Set((await listTags(event, 'genre')).map((genre) => genre.slug));
	const available = moods.filter((mood) => genres.has(mood.genre));
	const pool = available.length ? available : [...moods];

	const mood =
		pool.find((item) => item.key === event.url.searchParams.get('mood')) ??
		pool[Math.floor(Math.random() * pool.length)];
	const others = pool.filter((item) => item.key !== mood.key);

	const { games } = await listGames(event, {
		genres: [mood.genre],
		window: 'released',
		sort: 'newest',
		limit: MOOD_SIZE
	});

	return {
		mood: {
			key: mood.key,
			title: mood.title,
			subtitle: mood.subtitle,
			// The showcase needs wide artwork; a trailer is optional and not in the catalog yet.
			items: games
				.filter((game) => game.backdrop || game.cover)
				.map((game) => ({ ...game, trailer: null as string | null }))
		},
		// "Surprise me" opens one of the other moods.
		surprise: others.length ? others[Math.floor(Math.random() * others.length)].key : null
	};
};
