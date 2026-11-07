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

// Short-lived cookies for Google/Discord/Apple sign-in: the state token while the visitor is at the
// provider, then the pending sign-up while they pick a username.
const prefixed = (name: string) => (dev ? `playthru_${name}` : `__Host-playthru_${name}`);
export const OAUTH_STATE_COOKIE = prefixed('oauth');
export const OAUTH_PENDING_COOKIE = prefixed('oauth_pending');

export type PendingSignup = {
	pending_token: string;
	email: string;
	suggested_username: string | null;
	display_name: string | null;
};

export function setShortCookie(
	cookies: Cookies,
	name: string,
	value: string,
	maxAge: number,
	{ crossSitePost = false } = {}
) {
	// Apple returns with a cross-site form POST, which never carries SameSite=Lax cookies. Its
	// state cookie is SameSite=None (which requires Secure; browsers allow that on localhost too).
	// It is still bound to the signed state and expires in minutes.
	cookies.set(name, value, {
		path: '/',
		httpOnly: true,
		secure: crossSitePost || !dev,
		sameSite: crossSitePost ? 'none' : 'lax',
		maxAge
	});
}

export function clearCookie(cookies: Cookies, name: string) {
	cookies.delete(name, { path: '/', secure: !dev });
}

export function readPendingSignup(cookies: Cookies): PendingSignup | null {
	const raw = cookies.get(OAUTH_PENDING_COOKIE);
	if (!raw) return null;
	try {
		const value = JSON.parse(raw) as PendingSignup;
		return typeof value.pending_token === 'string' ? value : null;
	} catch {
		return null;
	}
}
