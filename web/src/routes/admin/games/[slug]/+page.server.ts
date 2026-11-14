import { error, fail, redirect } from '@sveltejs/kit';
import { adminErrorMessage } from '#lib/features/admin/game-form.js';
import {
	adminApi,
	buildGameInput,
	requireStaff,
	valuesFromDetail,
	valuesFromForm,
	type AdminGameDetail,
	type PlatformOption
} from '#lib/server/admin.js';
import type { Actions, PageServerLoad } from './$types';

const load_game = (event: Parameters<typeof adminApi>[0], slug: string) =>
	adminApi<AdminGameDetail>(event, `/games/${encodeURIComponent(slug)}`);

export const load: PageServerLoad = async (event) => {
	const { user } = requireStaff(event.locals);
	const [game, platforms] = await Promise.all([
		load_game(event, event.params.slug),
		adminApi<PlatformOption[]>(event, '/platforms')
	]);
	if (!game.ok)
		error(
			game.status === 404 ? 404 : 503,
			game.status === 404 ? 'Game not found' : 'The API is unavailable'
		);
	// An old address still finds the game; the page then moves to the current one.
	if (game.data.slug !== event.params.slug) redirect(307, `/admin/games/${game.data.slug}`);
	return {
		id: game.data.id,
		slug: game.data.slug,
		title: game.data.title,
		status: game.data.status,
		initial: valuesFromDetail(game.data),
		platforms: platforms.ok ? platforms.data : [],
		canDelete: user.role === 'admin'
	};
};

export const actions: Actions = {
	save: async (event) => {
		requireStaff(event.locals);
		const stored = await load_game(event, event.params.slug);
		if (!stored.ok) return fail(stored.status, { error: adminErrorMessage(stored.error) });
		const values = valuesFromForm(await event.request.formData());
		const result = await adminApi<{ id: string; slug: string }>(
			event,
			`/games/${encodeURIComponent(stored.data.id)}`,
			{ method: 'PUT', body: buildGameInput(values, stored.data) }
		);
		if (!result.ok) return fail(result.status, { error: adminErrorMessage(result.error) });
		// The slug may have changed; the page is reloaded at the current address.
		redirect(303, `/admin/games/${result.data.slug}?saved`);
	},
	delete: async (event) => {
		requireStaff(event.locals, 'admin');
		const stored = await load_game(event, event.params.slug);
		if (!stored.ok) return fail(stored.status, { error: adminErrorMessage(stored.error) });
		const result = await adminApi(event, `/games/${encodeURIComponent(stored.data.id)}`, {
			method: 'DELETE'
		});
		if (!result.ok) return fail(result.status, { error: adminErrorMessage(result.error) });
		redirect(303, '/admin/games');
	}
};
