import { error, redirect } from '@sveltejs/kit';
import { getGame } from '#lib/server/catalog.js';
import { getGameSocial } from '#lib/server/social.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const result = await getGame(event, event.params.slug);
	if (result.status !== 'found') {
		error(
			result.status === 'missing' ? 404 : 503,
			result.status === 'missing' ? 'Game not found' : 'We can’t reach the catalog right now'
		);
	}
	// An old address still finds the game; send the visitor to the current one.
	if (result.game.slug !== event.params.slug) redirect(308, `/games/${result.game.slug}`);
	// Re-run after the visitor rates, reviews or lists the game (`invalidate('app:social')`).
	event.depends('app:social');
	return {
		game: result.game,
		social: await getGameSocial(event, result.game.slug, event.locals.token)
	};
};
