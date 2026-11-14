import { json } from '@sveltejs/kit';
import { adminApi, requireStaff, type AdminGameRow } from '#lib/server/admin.js';
import type { RequestHandler } from './$types';

// Game search for the series editor: the first few matches by title or slug.
export const GET: RequestHandler = async (event) => {
	requireStaff(event.locals);
	const q = event.url.searchParams.get('q')?.trim().slice(0, 100) ?? '';
	if (q.length < 2) return json({ games: [] });
	const query = new URLSearchParams({ q, limit: '8' });
	const result = await adminApi<{ items: AdminGameRow[] }>(event, `/games?${query}`);
	if (!result.ok) return json({ games: [] }, { status: result.status });
	return json({
		games: result.data.items
			.filter((game) => !['dlc', 'expansion', 'bundle'].includes(game.kind))
			.map(({ id, slug, title, kind, status, cover_url, release_date, release_precision }) => ({
				id,
				slug,
				title,
				kind,
				status,
				cover_url,
				release_date,
				release_precision,
				note: null
			}))
	});
};
