import { fail } from '@sveltejs/kit';
import { adminErrorMessage } from '#lib/features/admin/game-form.js';
import { adminApi, requireStaff, type AdminGameRow } from '#lib/server/admin.js';
import type { Actions, PageServerLoad } from './$types';

const PER_PAGE = 25;
const STATUSES = ['draft', 'published', 'hidden'];

export const load: PageServerLoad = async (event) => {
	const { user } = requireStaff(event.locals);
	const params = event.url.searchParams;
	const q = params.get('q')?.trim().slice(0, 100) ?? '';
	const status = STATUSES.includes(params.get('status') ?? '') ? params.get('status')! : 'all';
	const page = Math.max(1, Math.floor(Number(params.get('page'))) || 1);

	const query = new URLSearchParams({
		limit: String(PER_PAGE),
		offset: String((page - 1) * PER_PAGE)
	});
	if (q) query.set('q', q);
	if (status !== 'all') query.set('status', status);
	const result = await adminApi<{ total: number; items: AdminGameRow[] }>(event, `/games?${query}`);

	return {
		games: result.ok ? result.data.items : [],
		total: result.ok ? result.data.total : 0,
		failed: !result.ok,
		q,
		status,
		page,
		perPage: PER_PAGE,
		canDelete: user.role === 'admin'
	};
};

export const actions: Actions = {
	// Deleting a game also removes its library entries, ratings and reviews, so it is admin only.
	delete: async (event) => {
		requireStaff(event.locals, 'admin');
		const form = await event.request.formData();
		const id = form.get('id');
		if (typeof id !== 'string' || !id)
			return fail(400, { error: adminErrorMessage('game_not_found') });
		const result = await adminApi(event, `/games/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!result.ok) return fail(result.status, { error: adminErrorMessage(result.error) });
		return { deleted: true };
	}
};
