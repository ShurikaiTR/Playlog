// Shapes and Turkish copy for the account page (/settings).

import { providerLabels, type Provider } from '#lib/features/auth/providers.js';

export type { Provider };

export type Account = {
	id: string;
	username: string;
	display_name: string | null;
	/** Null for accounts created with Steam, which gives no email. */
	email: string | null;
	avatar_url: string | null;
	created_at: string;
	has_password: boolean;
	sign_in_methods: { provider: Provider; created_at: string }[];
	/** Providers whose keys are configured, so they can be connected. */
	available: Record<Provider, boolean>;
};

export type Device = {
	id: string;
	user_agent: string | null;
	ip: string | null;
	created_at: string;
	last_seen_at: string;
	expires_at: string;
	current: boolean;
};

// In the order the account page lists them.
export const providers: { id: Provider; label: string }[] = (
	['apple', 'google', 'discord', 'steam'] as const
).map((id) => ({ id, label: providerLabels[id] }));

export const providerLabel = (id: Provider) => providerLabels[id] ?? id;

export const providerSignInLabel = (id: Provider) => `Sign in with ${providerLabel(id)}`;

const messages: Record<string, string> = {
	invalid_username: 'Usernames are 3 to 20 characters: lowercase letters, numbers and underscores.',
	reserved_username: 'That username isn’t available. Try another one.',
	username_taken: 'That username is taken. Try another one.',
	invalid_display_name: 'Your display name can be up to 40 characters.',
	invalid_password: 'Your current password is incorrect.',
	session_not_fresh:
		'For your security, sign out and back in, then try deleting your account again.',
	last_sign_in_method: 'This is your only sign-in method. Add another one before you continue.',
	last_admin: 'The last admin account can’t be deleted. Give someone else admin access first.',
	email_taken: 'That email is used by another account. Try a different address.',
	invalid_email: 'Enter a valid email address.',
	invalid_code: 'That code is wrong or has expired. You can ask for a new one.',
	code_cooldown: 'Wait a minute before asking for a new code.',
	provider_already_linked: 'This account is already connected.',
	provider_linked_elsewhere: 'This account is connected to another Playthru account.',
	not_linked: 'This account isn’t connected.',
	session_not_found: 'This device is already signed out.',
	access_denied: 'Connecting was cancelled.',
	invalid_state: 'The connection timed out. Please try again.',
	oauth_failed: 'The account couldn’t be connected. Please try again.',
	unknown_provider: 'This sign-in method isn’t available right now.',
	rate_limited: 'Too many attempts. Wait a bit and try again.',
	unauthenticated: 'You’ve been signed out. Please sign in again.',
	unavailable: 'We can’t reach the server right now. Try again in a moment.'
};

export function accountErrorMessage(code: string | undefined) {
	return (code && messages[code]) || 'Something went wrong. Please try again.';
}

/** A field-specific message for the new-password check, which shares a code with a wrong current password. */
export const NEW_PASSWORD_HINT = 'Passwords are 8 to 128 characters.';

/** Calls the account endpoints under /api/account; failures come back as an error code. */
export async function accountRequest(
	method: 'POST' | 'PUT' | 'DELETE',
	path: string,
	body?: unknown
): Promise<{ ok: true } | { ok: false; error: string }> {
	try {
		const response = await fetch(`/api/account${path}`, {
			method,
			headers: body === undefined ? undefined : { 'content-type': 'application/json' },
			body: body === undefined ? undefined : JSON.stringify(body)
		});
		if (response.ok) return { ok: true };
		const text = await response.text();
		const data = text ? JSON.parse(text) : null;
		return { ok: false, error: typeof data?.error === 'string' ? data.error : 'unknown' };
	} catch {
		return { ok: false, error: 'unavailable' };
	}
}

const systems: [RegExp, string][] = [
	[/iPhone/, 'iOS'],
	[/iPad/, 'iPadOS'],
	[/Android/, 'Android'],
	[/Mac OS X|Macintosh/, 'macOS'],
	[/Windows/, 'Windows'],
	[/CrOS/, 'ChromeOS'],
	[/Linux/, 'Linux']
];
// Order matters: Edge and Opera also say Chrome, and Chrome also says Safari.
const browsers: [RegExp, string][] = [
	[/Edg\//, 'Edge'],
	[/OPR\//, 'Opera'],
	[/Firefox\/|FxiOS/, 'Firefox'],
	[/Chrome\/|CriOS/, 'Chrome'],
	[/Safari\//, 'Safari']
];

/** Browser, operating system and form factor, read from the browser's user agent. */
export function describeDevice(userAgent: string | null) {
	const ua = userAgent ?? '';
	const os = systems.find(([pattern]) => pattern.test(ua))?.[1] ?? null;
	const browser = browsers.find(([pattern]) => pattern.test(ua))?.[1] ?? null;
	const kind: 'phone' | 'tablet' | 'computer' =
		/iPad/.test(ua) || (/Android/.test(ua) && !/Mobile/.test(ua))
			? 'tablet'
			: /iPhone|Mobile/.test(ua)
				? 'phone'
				: 'computer';
	return {
		/** "Safari · macOS" style name for lists. */
		name: [browser, os].filter(Boolean).join(' · ') || 'Unknown device',
		browser: browser ?? 'Unknown browser',
		os: os ?? 'Bilinmeyen sistem',
		kind
	};
}

const dateFormat = new Intl.DateTimeFormat('en', { dateStyle: 'long' });
const dateTimeFormat = new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeStyle: 'short' });

export const formatDate = (iso: string) => dateFormat.format(new Date(iso));
export const formatDateTime = (iso: string) => dateTimeFormat.format(new Date(iso));
const mediumDateFormat = new Intl.DateTimeFormat('en', { dateStyle: 'medium' });
export const formatMediumDate = (iso: string) => mediumDateFormat.format(new Date(iso));
