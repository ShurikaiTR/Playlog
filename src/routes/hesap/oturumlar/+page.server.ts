import { fail, redirect } from '@sveltejs/kit';
import { api } from '#lib/server/api.js';
import { clearSessionCookie } from '#lib/server/session.js';
import type { Actions } from './$types';

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

export const actions: Actions = {
	// Signs one other device out. This device signs out with "Çıkış yap".
	revoke: async (event) => {
		const token = event.locals.token;
		if (!token) redirect(303, '/sign-in');
		const id = String((await event.request.formData()).get('id') ?? '');
		if (!UUID.test(id)) return fail(404, { error: 'session_not_found' });
		const result = await api(event, `/v1/account/sessions/${id}`, { method: 'DELETE', token });
		if (!result.ok && result.error !== 'session_not_found') {
			return fail(result.status, { error: result.error });
		}
		redirect(303, '/hesap/oturumlar');
	},
	// Signs out everywhere, this device included.
	revokeAll: async (event) => {
		const token = event.locals.token;
		if (!token) redirect(303, '/sign-in');
		const others = await api(event, '/v1/account/sessions', { method: 'DELETE', token });
		if (!others.ok) return fail(others.status, { error: others.error });
		await api(event, '/v1/auth/logout', { token });
		clearSessionCookie(event.cookies);
		redirect(303, '/');
	}
};
