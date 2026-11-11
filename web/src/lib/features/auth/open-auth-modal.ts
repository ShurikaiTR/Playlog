import { goto, preloadData, pushState } from '$app/navigation';
import { page } from '$app/state';
import type { EnabledProviders } from './providers.js';
import { authHref, safeReturnPath, type AuthKind } from './return-to.js';

export type { AuthKind };

/** The page the visitor is on, to come back to after logging in or signing up. */
export const currentReturnPath = () => safeReturnPath(page.url.pathname + page.url.search);

/**
 * Click handler for the "Log in" and "Sign up" links. A plain click opens the form in a modal
 * over the current page (shallow routing, so the URL still becomes /sign-in or /sign-up); modified
 * clicks and failed loads fall back to the normal page. `return_to` brings the visitor back here.
 */
export async function openAuthModal(event: MouseEvent, kind: AuthKind) {
	if (event.shiftKey || event.metaKey || event.ctrlKey || event.altKey || event.button !== 0)
		return;
	event.preventDefault();
	const href = authHref(kind, currentReturnPath());
	const result = await preloadData(href);
	if (result.type === 'loaded' && result.status === 200) {
		const data = result.data as {
			providers: EnabledProviders;
			notice?: string | null;
			returnTo: string;
		};
		pushState(href, {
			authModal: {
				kind,
				enabled: data.providers,
				notice: data.notice ?? null,
				returnTo: data.returnTo
			}
		});
	} else {
		await goto(href);
	}
}
