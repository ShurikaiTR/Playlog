import { error } from '@sveltejs/kit';

export type Provider = 'google' | 'discord';

export function providerParam(value: string): Provider {
	if (value === 'google' || value === 'discord') return value;
	error(404, 'Sayfa bulunamadı');
}

// The only places the start route may send a visitor.
export const PROVIDER_ORIGINS = ['https://accounts.google.com', 'https://discord.com'];
