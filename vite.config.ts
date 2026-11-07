import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// Kendi sunucumuzda Node olarak çalışır: `node build`. Ayarlar ortam değişkenleriyle gelir.
			adapter: adapter(),

			// Apple returns from "Sign in with Apple" with a cross-site form POST to
			// /auth/apple/callback. Only that origin is trusted; the callback still checks the
			// signed state against the visitor's cookie.
			csrf: { trustedOrigins: ['https://appleid.apple.com'] }
		})
	]
});
