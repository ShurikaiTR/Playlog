import { defineEnvVars } from '@sveltejs/kit/env';
import { building } from '$app/env';

export const variables = defineEnvVars({
	// The Rust API. Only the SvelteKit server talks to it; the browser never does.
	API_URL: {
		description: 'Rust API base URL, e.g. http://api:8080',
		schema: (input) => {
			if (!input && !building) throw new Error('API_URL is not set');
			return (input ?? '').replace(/\/+$/, '');
		}
	},
	// Cloudflare Turnstile site key. Public by design; the secret lives only on the Rust API.
	// Unset means no widget is shown (local dev); the API then skips the check as well.
	TURNSTILE_SITE_KEY: {
		public: true,
		description: 'Cloudflare Turnstile site key (public); leave unset to turn the widget off',
		schema: (input) => input || undefined
	}
});
