<script lang="ts">
	import type { Component } from 'svelte';
	import { looksLikeEmail } from './validation.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';

	// The sign-in options: last-used hint, provider buttons, "or", then the email field. It is the
	// body of the first step, so the page and the modal share it; the title and terms come from AuthShell.
	type Provider = { id: string; label: string; icon: Component };
	type Props = {
		providers: Provider[];
		lastProvider: string | null;
		email: string;
		onProvider: (id: string) => void;
		onEmail: () => void;
		signUpHref: string;
		onSwitch?: (event: MouseEvent) => void;
		/** Status or error text, shown in a reserved slot under the email field. */
		message?: string;
	};

	let {
		providers,
		lastProvider,
		email = $bindable(),
		onProvider,
		onEmail,
		signUpHref,
		onSwitch,
		message = ''
	}: Props = $props();

	const lastLabel = $derived(
		lastProvider === 'email' ? 'Email' : providers.find((p) => p.id === lastProvider)?.label
	);
	const emailValid = $derived(looksLikeEmail(email));
	const ringStyle = 'mask-image: radial-gradient(circle, transparent 90%, black 100%);';
</script>

<form
	class="flex flex-1 flex-col"
	onsubmit={(event) => {
		event.preventDefault();
		onEmail();
	}}
>
	<div class="mt-auto flex flex-col gap-3">
		<div class="min-h-5">
			{#if lastLabel}
				<p
					class="flex items-center justify-center gap-2 text-center text-xs text-muted-foreground"
					aria-live="polite"
				>
					<span
						class="pointer-events-none size-3 rounded-full border border-orange-500"
						style="mask-image: linear-gradient(transparent 0%, black 50%, transparent 100%);"
						aria-hidden="true"
					></span>
					You last logged in with {lastLabel}
				</p>
			{/if}
		</div>

		{#each providers as provider (provider.id)}
			<div class="relative">
				{#if lastProvider === provider.id}
					<div
						class="pointer-events-none absolute -inset-1.5 rounded-full border border-orange-500"
						style={ringStyle}
						aria-hidden="true"
					></div>
				{/if}
				<Button variant="outline" size="lg" class="w-full" onclick={() => onProvider(provider.id)}
					><provider.icon />Continue with {provider.label}</Button
				>
			</div>
		{/each}

		<div class="flex items-center gap-3" aria-hidden="true">
			<Separator class="flex-1" />
			<span class="text-xs text-muted-foreground">or</span>
			<Separator class="flex-1" />
		</div>

		<div class="relative flex flex-col gap-1.5">
			{#if lastProvider === 'email'}
				<div
					class="pointer-events-none absolute -top-1.5 -right-1.5 -left-1.5 h-12 rounded-2xl border border-orange-500"
					style={ringStyle}
					aria-hidden="true"
				></div>
			{/if}
			<Input
				id="login-email"
				aria-label="Email"
				name="email"
				type="email"
				placeholder="Enter your email"
				bind:value={email}
				autocomplete="email"
				required
				class="h-9 rounded-xl"
			/>
			<div class="min-h-5 text-center text-xs text-muted-foreground" aria-live="polite">
				{message}
			</div>
		</div>
	</div>

	<div class="mt-auto flex flex-col gap-3 pt-5">
		<Button type="submit" size="lg" class="w-full" disabled={!emailValid}>Continue</Button>
		<div class="flex h-12 flex-col items-center justify-center">
			<p class="text-center text-footnote text-muted-foreground">
				Don't have an account?
				<a
					href={signUpHref}
					onclick={onSwitch}
					class="font-medium text-primary underline underline-offset-4">Sign up</a
				>
			</p>
		</div>
	</div>
</form>
