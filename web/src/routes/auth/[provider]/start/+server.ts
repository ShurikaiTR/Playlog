import { redirect } from '@sveltejs/kit';
import { api } from '#lib/server/api.js';
import {
	linkResultPath,
	PROVIDER_ORIGINS,
	providerParam,
	type Provider
} from '#lib/server/oauth.js';
import {
	clearCookie,
	OAUTH_LINK_COOKIE,
	OAUTH_RETURN_COOKIE,
	OAUTH_STATE_COOKIE,
	safeReturnPath,
	setShortCookie
} from '#lib/server/session.js';
import type { RequestHandler } from './$types';

// "Google/Discord/Apple ile devam et" lands here and is sent on to the provider.
// With `?link`, a signed-in user connects the provider to their account from settings instead.
export const GET: RequestHandler = async (event) => {
	const provider = providerParam(event.params.provider);
	if (event.url.searchParams.has('link')) return startLink(event, provider);
	const returnTo = safeReturnPath(event.url.searchParams.get('return_to'));
	if (event.locals.user) redirect(303, returnTo);
	clearCookie(event.cookies, OAUTH_LINK_COOKIE);

	const result = await api<{ authorization_url: string; state_token: string }>(
		event,
		`/v1/auth/oauth/${provider}/start`,
		{ body: {} }
	);
	if (!result.ok) redirect(303, `/sign-in?error=${encodeURIComponent(result.error)}`);

	setShortCookie(event.cookies, OAUTH_STATE_COOKIE, result.data.state_token, 600, {
		crossSitePost: provider === 'apple'
	});
	setShortCookie(event.cookies, OAUTH_RETURN_COOKIE, returnTo, 600, {
		crossSitePost: provider === 'apple'
	});
	redirect(302, result.data.authorization_url, { external: PROVIDER_ORIGINS });
};

async function startLink(event: Parameters<RequestHandler>[0], provider: Provider): Promise<never> {
	const back = safeReturnPath(event.url.searchParams.get('return_to'));
	if (!event.locals.token) redirect(303, '/sign-in');

	const result = await api<{ authorization_url: string; state_token: string }>(
		event,
		`/v1/account/sign-in-methods/${provider}/start`,
		{ body: {}, token: event.locals.token }
	);
	if (!result.ok) redirect(303, linkResultPath(back, { error: result.error }));

	const crossSitePost = provider === 'apple';
	setShortCookie(event.cookies, OAUTH_STATE_COOKIE, result.data.state_token, 600, {
		crossSitePost
	});
	setShortCookie(event.cookies, OAUTH_LINK_COOKIE, back, 600, { crossSitePost });
	redirect(302, result.data.authorization_url, { external: PROVIDER_ORIGINS });
}
