<script lang="ts">
	import type { Component } from 'svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';
	import AuthTerms from './auth-terms.svelte';

	// The sign-in options: last-used hint, provider buttons, "veya", then the email field.
	// Page-agnostic on purpose, so a sign-in modal can reuse it later.
	type Provider = { id: string; label: string; icon: Component };
	type Props = {
		providers: Provider[];
		lastProvider: string | null;
		email: string;
		onProvider: (id: string) => void;
		onEmail: () => void;
		signUpHref: string;
		onSwitch?: (event: MouseEvent) => void;
	};

	let {
		providers,
		lastProvider,
		email = $bindable(),
		onProvider,
		onEmail,
		signUpHref,
		onSwitch
	}: Props = $props();

	const lastLabel = $derived(
		lastProvider === 'email' ? 'Email' : providers.find((p) => p.id === lastProvider)?.label
	);
	const ringStyle = 'mask-image: radial-gradient(circle, transparent 90%, black 100%);';
</script>

<div class="flex flex-col gap-3">
	<h1 class="text-center text-title-2 font-semibold tracking-tight text-balance text-primary">
		Log in to your account
	</h1>

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
			<Button
				variant="outline"
				class="h-10 w-full text-sm font-medium [&_svg:not([class*='size-'])]:size-4"
				onclick={() => onProvider(provider.id)}
				><provider.icon />Continue with {provider.label}</Button
			>
		</div>
	{/each}

	<div class="flex items-center gap-3 py-1" aria-hidden="true">
		<Separator class="flex-1" />
		<span class="text-xs text-muted-foreground">or</span>
		<Separator class="flex-1" />
	</div>

	<form
		class="flex flex-col gap-3"
		onsubmit={(event) => {
			event.preventDefault();
			onEmail();
		}}
	>
		<div class="relative flex flex-col gap-2">
			<Label for="login-email" class="sr-only">Email</Label>
			{#if lastProvider === 'email'}
				<div
					class="pointer-events-none absolute -inset-1.5 rounded-2xl border border-orange-500"
					style={ringStyle}
					aria-hidden="true"
				></div>
			{/if}
			<Input
				id="login-email"
				name="email"
				type="email"
				placeholder="Email"
				bind:value={email}
				autocomplete="email"
				required
				class="h-9 rounded-xl"
			/>
		</div>

		<div class="pt-5">
			<Button type="submit" class="h-10 w-full text-sm font-medium">Next</Button>
		</div>
	</form>

	<p class="text-center text-footnote text-muted-foreground">
		Don't have an account?
		<a href={signUpHref} onclick={onSwitch} class="font-medium text-foreground hover:underline"
			>Kaydol</a
		>
	</p>

	<AuthTerms />
</div>
