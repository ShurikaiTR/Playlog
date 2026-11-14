import { json, type RequestEvent } from '@sveltejs/kit';
import { api } from '#lib/server/api.js';
import { clearSessionCookie } from '#lib/server/session.js';
import type { RequestHandler } from './$types';

// The game page and the library menu call these; they forward to the Rust API with the session
// token. Only the listed calls pass. Other sites can't call them: a JSON body or a PUT/PATCH/DELETE
// needs a CORS preflight this app never allows, and SvelteKit rejects cross-origin form posts.
const slug = '[a-z0-9]+(?:-[a-z0-9]+)*';
const uuid = '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}';
const allowed: Record<'POST' | 'PUT' | 'PATCH' | 'DELETE', RegExp> = {
	POST: /^me\/lists$/,
	PUT: new RegExp(`^(games/${slug}/(rating|review)|me/lists/${uuid}/games/${slug})$`),
	PATCH: new RegExp(`^me/lists/${uuid}$`),
	DELETE: new RegExp(`^(games/${slug}/(rating|review)|me/lists/${uuid}(/games/${slug})?)$`)
};

async function forward(event: RequestEvent, method: keyof typeof allowed) {
	const path = (event.params as { path: string }).path;
	if (!allowed[method].test(path)) return json({ error: 'not_found' }, { status: 404 });
	const token = event.locals.token;
	if (!token) return json({ error: 'unauthenticated' }, { status: 401 });

	const text = await event.request.text();
	if (text.length > 10_000) return json({ error: 'too_large' }, { status: 413 });
	let body: unknown;
	try {
		body = text ? JSON.parse(text) : undefined;
	} catch {
		return json({ error: 'invalid_body' }, { status: 400 });
	}

	const result = await api(event, `/v1/${path}`, { method, body, token });
	if (!result.ok) {
		if (result.status === 401 && result.error === 'unauthenticated') {
			clearSessionCookie(event.cookies);
		}
		return json({ error: result.error }, { status: result.status });
	}
	return result.data === null
		? new Response(null, { status: 204 })
		: json(result.data, { status: result.status });
}

export const POST: RequestHandler = (event) => forward(event, 'POST');
export const PUT: RequestHandler = (event) => forward(event, 'PUT');
export const PATCH: RequestHandler = (event) => forward(event, 'PATCH');
export const DELETE: RequestHandler = (event) => forward(event, 'DELETE');
