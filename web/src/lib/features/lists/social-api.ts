import { goto } from '$app/navigation';
import { toast } from 'svelte-sonner';
import { currentReturnPath } from '#lib/features/auth/open-auth-modal.js';
import { authHref } from '#lib/features/auth/return-to.js';

type Result<T> = { ok: true; data: T } | { ok: false; error: string };

/** Calls the app's own `/api/social/...` proxy, which adds the session and forwards to the API. */
export async function socialRequest<T = null>(
	method: 'POST' | 'PUT' | 'PATCH' | 'DELETE',
	path: string,
	body?: unknown
): Promise<Result<T>> {
	try {
		const response = await fetch(`/api/social/${path}`, {
			method,
			headers: body === undefined ? undefined : { 'content-type': 'application/json' },
			body: body === undefined ? undefined : JSON.stringify(body)
		});
		if (response.ok) {
			return { ok: true, data: (response.status === 204 ? null : await response.json()) as T };
		}
		const data = (await response.json().catch(() => null)) as { error?: string } | null;
		return { ok: false, error: data?.error ?? 'unknown' };
	} catch {
		return { ok: false, error: 'unavailable' };
	}
}

const messages: Record<string, string> = {
	too_many_lists: 'You have reached the limit of 100 lists.',
	list_full: 'This list is full.',
	rate_limited: 'Slow down a little and try again.',
	invalid_review: 'A review needs 10 to 5,000 characters.',
	unavailable: 'We can’t reach Playlog right now.'
};

export function failToast(error: string) {
	toast.error(messages[error] ?? 'Something went wrong. Try again.');
}

/** Saving, rating and reviewing need an account; offer sign-in instead of failing silently. */
export function promptSignIn(action: string) {
	toast(`Sign in to ${action}`, {
		action: {
			label: 'Sign in',
			onClick: () => goto(authHref('sign-in', currentReturnPath()))
		}
	});
}
