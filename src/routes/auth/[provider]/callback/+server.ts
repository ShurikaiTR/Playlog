import { redirect } from '@sveltejs/kit';
import { api, type ApiUser } from '#lib/server/api.js';
import { providerParam } from '#lib/server/oauth.js';
import {
	clearCookie,
	OAUTH_PENDING_COOKIE,
	OAUTH_STATE_COOKIE,
	setSessionCookie,
	setShortCookie,
	type PendingSignup
} from '#lib/server/session.js';
import type { RequestHandler } from './$types';

type CallbackResult =
	| { status: 'signed_in'; token: string; user: ApiUser }
	| ({ status: 'username_required' } & PendingSignup);

const backToLogin = (code: string) => `/giris?hata=${encodeURIComponent(code)}`;

// The provider sends the visitor back here with a one-time code.
export const GET: RequestHandler = async (event) => {
	const provider = providerParam(event.params.provider);
	const { searchParams } = event.url;
	const stateToken = event.cookies.get(OAUTH_STATE_COOKIE);
	clearCookie(event.cookies, OAUTH_STATE_COOKIE);

	// The visitor said no at the provider (or the provider failed before asking).
	if (searchParams.get('error')) redirect(303, backToLogin('access_denied'));

	const code = searchParams.get('code');
	const state = searchParams.get('state');
	if (!code || !state || !stateToken) redirect(303, backToLogin('invalid_state'));

	const result = await api<CallbackResult>(event, `/v1/auth/oauth/${provider}/callback`, {
		body: { code, state, state_token: stateToken }
	});
	if (!result.ok) redirect(303, backToLogin(result.error));

	if (result.data.status === 'signed_in') {
		setSessionCookie(event.cookies, result.data.token);
		redirect(303, '/');
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
};
