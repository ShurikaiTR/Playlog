import { redirect } from '@sveltejs/kit';
import type { Handle } from '@sveltejs/kit/hooks';
import { api, type ApiUser } from '#lib/server/api.js';
import { clearSessionCookie, SESSION_COOKIE } from '#lib/server/session.js';

// The sign-in and sign-up pages used to live at Turkish paths; old links keep working.
const MOVED_PATHS: Record<string, string> = { '/giris': '/sign-in', '/kayit-ol': '/sign-up' };

export const handle: Handle = async ({ event, resolve }) => {
	const moved = MOVED_PATHS[event.url.pathname];
	if (moved && (event.request.method === 'GET' || event.request.method === 'HEAD')) {
		redirect(308, moved + event.url.search);
	}

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
