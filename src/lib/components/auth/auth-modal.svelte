<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { page } from '$app/state';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import SignInFlow from './sign-in-flow.svelte';
	import SignUpFlow from './sign-up-flow.svelte';

	// Shown while the history entry carries `authModal` state (set by openAuthModal). A direct visit
	// or refresh on /sign-in or /sign-up has no such state and renders the full page instead.
	const modal = $derived(page.state.authModal);

	function swap(event: MouseEvent, kind: 'sign-in' | 'sign-up') {
		if (!modal) return;
		event.preventDefault();
		replaceState(`/${kind}`, { authModal: { ...modal, kind, notice: null } });
	}

	// Closing goes back one entry, so the URL returns to the page underneath.
	function onOpenChange(open: boolean) {
		if (!open && page.state.authModal) history.back();
	}
</script>

<Dialog.Root open={!!modal} {onOpenChange}>
	<Dialog.Content class="gap-0 p-0 sm:max-w-100" aria-describedby={undefined}>
		<Dialog.Title class="sr-only">{modal?.kind === 'sign-up' ? 'Sign up' : 'Log in'}</Dialog.Title>
		<div class="max-h-[calc(100dvh-2rem)] overflow-y-auto px-5 pt-14 pb-6 sm:px-6">
			{#if modal?.kind === 'sign-up'}
				<SignUpFlow enabled={modal.enabled} onSwitch={(event) => swap(event, 'sign-in')} />
			{:else if modal}
				<SignInFlow
					enabled={modal.enabled}
					notice={modal.notice}
					onSwitch={(event) => swap(event, 'sign-up')}
				/>
			{/if}
		</div>
	</Dialog.Content>
</Dialog.Root>
