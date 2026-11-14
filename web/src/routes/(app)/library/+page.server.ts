import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

// Lists belong to a member; visitors go to sign in and come back here.
export const load: PageServerLoad = ({ locals }) => {
	if (!locals.user) redirect(303, '/sign-in?return_to=/library');
	return {};
};
