import type { Handle } from '@sveltejs/kit/hooks';
import { api, type ApiUser } from '#lib/server/api.js';
import { clearSessionCookie, SESSION_COOKIE } from '#lib/server/session.js';

// The site's addresses used to be Turkish. Old links and bookmarks move to the new ones for good.
// Longer prefixes come first so `/hesap/oturumlar` wins over `/hesap`.
const MOVED: [from: string, to: string][] = [
	['/hesap/veri/indir', '/settings/data/export'],
	['/hesap/veri', '/settings/data'],
	['/hesap/oturumlar', '/settings/sessions'],
	['/hesap/guvenlik', '/settings/security'],
	['/hesap', '/settings'],
	['/kesfet/turler', '/genres'],
	['/kesfet/populer', '/games'],
	['/kesfet/yeni', '/games'],
	['/kesfet', '/discover'],
	['/oyunlar', '/games'],
	['/oyun', '/games'],
	['/favoriler', '/favorites'],
	['/planlananlar', '/backlog'],
	['/sonra-oyna', '/backlog'],
	['/liste', '/lists'],
	['/profil', '/u']
];

/** The new address for an old Turkish path, or null when the path was never moved. */
function movedTo(pathname: string): string | null {
	for (const [from, to] of MOVED) {
		if (pathname === from || pathname.startsWith(`${from}/`))
			return to + pathname.slice(from.length);
	}
	return null;
}

export const handle: Handle = async ({ event, resolve }) => {
	if (event.request.method === 'GET') {
		const moved = movedTo(event.url.pathname);
		if (moved)
			return new Response(null, { status: 308, headers: { location: moved + event.url.search } });
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
