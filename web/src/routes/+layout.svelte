<script lang="ts">
	import './layout.css';
	import favicon from '#lib/assets/favicon.svg';
	import { Toaster } from '#lib/components/ui/sonner/index.js';
	import { ModeWatcher } from 'mode-watcher';
	import AuthTransitionScreen from '#lib/features/auth/auth-transition-screen.svelte';
	import AuthModal from '#lib/features/auth/auth-modal.svelte';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<link rel="apple-touch-icon" href="/apple-touch-icon.png" />
</svelte:head>

<ModeWatcher defaultMode="dark" />
<AuthTransitionScreen />
{@render children()}
<AuthModal />
<Toaster />
