import { error, redirect } from '@sveltejs/kit';
import { api } from '#lib/server/api.js';
import type { RequestHandler } from './$types';

// "Verilerimi indir": the account's data from the API as a JSON file download.
export const GET: RequestHandler = async (event) => {
	const token = event.locals.token;
	if (!event.locals.user || !token) redirect(303, '/sign-in');
	const result = await api(event, '/v1/account/export', { method: 'GET', token });
	if (!result.ok) error(result.status === 429 ? 429 : 503, 'Verilerin şu an hazırlanamadı.');

	const day = new Date().toISOString().slice(0, 10);
	return new Response(JSON.stringify(result.data, null, 2), {
		headers: {
			'content-type': 'application/json; charset=utf-8',
			'content-disposition': `attachment; filename="playthru-${event.locals.user.username}-${day}.json"`,
			'cache-control': 'no-store'
		}
	});
};
