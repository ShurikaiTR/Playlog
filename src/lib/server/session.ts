import type { Cookies } from '@sveltejs/kit';
import { dev } from '$app/env';

// `__Host-` cookies must be Secure, which plain http://localhost can't set in every browser,
// so development uses an unprefixed name.
export const SESSION_COOKIE = dev ? 'playthru_session' : '__Host-playthru_session';

// The API enforces its own idle (30 days) and absolute (90 days) limits.
const MAX_AGE = 60 * 60 * 24 * 30;

export function setSessionCookie(cookies: Cookies, token: string) {
	cookies.set(SESSION_COOKIE, token, {
		path: '/',
		httpOnly: true,
		secure: !dev,
		sameSite: 'lax',
		maxAge: MAX_AGE
	});
}

export function clearSessionCookie(cookies: Cookies) {
	cookies.delete(SESSION_COOKIE, { path: '/', secure: !dev });
}
