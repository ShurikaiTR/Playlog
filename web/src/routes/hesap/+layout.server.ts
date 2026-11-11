import { error, redirect } from '@sveltejs/kit';
import type { Account } from '#lib/features/account/account.js';
import { api } from '#lib/server/api.js';
import type { LayoutServerLoad } from './$types';

type Providers = Account['available'];

export const load: LayoutServerLoad = async (event) => {
	const token = event.locals.token;
	if (!event.locals.user || !token) redirect(303, '/sign-in');

	const [account, providers] = await Promise.all([
		api<Omit<Account, 'available'>>(event, '/v1/account', { method: 'GET', token }),
		api<Providers>(event, '/v1/auth/oauth/providers', { method: 'GET' })
	]);
	if (!account.ok) {
		if (account.status === 401) redirect(303, '/sign-in');
		error(503, 'Hesap bilgilerin şu an yüklenemedi. Biraz sonra tekrar dene.');
	}
	return {
		account: {
			...account.data,
			// Providers without keys on the server are listed but can't be connected yet.
			available: providers.ok
				? providers.data
				: { google: false, discord: false, apple: false, steam: false }
		} satisfies Account
	};
};
