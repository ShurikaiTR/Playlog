import { error, redirect } from '@sveltejs/kit';
import type { Device } from '#lib/features/account/account.js';
import { api } from '#lib/server/api.js';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async (event) => {
	const token = event.locals.token;
	if (!token) redirect(303, '/sign-in');
	const sessions = await api<Device[]>(event, '/v1/account/sessions', { method: 'GET', token });
	if (!sessions.ok) error(503, 'We couldn’t load your sessions right now. Try again in a moment.');
	// This device first, then the most recently active.
	return { sessions: sessions.data };
};
