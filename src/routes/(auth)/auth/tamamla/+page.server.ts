import { fail, redirect } from '@sveltejs/kit';
import { authErrorMessage } from '#lib/auth-errors.js';
import { api, type ApiUser } from '#lib/server/api.js';
import {
	clearCookie,
	OAUTH_PENDING_COOKIE,
	readPendingSignup,
	setSessionCookie
} from '#lib/server/session.js';
import type { Actions, PageServerLoad } from './$types';

// After Google/Discord sign-up: the visitor picks a username, then the account is created.
export const load: PageServerLoad = ({ cookies, locals }) => {
	if (locals.user) redirect(303, '/');
	const pending = readPendingSignup(cookies);
	if (!pending) redirect(303, '/sign-in?hata=invalid_pending');
	return {
		email: pending.email,
		suggestedUsername: pending.suggested_username ?? '',
		displayName: pending.display_name
	};
};

export const actions: Actions = {
	default: async (event) => {
		const pending = readPendingSignup(event.cookies);
		if (!pending) redirect(303, '/sign-in?hata=invalid_pending');

		const data = await event.request.formData();
		const result = await api<{ token: string; user: ApiUser }>(event, '/v1/auth/oauth/complete', {
			body: {
				pending_token: pending.pending_token,
				username: String(data.get('username') ?? '').trim(),
				marketing_opt_in: data.get('marketing_opt_in') === 'true'
			}
		});
		if (!result.ok) {
			if (result.error === 'invalid_pending') {
				clearCookie(event.cookies, OAUTH_PENDING_COOKIE);
				redirect(303, '/sign-in?hata=invalid_pending');
			}
			return fail(result.status, { error: authErrorMessage(result.error) });
		}

		clearCookie(event.cookies, OAUTH_PENDING_COOKIE);
		setSessionCookie(event.cookies, result.data.token);
		redirect(303, '/');
	}
};
