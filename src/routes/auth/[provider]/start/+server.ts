import { redirect } from '@sveltejs/kit';
import { api } from '#lib/server/api.js';
import { PROVIDER_ORIGINS, providerParam } from '#lib/server/oauth.js';
import { OAUTH_STATE_COOKIE, setShortCookie } from '#lib/server/session.js';
import type { RequestHandler } from './$types';

// "Google/Discord/Apple ile devam et" lands here and is sent on to the provider.
export const GET: RequestHandler = async (event) => {
	const provider = providerParam(event.params.provider);
	if (event.locals.user) redirect(303, '/');

	const result = await api<{ authorization_url: string; state_token: string }>(
		event,
		`/v1/auth/oauth/${provider}/start`,
		{ body: {} }
	);
	if (!result.ok) redirect(303, `/giris?hata=${encodeURIComponent(result.error)}`);

	setShortCookie(event.cookies, OAUTH_STATE_COOKIE, result.data.state_token, 600, {
		crossSitePost: provider === 'apple'
	});
	redirect(302, result.data.authorization_url, { external: PROVIDER_ORIGINS });
};
