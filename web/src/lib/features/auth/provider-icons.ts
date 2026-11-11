import type { Component } from 'svelte';
import AppleIcon from '#lib/components/icons/apple.svelte';
import DiscordIcon from '#lib/components/icons/discord.svelte';
import GoogleIcon from '#lib/components/icons/google.svelte';
import SteamIcon from '#lib/components/icons/steam.svelte';
import type { Provider } from './providers.js';

export const providerIcons: Record<Provider, Component<{ class?: string }, object, ''>> = {
	google: GoogleIcon,
	apple: AppleIcon,
	discord: DiscordIcon,
	steam: SteamIcon
};
