import { redirect } from '@sveltejs/kit';
import { api } from '#lib/server/api.js';
import { clearSessionCookie } from '#lib/server/session.js';
import type { RequestHandler } from './$types';

// "Çıkış yap" posts here. The cookie goes even if the API can't be reached.
export const POST: RequestHandler = async (event) => {
	if (event.locals.token) {
		await api(event, '/v1/auth/logout', { token: event.locals.token });
	}
	clearSessionCookie(event.cookies);
	redirect(303, '/');
};
