import { error, fail, redirect } from '@sveltejs/kit';
import { adminErrorMessage } from '#lib/features/admin/game-form.js';
import { adminApi, requireStaff, type AdminSeries } from '#lib/server/admin.js';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const { user } = requireStaff(event.locals);
	const result = await adminApi<AdminSeries>(
		event,
		`/series/${encodeURIComponent(event.params.id)}`
	);
	if (!result.ok) error(result.status === 404 ? 404 : 503, adminErrorMessage(result.error));
	return { series: result.data, canDelete: user.role === 'admin' };
};

export const actions: Actions = {
	// The whole list, its order and the notes go in one call: what the editor shows is what is saved.
	save: async (event) => {
		requireStaff(event.locals);
		const form = await event.request.formData();
		const name = String(form.get('name') ?? '').trim();
		let games: { game_id: string; note: string | null }[];
		try {
			const parsed = JSON.parse(String(form.get('games') ?? '[]')) as {
				id: string;
				note: string;
			}[];
			games = parsed.map((game) => ({ game_id: game.id, note: game.note?.trim() || null }));
		} catch {
			return fail(400, { error: adminErrorMessage('invalid_game') });
		}
		const id = encodeURIComponent(event.params.id);
		if (name) {
			const renamed = await adminApi(event, `/series/${id}`, { method: 'PATCH', body: { name } });
			if (!renamed.ok) return fail(renamed.status, { error: adminErrorMessage(renamed.error) });
		}
		const saved = await adminApi(event, `/series/${id}/games`, {
			method: 'PUT',
			body: { ordered: form.get('ordered') === 'on', games }
		});
		if (!saved.ok) return fail(saved.status, { error: adminErrorMessage(saved.error) });
		return { saved: true };
	},
	// Deleting a series only takes its games out of it; the games stay.
	delete: async (event) => {
		requireStaff(event.locals, 'admin');
		const result = await adminApi(event, `/series/${encodeURIComponent(event.params.id)}`, {
			method: 'DELETE'
		});
		if (!result.ok) return fail(result.status, { error: adminErrorMessage(result.error) });
		redirect(303, '/admin/series');
	}
};
