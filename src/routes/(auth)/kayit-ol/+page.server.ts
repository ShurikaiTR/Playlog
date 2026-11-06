import { fail, redirect } from '@sveltejs/kit';
import { authErrorMessage } from '#lib/auth-errors.js';
import { api, type ApiUser } from '#lib/server/api.js';
import { setSessionCookie } from '#lib/server/session.js';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = ({ locals }) => {
	if (locals.user) redirect(303, '/');
};

const text = (data: FormData, key: string) => String(data.get(key) ?? '').trim();

export const actions: Actions = {
	// Checks the details and mails the 6-digit code. Also used by "Tekrar gönder".
	start: async (event) => {
		const data = await event.request.formData();
		const result = await api(event, '/v1/auth/signup/start', {
			body: {
				email: text(data, 'email'),
				username: text(data, 'username'),
				password: String(data.get('password') ?? ''),
				display_name: text(data, 'display_name') || undefined,
				marketing_opt_in: data.get('marketing_opt_in') === 'true'
			}
		});
		if (!result.ok) {
			return fail(result.status, { error: authErrorMessage(result.error), code: result.error });
		}
		return { codeSent: true };
	},

	// The right code creates the account and signs the visitor in.
	verify: async (event) => {
		const data = await event.request.formData();
		const code = text(data, 'code').replace(/\D/g, '');
		if (code.length !== 6) return fail(400, { error: 'E-postana gelen 6 haneli kodu gir.' });

		const result = await api<{ token: string; user: ApiUser }>(event, '/v1/auth/signup/verify', {
			body: { email: text(data, 'email'), code }
		});
		if (!result.ok) {
			return fail(result.status, { error: authErrorMessage(result.error), code: result.error });
		}

		setSessionCookie(event.cookies, result.data.token);
		redirect(303, '/');
	}
};
