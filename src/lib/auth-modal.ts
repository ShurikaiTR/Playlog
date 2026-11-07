import { goto, preloadData, pushState } from '$app/navigation';

export type AuthKind = 'sign-in' | 'sign-up';

/**
 * Click handler for the "Giriş yap" and "Kayıt ol" links. A plain click opens the form in a modal
 * over the current page (shallow routing, so the URL still becomes /sign-in or /sign-up); modified
 * clicks and failed loads fall back to the normal page.
 */
export async function openAuthModal(event: MouseEvent, kind: AuthKind) {
	if (event.shiftKey || event.metaKey || event.ctrlKey || event.altKey || event.button !== 0)
		return;
	event.preventDefault();
	const href = `/${kind}`;
	const result = await preloadData(href);
	if (result.type === 'loaded' && result.status === 200) {
		const data = result.data as {
			providers: Record<'google' | 'apple' | 'discord', boolean>;
			notice?: string | null;
		};
		pushState(href, { authModal: { kind, enabled: data.providers, notice: data.notice ?? null } });
	} else {
		await goto(href);
	}
}
