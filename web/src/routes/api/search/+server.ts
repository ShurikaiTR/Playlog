import { json } from '@sveltejs/kit';
import type { GameResult } from '#lib/features/search/search.js';
import { listGames } from '#lib/server/catalog.js';
import type { RequestHandler } from './$types';

// The search dialog's results. Without a query it suggests the newest releases.
export const GET: RequestHandler = async (event) => {
	const query = (event.url.searchParams.get('q') ?? '').trim().slice(0, 100);
	const { games } = await listGames(
		event,
		query ? { q: query, limit: 20 } : { window: 'released', sort: 'newest', limit: 6 }
	);
	const results: GameResult[] = games.map((game) => ({
		slug: game.slug,
		title: game.title,
		cover: game.cover,
		backdrop: game.backdrop,
		genres: game.genres,
		summary: game.summary,
		developer: game.developers.join(', '),
		year: game.releaseDate?.slice(0, 4) ?? ''
	}));
	return json(results, { headers: { 'cache-control': 'private, max-age=30' } });
};
