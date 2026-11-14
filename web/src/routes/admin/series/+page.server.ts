import { fail, redirect } from '@sveltejs/kit';
import { adminErrorMessage } from '#lib/features/admin/game-form.js';
import { adminApi, requireStaff, type AdminSeriesRow } from '#lib/server/admin.js';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	requireStaff(event.locals);
	const result = await adminApi<AdminSeriesRow[]>(event, '/series');
	return { series: result.ok ? result.data : [], failed: !result.ok };
};

export const actions: Actions = {
	create: async (event) => {
		requireStaff(event.locals);
		const form = await event.request.formData();
		const name = String(form.get('name') ?? '').trim();
		if (!name) return fail(400, { error: adminErrorMessage('invalid_series') });
		const result = await adminApi<{ id: string }>(event, '/series', {
			method: 'POST',
			body: { name }
		});
		if (!result.ok) return fail(result.status, { error: adminErrorMessage(result.error) });
		redirect(303, `/admin/series/${result.data.id}`);
	}
};
