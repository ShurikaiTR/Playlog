import { error, redirect } from '@sveltejs/kit';
import { hasRole, type Role } from '#lib/roles.js';
import type { ApiUser } from './api.js';

/**
 * For `+page.server.ts` / `+layout.server.ts` loads of staff-only pages: sends visitors without a
 * session to sign in and answers 403 to signed-in users whose role is too low. The API checks the
 * role again on every call, so this only keeps the page itself from rendering.
 */
export function requireRole(locals: App.Locals, min: Exclude<Role, 'user'>): ApiUser {
	if (!locals.user) redirect(303, '/giris');
	if (!hasRole(locals.user.role, min)) error(403, 'Bu sayfaya erişim yetkin yok');
	return locals.user;
}
