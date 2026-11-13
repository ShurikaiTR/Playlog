import { fail, redirect } from '@sveltejs/kit';
import { authErrorMessage } from '#lib/features/auth/auth-errors.js';
import { api, type ApiUser } from '#lib/server/api.js';
import { enabledProviders } from '#lib/server/oauth.js';
import { safeReturnPath, setSessionCookie } from '#lib/server/session.js';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const returnTo = safeReturnPath(event.url.searchParams.get('return_to'));
	if (event.locals.user) redirect(303, returnTo);
	const notice = event.url.searchParams.get('error');
	return {
		returnTo,
		notice: notice ? authErrorMessage(notice) : null,
		providers: await enabledProviders(event)
	};
};

const text = (data: FormData, key: string) => String(data.get(key) ?? '').trim();
// The Turnstile widget's response, posted by the form as `cf-turnstile-response`.
const captcha = (data: FormData) => text(data, 'cf-turnstile-response');

const codeError = (result: { status: number; error: string }) =>
	result.status === 404
		? 'Email login codes are not available right now. Log in with your password instead.'
		: authErrorMessage(result.error);

export const actions: Actions = {
	login: async (event) => {
		const data = await event.request.formData();
		const identifier = text(data, 'email');
		const password = String(data.get('password') ?? '');
		if (!identifier || !password) return fail(400, { error: 'Enter your email and password.' });

		const result = await api<{ token: string; user: ApiUser }>(event, '/v1/auth/login', {
			body: { identifier, password, turnstile_token: captcha(data) }
		});
		if (!result.ok) return fail(result.status, { error: authErrorMessage(result.error) });

		setSessionCookie(event.cookies, result.data.token);
		redirect(303, safeReturnPath(text(data, 'return_to')));
	},

	// Mails a reset code. The API answers the same whether or not the address has an account.
	resetStart: async (event) => {
		const data = await event.request.formData();
		const email = text(data, 'email');
		if (!email) return fail(400, { error: 'Enter your email address first.' });

		const result = await api(event, '/v1/auth/password-reset/start', {
			body: { email, turnstile_token: captcha(data) }
		});
		if (!result.ok) return fail(result.status, { error: authErrorMessage(result.error) });
		return { resetSent: true };
	},

	resetConfirm: async (event) => {
		const data = await event.request.formData();
		const email = text(data, 'email');
		const code = text(data, 'code').replace(/\D/g, '');
		const newPassword = String(data.get('password') ?? '');
		if (code.length !== 6) return fail(400, { error: 'Enter the 6-digit code from your email.' });

		const result = await api(event, '/v1/auth/password-reset/confirm', {
			body: { email, code, new_password: newPassword }
		});
		if (!result.ok) {
			return fail(result.status, { error: authErrorMessage(result.error), code: result.error });
		}
		return { resetDone: true };
	},

	// Emails a one-time sign-in code (the API answers the same whether or not the address has an account).
	codeStart: async (event) => {
		const data = await event.request.formData();
		const email = text(data, 'email');
		if (!email) return fail(400, { error: 'Enter your email address first.' });

		const result = await api(event, '/v1/auth/login-code/start', {
			body: { email, turnstile_token: captcha(data) }
		});
		if (!result.ok) return fail(result.status, { error: codeError(result) });
		return { codeSent: true };
	},

	codeVerify: async (event) => {
		const data = await event.request.formData();
		const email = text(data, 'email');
		const code = text(data, 'code').replace(/\D/g, '');
		if (code.length !== 6) return fail(400, { error: 'Enter the 6-digit code from your email.' });

		const result = await api<{ token: string; user: ApiUser }>(
			event,
			'/v1/auth/login-code/verify',
			{
				body: { email, code }
			}
		);
		if (!result.ok) return fail(result.status, { error: codeError(result), code: result.error });

		setSessionCookie(event.cookies, result.data.token);
		redirect(303, safeReturnPath(text(data, 'return_to')));
	}
};
