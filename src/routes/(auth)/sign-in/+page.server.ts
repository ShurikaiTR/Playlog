import { fail, redirect } from '@sveltejs/kit';
import { authErrorMessage } from '#lib/auth-errors.js';
import { api, type ApiUser } from '#lib/server/api.js';
import { enabledProviders } from '#lib/server/oauth.js';
import { setSessionCookie } from '#lib/server/session.js';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	if (event.locals.user) redirect(303, '/');
	const notice = event.url.searchParams.get('hata');
	return {
		notice: notice ? authErrorMessage(notice) : null,
		providers: await enabledProviders(event)
	};
};

const text = (data: FormData, key: string) => String(data.get(key) ?? '').trim();

export const actions: Actions = {
	login: async (event) => {
		const data = await event.request.formData();
		const identifier = text(data, 'email');
		const password = String(data.get('password') ?? '');
		if (!identifier || !password) return fail(400, { error: 'E-postanı ve şifreni gir.' });

		const result = await api<{ token: string; user: ApiUser }>(event, '/v1/auth/login', {
			body: { identifier, password }
		});
		if (!result.ok) return fail(result.status, { error: authErrorMessage(result.error) });

		setSessionCookie(event.cookies, result.data.token);
		redirect(303, '/');
	},

	// Mails a reset code. The API answers the same whether or not the address has an account.
	resetStart: async (event) => {
		const data = await event.request.formData();
		const email = text(data, 'email');
		if (!email) return fail(400, { error: 'Önce e-posta adresini gir.' });

		const result = await api(event, '/v1/auth/password-reset/start', { body: { email } });
		if (!result.ok) return fail(result.status, { error: authErrorMessage(result.error) });
		return { resetSent: true };
	},

	resetConfirm: async (event) => {
		const data = await event.request.formData();
		const email = text(data, 'email');
		const code = text(data, 'code').replace(/\D/g, '');
		const newPassword = String(data.get('password') ?? '');
		if (code.length !== 6) return fail(400, { error: 'E-postana gelen 6 haneli kodu gir.' });

		const result = await api(event, '/v1/auth/password-reset/confirm', {
			body: { email, code, new_password: newPassword }
		});
		if (!result.ok) {
			return fail(result.status, { error: authErrorMessage(result.error), code: result.error });
		}
		return { resetDone: true };
	}
};
