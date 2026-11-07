import { error, redirect, type RequestEvent } from '@sveltejs/kit';
import { api } from './api.js';

export type Provider = 'google' | 'discord' | 'apple';

export function providerParam(value: string): Provider {
	if (value === 'google' || value === 'discord' || value === 'apple') return value;
	error(404, 'Sayfa bulunamadı');
}

// The only places the start route may send a visitor.
export const PROVIDER_ORIGINS = [
	'https://accounts.google.com',
	'https://discord.com',
	'https://appleid.apple.com'
];

/**
 * Apple sends the visitor's name only on the first sign-in, as a JSON `user` form field
 * (`{"name":{"firstName":"…","lastName":"…"}}`). It is unverified and used only as a display
 * name suggestion; anything malformed is ignored.
 */
export function appleDisplayName(raw: FormDataEntryValue | null): string | null {
	if (typeof raw !== 'string' || raw.length > 1000) return null;
	try {
		const name = (JSON.parse(raw) as { name?: { firstName?: unknown; lastName?: unknown } }).name;
		const parts = [name?.firstName, name?.lastName].filter(
			(part): part is string => typeof part === 'string'
		);
		return parts.join(' ').trim() || null;
	} catch {
		return null;
	}
}

export type EnabledProviders = Record<Provider, boolean>;

/**
 * Which sign-in providers have keys configured on the API, so pages only offer those. If the API
 * can't be reached none are shown: signing in through them would fail anyway.
 */
export async function enabledProviders(
	event: Pick<RequestEvent, 'request' | 'getClientAddress'>
): Promise<EnabledProviders> {
	const result = await api<EnabledProviders>(event, '/v1/auth/oauth/providers', { method: 'GET' });
	return result.ok
		? { google: result.data.google, discord: result.data.discord, apple: result.data.apple }
		: { google: false, discord: false, apple: false };
}

/**
 * Where a connect-from-the-account-page flow ends: back on /hesap (or the page it started from)
 * with the outcome in the query, `baglandi=<provider>` or `hata=<code>`.
 */
export function linkResultPath(
	back: string,
	outcome: { linked: Provider } | { error: string }
): string {
	const url = new URL(back, 'http://x');
	if ('linked' in outcome) url.searchParams.set('baglandi', outcome.linked);
	else url.searchParams.set('hata', outcome.error);
	return url.pathname + url.search;
}

export type LinkReturn = {
	back: string;
	code: string | null;
	state: string | null;
	stateToken: string | null;
	error: string | null;
};

/** Ends a connect-from-settings flow; always redirects back to the page it started from. */
export async function finishLink(
	event: RequestEvent,
	provider: Provider,
	link: LinkReturn
): Promise<never> {
	if (link.error) redirect(303, linkResultPath(link.back, { error: 'access_denied' }));
	if (!event.locals.token) redirect(303, '/sign-in');
	if (!link.code || !link.state || !link.stateToken) {
		redirect(303, linkResultPath(link.back, { error: 'invalid_state' }));
	}
	const result = await api(event, `/v1/account/sign-in-methods/${provider}/callback`, {
		body: { code: link.code, state: link.state, state_token: link.stateToken },
		token: event.locals.token
	});
	redirect(
		303,
		linkResultPath(link.back, result.ok ? { linked: provider } : { error: result.error })
	);
}
