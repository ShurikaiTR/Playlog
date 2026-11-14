import type { RequestEvent } from '@sveltejs/kit';
import { API_URL } from '$app/env/private';
import type { Role } from '#lib/features/auth/roles.js';

export type ApiUser = {
	id: string;
	username: string;
	email: string | null;
	display_name: string | null;
	role: Role;
};

export type ApiResult<T> =
	{ ok: true; status: number; data: T } | { ok: false; status: number; error: string };

type Options = {
	method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
	body?: unknown;
	token?: string | null;
};

/**
 * Calls the Rust API from the server. The visitor's address and browser are passed on so
 * sessions record the real client, not this server. A network failure comes back as
 * `{ ok: false, status: 503, error: 'unavailable' }` so callers handle one shape.
 */
export async function api<T = unknown>(
	event: Pick<RequestEvent, 'request' | 'getClientAddress'>,
	path: string,
	{ method = 'POST', body, token }: Options = {}
): Promise<ApiResult<T>> {
	const headers = new Headers({ accept: 'application/json' });
	if (body !== undefined) headers.set('content-type', 'application/json');
	if (token) headers.set('authorization', `Bearer ${token}`);
	try {
		headers.set('x-client-ip', event.getClientAddress());
	} catch {
		// No address available (e.g. during prerender); the API then stores none.
	}
	const userAgent = event.request.headers.get('user-agent');
	if (userAgent) headers.set('x-client-user-agent', userAgent.slice(0, 300));

	let response: Response;
	try {
		response = await fetch(`${API_URL}${path}`, {
			method,
			headers,
			body: body === undefined ? undefined : JSON.stringify(body),
			signal: AbortSignal.timeout(10_000)
		});
	} catch (cause) {
		console.error(`API isteği başarısız (${path}):`, cause);
		return { ok: false, status: 503, error: 'unavailable' };
	}

	const text = await response.text();
	let data: unknown = null;
	try {
		data = text ? JSON.parse(text) : null;
	} catch {
		// Not JSON (e.g. a proxy error page); treated as an unknown error below.
	}
	if (response.ok) return { ok: true, status: response.status, data: data as T };
	const error =
		data && typeof data === 'object' && 'error' in data && typeof data.error === 'string'
			? data.error
			: 'unknown';
	return { ok: false, status: response.status, error };
}
