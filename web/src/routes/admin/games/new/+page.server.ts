import { fail, redirect } from '@sveltejs/kit';
import { adminErrorMessage, emptyGameForm } from '#lib/features/admin/game-form.js';
import {
	adminApi,
	buildGameInput,
	requireStaff,
	valuesFromForm,
	type PlatformOption
} from '#lib/server/admin.js';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	requireStaff(event.locals);
	const platforms = await adminApi<PlatformOption[]>(event, '/platforms');
	return { platforms: platforms.ok ? platforms.data : [], initial: emptyGameForm() };
};

export const actions: Actions = {
	default: async (event) => {
		requireStaff(event.locals);
		const values = valuesFromForm(await event.request.formData());
		const result = await adminApi<{ id: string; slug: string }>(event, '/games', {
			method: 'POST',
			body: buildGameInput(values, null)
		});
		if (!result.ok) return fail(result.status, { error: adminErrorMessage(result.error) });
		redirect(303, `/admin/games/${result.data.slug}`);
	}
};
