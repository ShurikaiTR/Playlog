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
	}
});
