import { error } from '@sveltejs/kit';

export type Provider = 'google' | 'discord' | 'apple';

export function providerParam(value: string): Provider {
	if (value === 'google' || value === 'discord' || value === 'apple') return value;
	error(404, 'Sayfa bulunamadı');
}

// The only places the start route may send a visitor.
export const PROVIDER_ORIGINS = [
	'https://accounts.google.com',
	'https://discord.com',
	'https://appleid.apple.com'
];

/**
 * Apple sends the visitor's name only on the first sign-in, as a JSON `user` form field
 * (`{"name":{"firstName":"…","lastName":"…"}}`). It is unverified and used only as a display
 * name suggestion; anything malformed is ignored.
 */
export function appleDisplayName(raw: FormDataEntryValue | null): string | null {
	if (typeof raw !== 'string' || raw.length > 1000) return null;
	try {
		const name = (JSON.parse(raw) as { name?: { firstName?: unknown; lastName?: unknown } }).name;
		const parts = [name?.firstName, name?.lastName].filter(
			(part): part is string => typeof part === 'string'
		);
		return parts.join(' ').trim() || null;
	} catch {
		return null;
	}
}
