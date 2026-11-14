import { requireStaff } from '#lib/server/admin.js';
import type { LayoutServerLoad } from './$types';

// Moderators and admins only; everyone else gets a 404 (see requireStaff).
export const load: LayoutServerLoad = ({ locals }) => {
	const { user } = requireStaff(locals);
	return {
		staff: {
			username: user.username,
			displayName: user.display_name,
			role: user.role as 'moderator' | 'admin'
		}
	};
};
