import { redirect } from '@sveltejs/kit';
import { finishLink, providerParam, type LinkReturn } from '#lib/server/oauth.js';
import { clearCookie, OAUTH_LINK_RETURN_COOKIE, safeReturnPath } from '#lib/server/session.js';
import type { RequestHandler } from './$types';

// Second half of connecting Apple from settings: Apple's cross-site POST had no session cookie,
// so the callback parked the code and sent the browser here, a same-site GET that carries it.
export const GET: RequestHandler = async (event) => {
	const provider = providerParam(event.params.provider);
	const raw = event.cookies.get(OAUTH_LINK_RETURN_COOKIE);
	clearCookie(event.cookies, OAUTH_LINK_RETURN_COOKIE);
	if (!raw) redirect(303, '/');

	let link: LinkReturn;
	try {
		link = JSON.parse(raw) as LinkReturn;
	} catch {
		redirect(303, '/');
	}
	return finishLink(event, provider, { ...link, back: safeReturnPath(link.back) });
};
