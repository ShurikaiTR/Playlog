<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { page } from '$app/state';
	import XIcon from '@lucide/svelte/icons/x';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { authHref } from './return-to.js';
	import SignInFlow from './sign-in-flow.svelte';
	import SignUpFlow from './sign-up-flow.svelte';

	// Shown while the history entry carries `authModal` state (set by openAuthModal). A direct visit
	// or refresh on /sign-in or /sign-up has no such state and renders the full page instead.
	const modal = $derived(page.state.authModal);

	function swap(event: MouseEvent, kind: 'sign-in' | 'sign-up') {
		if (!modal) return;
		event.preventDefault();
		replaceState(authHref(kind, modal.returnTo), { authModal: { ...modal, kind, notice: null } });
	}

	// Closing goes back one entry, so the URL returns to the page underneath.
	function onOpenChange(open: boolean) {
		if (!open && page.state.authModal) history.back();
	}
</script>

<Dialog.Root open={!!modal} {onOpenChange}>
	<Dialog.Content
		class="max-w-[calc(100vw-2rem)] gap-0 overflow-hidden p-0 sm:max-w-100"
		showCloseButton={false}
		aria-describedby={undefined}
	>
		<Dialog.Title class="sr-only">{modal?.kind === 'sign-up' ? 'Sign up' : 'Log in'}</Dialog.Title>
		<Dialog.Close>
			{#snippet child({ props })}
				<!-- The class goes after the spread: the snippet's props carry their own class, which would
				replace this one and drop rounded-xl. -->
				<Button
					variant="ghost"
					size="icon-sm"
					aria-label="Close"
					{...props}
					class="absolute end-4 top-4 z-10 rounded-xl text-muted-foreground hover:text-foreground"
					><XIcon /></Button
				>
			{/snippet}
		</Dialog.Close>
		<div class="max-h-[calc(100dvh-2rem)] overflow-y-auto overscroll-contain">
			{#if modal?.kind === 'sign-up'}
				<SignUpFlow
					enabled={modal.enabled}
					returnTo={modal.returnTo}
					onSwitch={(event) => swap(event, 'sign-in')}
				/>
			{:else if modal}
				<SignInFlow
					enabled={modal.enabled}
					returnTo={modal.returnTo}
					notice={modal.notice}
					onSwitch={(event) => swap(event, 'sign-up')}
				/>
			{/if}
		</div>
	</Dialog.Content>
</Dialog.Root>
