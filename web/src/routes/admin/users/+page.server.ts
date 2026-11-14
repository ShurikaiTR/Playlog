import { fail } from '@sveltejs/kit';
import { ROLES } from '#lib/features/auth/roles.js';
import { adminApi, requireStaff, type AdminUserRow } from '#lib/server/admin.js';
import type { Actions, PageServerLoad } from './$types';

const PER_PAGE = 25;

const errors: Record<string, string> = {
	last_admin: 'There must always be at least one admin.',
	own_role: 'You can’t change your own role.',
	user_not_found: 'That user no longer exists.',
	invalid_role: 'Pick a valid role.',
	rate_limited: 'Too many changes in a short time. Try again in a few minutes.',
	forbidden: 'Only admins can change roles.'
};

// Only admins see this list: it shows email addresses.
export const load: PageServerLoad = async (event) => {
	const { user } = requireStaff(event.locals, 'admin');
	const params = event.url.searchParams;
	const q = params.get('q')?.trim().slice(0, 100) ?? '';
	const role = ROLES.includes((params.get('role') ?? '') as never) ? params.get('role')! : 'all';
	const page = Math.max(1, Math.floor(Number(params.get('page'))) || 1);

	const query = new URLSearchParams({
		limit: String(PER_PAGE),
		offset: String((page - 1) * PER_PAGE)
	});
	if (q) query.set('q', q);
	if (role !== 'all') query.set('role', role);
	const result = await adminApi<{ total: number; items: AdminUserRow[] }>(event, `/users?${query}`);

	return {
		users: result.ok ? result.data.items : [],
		total: result.ok ? result.data.total : 0,
		failed: !result.ok,
		q,
		role,
		page,
		perPage: PER_PAGE,
		meId: user.id
	};
};

export const actions: Actions = {
	role: async (event) => {
		requireStaff(event.locals, 'admin');
		const form = await event.request.formData();
		const id = form.get('id');
		const role = form.get('role');
		if (typeof id !== 'string' || typeof role !== 'string' || !id) {
			return fail(400, { error: errors.invalid_role });
		}
		const result = await adminApi(event, `/users/${encodeURIComponent(id)}/role`, {
			method: 'PATCH',
			body: { role }
		});
		if (!result.ok) {
			return fail(result.status, { error: errors[result.error] ?? 'Could not change the role.' });
		}
		return { changed: true };
	}
};
