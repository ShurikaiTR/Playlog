import { json, type RequestEvent } from '@sveltejs/kit';
import { api } from '#lib/server/api.js';
import { clearSessionCookie } from '#lib/server/session.js';
import type { RequestHandler } from './$types';

// The /settings dialogs call these, and they forward to the Rust API's /v1/account with the session
// token. Only the listed calls pass; connecting a provider is a browser redirect under /auth and
// sessions are form actions on /settings/sessions. Other sites can't call these: a JSON body or a
// PUT/DELETE needs a CORS preflight this app never allows.
const allowed: Record<'POST' | 'PUT' | 'DELETE', RegExp> = {
	POST: /^email-code$/,
	PUT: /^(profile|password)$/,
	DELETE: /^(|sign-in-methods\/(google|discord|apple))$/
};

async function forward(event: RequestEvent, method: 'POST' | 'PUT' | 'DELETE') {
	const path = (event.params as { path: string }).path;
	if (!allowed[method].test(path)) return json({ error: 'not_found' }, { status: 404 });
	const token = event.locals.token;
	if (!token) return json({ error: 'unauthenticated' }, { status: 401 });

	const text = await event.request.text();
	if (text.length > 10_000) return json({ error: 'too_large' }, { status: 413 });
	let body: unknown;
	try {
		body = text ? JSON.parse(text) : {};
	} catch {
		return json({ error: 'invalid_body' }, { status: 400 });
	}

	const result = await api(event, `/v1/account${path ? `/${path}` : ''}`, { method, body, token });
	if (!result.ok) {
		if (result.status === 401 && result.error === 'unauthenticated') {
			clearSessionCookie(event.cookies);
		}
		return json({ error: result.error }, { status: result.status });
	}

	// The account was deleted: its sessions are gone, so is the cookie.
	if (method === 'DELETE' && path === '') clearSessionCookie(event.cookies);
	return new Response(null, { status: 204 });
}

export const POST: RequestHandler = (event) => forward(event, 'POST');
export const PUT: RequestHandler = (event) => forward(event, 'PUT');
export const DELETE: RequestHandler = (event) => forward(event, 'DELETE');
