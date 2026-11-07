<script lang="ts">
	import { hideAuthTransition, showAuthTransition } from '#lib/auth-transition.svelte.js';
	import { applyAction, enhance, type SubmitFunction } from '$app/forms';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Checkbox } from '#lib/components/ui/checkbox/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import MailIcon from '#lib/components/icons/mail.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Prefilled once from the provider's suggestion; the visitor can change it.
	// svelte-ignore state_referenced_locally
	let username = $state(data.suggestedUsername);
	let wantsUpdates = $state(false);
	let statusMessage = $state('');
	let pending = $state(false);

	const submit: SubmitFunction = () => {
		pending = true;
		statusMessage = '';
		showAuthTransition({
			title: 'Creating your account',
			description: 'Getting everything ready, please wait…'
		});
		return async ({ result }) => {
			pending = false;
			if (result.type === 'redirect') {
				await applyAction(result);
				hideAuthTransition();
				return;
			}
			hideAuthTransition();
			statusMessage =
				result.type === 'failure' && typeof result.data?.error === 'string'
					? result.data.error
					: 'Something went wrong. Please try again.';
		};
	};
</script>

<svelte:head>
	<title>Finish signing up | Playthru</title>
</svelte:head>

<div lang="en">
	<form method="POST" class="space-y-4" use:enhance={submit}>
		<div class="mb-8 text-center">
			<h1 class="text-title-1 text-balance text-primary">Finish signing up</h1>
			<span
				class="mt-4 inline-flex max-w-full items-center gap-2 rounded-full border px-4 py-1.5 text-footnote"
			>
				<MailIcon class="size-4 shrink-0" />
				<span class="truncate">{data.email}</span>
			</span>
		</div>
		<input type="hidden" name="marketing_opt_in" value={wantsUpdates ? 'true' : ''} />

		<div class="space-y-2">
			<Label for="complete-username">Username</Label>
			<Input
				id="complete-username"
				name="username"
				bind:value={username}
				minlength={3}
				maxlength={30}
				autocomplete="username"
				required
				autofocus
			/>
		</div>
		<div class="flex items-center gap-2.5 py-1">
			<Checkbox id="complete-updates" bind:checked={wantsUpdates} />
			<Label for="complete-updates" class="cursor-pointer text-callout font-normal"
				>Email me about new features and announcements</Label
			>
		</div>

		<Button type="submit" size="auth" class="w-full" disabled={pending}
			>{pending ? 'Finishing…' : 'Finish sign-up'}</Button
		>
	</form>

	{#if statusMessage}
		<p class="mt-5 text-center text-footnote text-muted-foreground" aria-live="polite">
			{statusMessage}
		</p>
	{/if}
</div>
