// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
import type { ApiUser } from '#lib/server/api.js';

declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			/** The signed-in user, filled from the session cookie in hooks.server.ts. */
			user: ApiUser | null;
			/** The session token, for API calls made on the user's behalf. */
			token: string | null;
		}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
