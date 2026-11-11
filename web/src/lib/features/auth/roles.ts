// Mirrors the API's `user_role` enum. Order is privilege: user < moderator < admin.
// Checks here only decide what the UI shows; the Rust API enforces every permission itself.
export const ROLES = ['user', 'moderator', 'admin'] as const;

export type Role = (typeof ROLES)[number];

/** Whether `role` is at least `min` (an admin also counts as a moderator). */
export function hasRole(role: Role, min: Role): boolean {
	return ROLES.indexOf(role) >= ROLES.indexOf(min);
}
