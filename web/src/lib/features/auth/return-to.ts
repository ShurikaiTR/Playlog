/**
 * A same-origin path to come back to (e.g. `return_to`); anything else falls back to `/`.
 * Rejects `//host`, backslashes and control characters (browsers strip tabs and newlines, so
 * `/\t/evil.com` would become `//evil.com`), re-parses the value to confirm it stays on this
 * origin, and never returns to an auth page, which would loop.
 */
export function safeReturnPath(raw: string | null | undefined): string {
	if (!raw || raw.length > 300 || !raw.startsWith('/') || raw.startsWith('//')) return '/';
	if (/[\\\u0000-\u001f\u007f]/.test(raw)) return '/';
	try {
		const url = new URL(raw, 'http://localhost');
		if (url.origin !== 'http://localhost') return '/';
		if (/^\/(sign-in|sign-up|auth)(\/|$)/.test(url.pathname)) return '/';
		return url.pathname + url.search + url.hash;
	} catch {
		return '/';
	}
}

/** The sign-in or sign-up URL, carrying `return_to` unless it is just the home page. */
export function authHref(kind: 'sign-in' | 'sign-up', returnTo: string): string {
	return returnTo === '/' ? `/${kind}` : `/${kind}?return_to=${encodeURIComponent(returnTo)}`;
}

export type AuthKind = 'sign-in' | 'sign-up';
