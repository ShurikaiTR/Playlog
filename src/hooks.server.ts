import type { Handle } from '@sveltejs/kit/hooks';
import { api, type ApiUser } from '#lib/server/api.js';
import { clearSessionCookie, SESSION_COOKIE } from '#lib/server/session.js';

export const handle: Handle = async ({ event, resolve }) => {
	event.locals.user = null;
	event.locals.token = null;

	const token = event.cookies.get(SESSION_COOKIE);
	if (token) {
		const me = await api<ApiUser>(event, '/v1/auth/me', { method: 'GET', token });
		if (me.ok) {
			event.locals.user = me.data;
			event.locals.token = token;
		} else if (me.status === 401) {
			// Expired or signed out elsewhere. Other failures (API down) keep the cookie for next time.
			clearSessionCookie(event.cookies);
		}
	}

	return resolve(event);
};
