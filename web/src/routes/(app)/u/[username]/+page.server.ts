import { error } from '@sveltejs/kit';
import type { Profile } from '#lib/features/profiles/profiles.js';
import type { PageServerLoad } from './$types';

// Profiles are not public yet: the API has no way to look up another member, so the only
// profile that exists is the signed-in visitor's own. Following and public lists come with it.
export const load: PageServerLoad = ({ params, locals }) => {
	const user = locals.user;
	if (!user || user.username !== params.username.toLowerCase()) error(404, 'Profile not found');
	const profile: Profile = {
		name: user.display_name || user.username,
		username: user.username,
		cover: '',
		following: [],
		followers: [],
		lists: []
	};
	return { profile, isOwner: true, linkable: [] as string[] };
};
