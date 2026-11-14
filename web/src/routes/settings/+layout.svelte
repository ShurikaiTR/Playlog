<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import ArrowUpLeftIcon from '@lucide/svelte/icons/arrow-up-left';
	import DatabaseIcon from '@lucide/svelte/icons/database';
	import LockIcon from '@lucide/svelte/icons/lock';
	import LogOutIcon from '@lucide/svelte/icons/log-out';
	import MonitorSmartphoneIcon from '@lucide/svelte/icons/monitor-smartphone';
	import UserRoundIcon from '@lucide/svelte/icons/user-round';
	import { showAuthTransition } from '#lib/features/auth/auth-transition.svelte.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { ScrollArea } from '#lib/components/ui/scroll-area/index.js';
	import { cn } from '#lib/utils.js';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	// The account page has its own window, like the settings app: a left menu of sections and
	// the open section beside it. On phones the menu scrolls sideways above the section.
	const navItems = [
		{ href: '/settings', label: 'Account', icon: UserRoundIcon },
		{ href: '/settings/security', label: 'Security', icon: LockIcon },
		{ href: '/settings/sessions', label: 'Sessions', icon: MonitorSmartphoneIcon },
		{ href: '/settings/data', label: 'Data', icon: DatabaseIcon }
	] as const;

	const isActive = (href: string) =>
		href === '/settings' ? page.url.pathname === href : page.url.pathname.startsWith(href);

	let logoutForm = $state<HTMLFormElement | null>(null);

	function signOut() {
		showAuthTransition({ title: 'Signing out', description: 'See you around…' });
		logoutForm?.requestSubmit();
	}
</script>

<form bind:this={logoutForm} method="POST" action="/sign-out" hidden></form>

<div class="min-h-dvh bg-background text-foreground md:flex">
	<aside
		class="flex shrink-0 flex-col gap-6 px-4 pt-5 md:sticky md:top-0 md:h-dvh md:w-64 md:gap-10 md:px-5 md:py-8"
	>
		<div class="flex items-center justify-between gap-4 md:flex-col md:items-start md:gap-6">
			<a
				href={resolve('/(app)')}
				class="inline-block rounded-full outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
				aria-label="Playthru home"
			>
				<img
					src="/logos/playthru.svg"
					alt="Playthru"
					width="282"
					height="56"
					class="h-8 w-auto dark:invert"
				/>
			</a>
			<Button
				href="/"
				variant="ghost"
				class="-mx-3 h-11 gap-2 rounded-xl px-3 text-callout font-normal text-muted-foreground"
			>
				<ArrowUpLeftIcon class="size-4" />Back to the site
			</Button>
		</div>

		<ScrollArea orientation="horizontal" class="-mx-1 md:mx-0">
			<nav class="flex gap-1 px-1 pb-1 md:flex-col md:px-0" aria-label="Account sections">
				{#each navItems as item (item.href)}
					{@const active = isActive(item.href)}
					<Button
						href={item.href}
						variant="ghost"
						class={cn(
							'h-11 justify-start gap-2.5 rounded-xl px-3 text-callout font-normal',
							active ? 'bg-muted text-foreground' : 'text-muted-foreground'
						)}
						aria-current={active ? 'page' : undefined}
					>
						<item.icon class="size-4" />{item.label}
					</Button>
				{/each}
			</nav>
		</ScrollArea>

		<Button
			variant="ghost"
			class="hidden h-11 cursor-pointer justify-start gap-2.5 rounded-xl px-3 text-callout font-normal text-muted-foreground md:mt-auto md:inline-flex"
			onclick={signOut}
		>
			<LogOutIcon class="size-4" />Sign out
		</Button>
	</aside>

	<main class="min-w-0 flex-1 px-4 pt-8 pb-20 md:px-16 md:pt-20">
		<div class="mx-auto max-w-3xl text-callout">
			{@render children()}
		</div>
	</main>
</div>
