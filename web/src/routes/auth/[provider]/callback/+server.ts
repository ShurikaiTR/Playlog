import { redirect, type RequestEvent } from '@sveltejs/kit';
import { api, type ApiUser } from '#lib/server/api.js';
import {
	appleDisplayName,
	finishLink,
	providerParam,
	steamReturn,
	type Provider
} from '#lib/server/oauth.js';
import {
	clearCookie,
	OAUTH_LINK_COOKIE,
	OAUTH_LINK_RETURN_COOKIE,
	OAUTH_PENDING_COOKIE,
	OAUTH_RETURN_COOKIE,
	OAUTH_STATE_COOKIE,
	safeReturnPath,
	setSessionCookie,
	setShortCookie,
	type PendingSignup
} from '#lib/server/session.js';
import type { RequestHandler } from './$types';

type CallbackResult =
	| { status: 'signed_in'; token: string; user: ApiUser }
	| ({ status: 'username_required' } & PendingSignup);

type ProviderReturn = {
	code: string | null;
	state: string | null;
	error: string | null;
	displayName?: string | null;
	/** Steam only: the signed `openid.*` fields of the return. */
	openid?: Record<string, string> | null;
};

const backToLogin = (code: string) => `/sign-in?hata=${encodeURIComponent(code)}`;

const field = (value: FormDataEntryValue | string | null) =>
	typeof value === 'string' ? value : null;

// Always ends in a redirect (thrown), so it never returns.
async function finish(
	event: RequestEvent,
	provider: Provider,
	back: ProviderReturn
): Promise<never> {
	const stateToken = event.cookies.get(OAUTH_STATE_COOKIE);
	clearCookie(event.cookies, OAUTH_STATE_COOKIE);
	// Where the visitor was headed; for a brand-new account it stays until the username step.
	const returnTo = safeReturnPath(event.cookies.get(OAUTH_RETURN_COOKIE));

	// Connecting a provider from settings rather than signing in.
	const linkBack = event.cookies.get(OAUTH_LINK_COOKIE);
	if (linkBack !== undefined) {
		clearCookie(event.cookies, OAUTH_LINK_COOKIE);
		const link = {
			back: safeReturnPath(linkBack),
			code: back.code,
			state: back.state,
			stateToken: stateToken ?? null,
			error: back.error,
			openid: back.openid
		};
		// Apple's cross-site POST carries no session cookie: park the code for one same-site GET.
		if (!event.locals.token && provider === 'apple' && !back.error) {
			setShortCookie(event.cookies, OAUTH_LINK_RETURN_COOKIE, JSON.stringify(link), 120);
			redirect(303, `/auth/${provider}/baglan`);
		}
		return finishLink(event, provider, link);
	}

	// The visitor said no at the provider (or the provider failed before asking).
	if (back.error) redirect(303, backToLogin('access_denied'));
	if ((!back.code && !back.openid) || !back.state || !stateToken) {
		redirect(303, backToLogin('invalid_state'));
	}

	const result = await api<CallbackResult>(event, `/v1/auth/oauth/${provider}/callback`, {
		body: {
			code: back.code ?? '',
			state: back.state,
			state_token: stateToken,
			openid: back.openid ?? {},
			display_name: back.displayName ?? null
		}
	});
	if (!result.ok) redirect(303, backToLogin(result.error));

	if (result.data.status === 'signed_in') {
		setSessionCookie(event.cookies, result.data.token);
		clearCookie(event.cookies, OAUTH_RETURN_COOKIE);
		redirect(303, returnTo);
	}

	// New account: the visitor picks a username first.
	const { pending_token, email, suggested_username, display_name } = result.data;
	setShortCookie(
		event.cookies,
		OAUTH_PENDING_COOKIE,
		JSON.stringify({ pending_token, email, suggested_username, display_name }),
		900
	);
	redirect(303, '/auth/tamamla');
}

// Google, Discord and Steam send the visitor back with the code in the query string.
export const GET: RequestHandler = async (event) => {
	const provider = providerParam(event.params.provider);
	if (provider === 'apple') redirect(303, backToLogin('invalid_state'));
	const { searchParams } = event.url;
	// Steam says no with openid.mode=cancel; a missing id_res is treated the same.
	if (provider === 'steam') {
		const openid = steamReturn(searchParams);
		return finish(event, provider, {
			code: null,
			state: searchParams.get('state'),
			error: openid ? null : 'access_denied',
			openid
		});
	}
	return finish(event, provider, {
		code: searchParams.get('code'),
		state: searchParams.get('state'),
		error: searchParams.get('error')
	});
};

// Apple posts the code as a form (response_mode=form_post), the only way it returns name and email.
export const POST: RequestHandler = async (event) => {
	const provider = providerParam(event.params.provider);
	if (provider !== 'apple') redirect(303, backToLogin('invalid_state'));
	const form = await event.request.formData();
	return finish(event, provider, {
		code: field(form.get('code')),
		state: field(form.get('state')),
		error: field(form.get('error')),
		displayName: appleDisplayName(form.get('user'))
	});
};
