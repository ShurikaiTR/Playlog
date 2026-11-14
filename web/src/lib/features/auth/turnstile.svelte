<script lang="ts" module>
	declare global {
		interface Window {
			turnstile?: {
				render: (el: HTMLElement, options: Record<string, unknown>) => string;
				reset: (widgetId?: string) => void;
				remove: (widgetId?: string) => void;
			};
		}
	}

	// Cloudflare's script is loaded once, in explicit-render mode (official docs).
	let scriptPromise: Promise<void> | undefined;
	function loadScript() {
		scriptPromise ??= new Promise<void>((resolve, reject) => {
			const script = document.createElement('script');
			script.src = 'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit';
			script.async = true;
			script.onload = () => resolve();
			script.onerror = () => {
				scriptPromise = undefined;
				reject(new Error('Turnstile could not be loaded'));
			};
			document.head.append(script);
		});
		return scriptPromise;
	}
</script>

<script lang="ts">
	import { TURNSTILE_SITE_KEY } from '$app/env/public';
	import { mode } from 'mode-watcher';

	// The Managed widget. `token` is the response to post as `cf-turnstile-response`; tokens work
	// once, so call `reset()` after every submit to get a fresh one.
	// The rule doesn't know `token` is a bindable prop that the parent reads.
	// eslint-disable-next-line no-useless-assignment
	let { token = $bindable('') }: { token?: string } = $props();

	let container = $state<HTMLElement>();
	let widgetId: string | undefined;

	export function reset() {
		token = '';
		if (widgetId) window.turnstile?.reset(widgetId);
	}

	$effect(() => {
		if (!container || !TURNSTILE_SITE_KEY) return;
		const element = container;
		let cancelled = false;
		loadScript()
			.then(() => {
				if (cancelled || !window.turnstile) return;
				widgetId = window.turnstile.render(element, {
					sitekey: TURNSTILE_SITE_KEY,
					theme: mode.current === 'light' ? 'light' : 'dark',
					language: 'en',
					callback: (value: string) => (token = value),
					'expired-callback': () => (token = ''),
					'timeout-callback': () => (token = ''),
					'error-callback': () => (token = '')
				});
			})
			.catch(() => {
				token = '';
			});
		return () => {
			cancelled = true;
			if (widgetId) window.turnstile?.remove(widgetId);
			widgetId = undefined;
			token = '';
		};
	});
</script>

{#if TURNSTILE_SITE_KEY}
	<div bind:this={container} class="flex min-h-16.25 justify-center" data-testid="turnstile"></div>
{/if}
