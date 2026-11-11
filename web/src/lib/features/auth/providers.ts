// The sign-in providers the Rust API knows. Which ones are switched on depends on the keys
// configured there; pages get that as an `EnabledProviders` record.
export const PROVIDERS = ['google', 'apple', 'discord', 'steam'] as const;

export type Provider = (typeof PROVIDERS)[number];

export type EnabledProviders = Record<Provider, boolean>;

export const providerLabels: Record<Provider, string> = {
	google: 'Google',
	apple: 'Apple',
	discord: 'Discord',
	steam: 'Steam'
};

export const isProvider = (value: string): value is Provider =>
	(PROVIDERS as readonly string[]).includes(value);
