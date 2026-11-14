import { adminApi, requireStaff, type AdminStats } from '#lib/server/admin.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const { user } = requireStaff(event.locals);
	const stats = await adminApi<AdminStats>(event, '/stats');
	return { stats: stats.ok ? stats.data : null, isAdmin: user.role === 'admin' };
};
