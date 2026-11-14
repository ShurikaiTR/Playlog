import { getMyLists } from '#lib/server/social.js';
import type { LayoutServerLoad } from './$types';

// The signed-in member's lists, for the library menu, the list pages and the profile. Visitors
// have none. Mutations update the client copy as they go, so this loads once per visit.
export const load: LayoutServerLoad = async (event) => ({
	lists: event.locals.token ? await getMyLists(event, event.locals.token) : null
});
