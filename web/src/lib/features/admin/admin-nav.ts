import type { Component } from 'svelte';
import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
import GamepadIcon from '@lucide/svelte/icons/gamepad-2';
import LibraryIcon from '@lucide/svelte/icons/library';
import UsersIcon from '@lucide/svelte/icons/users';

export type AdminNavItem = { title: string; url: string; icon: Component; adminOnly?: boolean };

// A Moderation entry (edit suggestions, queue, history) goes here when that work starts.
export const adminNav: AdminNavItem[] = [
	{ title: 'Dashboard', url: '/admin', icon: LayoutDashboardIcon },
	{ title: 'Games', url: '/admin/games', icon: GamepadIcon },
	{ title: 'Series', url: '/admin/series', icon: LibraryIcon },
	{ title: 'Users', url: '/admin/users', icon: UsersIcon, adminOnly: true }
];

/** The section name shown in the header for an /admin address. */
export function sectionTitle(pathname: string): string {
	const item = [...adminNav]
		.reverse()
		.find((item) =>
			item.url === '/admin' ? pathname === item.url : pathname.startsWith(item.url)
		);
	return item?.title ?? 'Admin';
}
