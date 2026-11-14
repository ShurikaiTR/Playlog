import { adminApi, requireStaff, type AdminStats } from '#lib/server/admin.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	requireStaff(event.locals);
	const stats = await adminApi<AdminStats>(event, '/stats');
	return { stats: stats.ok ? stats.data : null };
};
