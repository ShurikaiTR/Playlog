<script lang="ts">
	import { hideAuthTransition, showAuthTransition } from '#lib/auth-transition.svelte.js';
	import { applyAction, enhance, type SubmitFunction } from '$app/forms';
	import { onDestroy } from 'svelte';
	import { fade } from 'svelte/transition';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import Turnstile from './turnstile.svelte';
	import { TURNSTILE_SITE_KEY } from '$app/env/public';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as InputGroup from '#lib/components/ui/input-group/index.js';
	import { Checkbox } from '#lib/components/ui/checkbox/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as InputOTP from '#lib/components/ui/input-otp/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';
	import AppleIcon from '#lib/components/icons/apple.svelte';
	import DiscordIcon from '#lib/components/icons/discord.svelte';
	import GoogleIcon from '#lib/components/icons/google.svelte';
	import AuthTerms from '#lib/components/auth/auth-terms.svelte';
	import MailIcon from '#lib/components/icons/mail.svelte';

	// Sign-up options, then email, then the account details (which mails the code), then the code.
	// The right code creates the account and signs in.
	type Props = {
		enabled: Record<'google' | 'apple' | 'discord', boolean>;
		notice?: string | null;
		/** Set when the form sits in a modal, to swap to the other auth modal instead of navigating. */
		onSwitch?: (event: MouseEvent) => void;
	};
	let { enabled, notice = null, onSwitch }: Props = $props();

	let step = $state<'providers' | 'email' | 'details' | 'verify'>('providers');
	let code = $state('');
	let resendIn = $state(0);
	let showPassword = $state(false);
	let statusMessage = $state('');
	let displayName = $state('');
	let username = $state('');
	let wantsUpdates = $state(false);
	let email = $state('');
	let password = $state('');
	let pending = $state(false);

	// Cloudflare Turnstile: one widget per flow, its token posted with the forms that need it.
	let captchaToken = $state('');
	let captcha = $state<ReturnType<typeof Turnstile>>();
	function captchaReady() {
		if (!TURNSTILE_SITE_KEY || captchaToken) return true;
		statusMessage = 'Verifying you are human, one moment.';
		return false;
	}

	// Google, Apple and Discord share the sign-in route; a new account then picks a username.
	function signUpWith(provider: 'google' | 'apple' | 'discord') {
		window.location.assign(`/auth/${provider}/start`);
	}

	function goTo(next: typeof step) {
		statusMessage = '';
		step = next;
	}

	function submitEmail(event: SubmitEvent) {
		event.preventDefault();
		goTo('details');
	}

	// Counts down the "resend code" wait; the API allows a new code once a minute.
	let countdown: ReturnType<typeof setInterval> | undefined;
	function startResendCountdown() {
		clearInterval(countdown);
		resendIn = 60;
		countdown = setInterval(() => {
			resendIn -= 1;
			if (resendIn <= 0) clearInterval(countdown);
		}, 1000);
	}
	onDestroy(() => clearInterval(countdown));

	const failure = (data: Record<string, unknown> | undefined) =>
		typeof data?.error === 'string' ? data.error : 'Something went wrong. Please try again.';

	// Errors that belong to the email step send the visitor back there.
	const EMAIL_ERRORS = new Set(['invalid_email', 'email_taken']);

	// Details step and "Tekrar gönder": checks the details and mails the code.
	const submitStart: SubmitFunction = ({ cancel }) => {
		if (!captchaReady()) {
			cancel();
			return;
		}
		pending = true;
		statusMessage = '';
		return async ({ result }) => {
			captcha?.reset();
			pending = false;
			if (result.type === 'success') {
				startResendCountdown();
				if (step === 'verify') {
					statusMessage = 'A new code was sent.';
				} else {
					code = '';
					goTo('verify');
				}
				return;
			}
			const data = result.type === 'failure' ? result.data : undefined;
			if (typeof data?.code === 'string' && EMAIL_ERRORS.has(data.code)) step = 'email';
			else if (step === 'verify' && data?.code !== 'code_cooldown') step = 'details';
			statusMessage = failure(data);
		};
	};

	// Verify step: the right code creates the account.
	const submitVerify: SubmitFunction = ({ cancel }) => {
		if (code.length < 6) {
			statusMessage = 'Enter the 6-digit code from your email.';
			cancel();
			return;
		}
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
			const data = result.type === 'failure' ? result.data : undefined;
			if (data?.code === 'invalid_code') code = '';
			else if (data?.code === 'username_taken') step = 'details';
			else if (data?.code === 'email_taken') step = 'email';
			statusMessage = failure(data);
		};
	};
</script>

<div lang="en">
	<!-- "Resend code" posts the same details again; kept outside the code form so Enter there verifies. -->
	<form id="signup-resend" method="POST" action="/sign-up?/start" hidden use:enhance={submitStart}>
		<input type="hidden" name="cf-turnstile-response" value={captchaToken} />
		<input type="hidden" name="email" value={email} />
		<input type="hidden" name="username" value={username} />
		<input type="hidden" name="password" value={password} />
		<input type="hidden" name="display_name" value={displayName} />
		<input type="hidden" name="marketing_opt_in" value={wantsUpdates ? 'true' : ''} />
	</form>

	{#if step === 'providers'}
		<div in:fade={{ duration: 150 }}>
			<h1
				class="mb-6 text-center text-title-2 font-semibold tracking-tight text-balance text-primary"
			>
				Create your account
			</h1>

			{#if enabled.google}
				<Button size="lg" class="w-full" onclick={() => signUpWith('google')}
					><GoogleIcon />Sign up with Google</Button
				>

				<Separator class="mx-auto my-6 w-3/4!" />
			{/if}

			<div class="space-y-3">
				<Button variant="outline" size="lg" class="w-full" onclick={() => goTo('email')}
					><MailIcon />Sign up with email</Button
				>
				{#if enabled.apple}
					<Button variant="outline" size="lg" class="w-full" onclick={() => signUpWith('apple')}
						><AppleIcon />Sign up with Apple</Button
					>
				{/if}
				{#if enabled.discord}
					<Button variant="outline" size="lg" class="w-full" onclick={() => signUpWith('discord')}
						><DiscordIcon />Sign up with Discord</Button
					>
				{/if}
			</div>
		</div>
	{:else if step === 'email'}
		<form class="space-y-3" onsubmit={submitEmail} in:fade={{ duration: 150 }}>
			<h1
				class="mb-6 text-center text-title-2 font-semibold tracking-tight text-balance text-primary"
			>
				Sign up with email
			</h1>
			<div class="space-y-2">
				<Label for="signup-email">Email</Label>
				<Input
					id="signup-email"
					name="email"
					type="email"
					bind:value={email}
					autocomplete="email"
					required
					autofocus
				/>
			</div>

			<Button type="submit" size="lg" class="w-full">Continue</Button>

			<Button variant="outline" size="lg" class="w-full" onclick={() => goTo('providers')}
				>Go back</Button
			>
		</form>
	{:else if step === 'details'}
		<form
			method="POST"
			action="/sign-up?/start"
			class="space-y-3"
			use:enhance={submitStart}
			in:fade={{ duration: 150 }}
		>
			<input type="hidden" name="cf-turnstile-response" value={captchaToken} />
			<input type="hidden" name="email" value={email} />
			<input type="hidden" name="marketing_opt_in" value={wantsUpdates ? 'true' : ''} />
			<div class="mb-8 text-center">
				<h1 class="text-title-2 font-semibold tracking-tight text-balance text-primary">
					Finish signing up
				</h1>
				<span
					class="mt-4 inline-flex max-w-full items-center gap-2 rounded-full border px-4 py-1.5 text-footnote"
				>
					<MailIcon class="size-4 shrink-0" />
					<span class="truncate">{email}</span>
				</span>
			</div>
			<div class="grid grid-cols-2 gap-3">
				<div class="space-y-2">
					<Label for="signup-name">Display name</Label>
					<Input
						id="signup-name"
						name="display_name"
						bind:value={displayName}
						maxlength={40}
						autocomplete="nickname"
						autofocus
					/>
				</div>
				<div class="space-y-2">
					<Label for="signup-username">Username</Label>
					<Input
						id="signup-username"
						name="username"
						bind:value={username}
						minlength={3}
						maxlength={30}
						autocomplete="username"
						required
					/>
				</div>
			</div>
			<div class="space-y-2">
				<Label for="signup-password">Password</Label>
				<InputGroup.Root>
					<InputGroup.Input
						id="signup-password"
						name="password"
						type={showPassword ? 'text' : 'password'}
						bind:value={password}
						autocomplete="new-password"
						minlength={8}
						required
					/>
					<InputGroup.Addon align="inline-end">
						<InputGroup.Button
							size="icon-xs"
							class="cursor-pointer rounded-full"
							aria-label={showPassword ? 'Hide password' : 'Show password'}
							onclick={() => (showPassword = !showPassword)}
						>
							{#if showPassword}
								<EyeOffIcon />
							{:else}
								<EyeIcon />
							{/if}
						</InputGroup.Button>
					</InputGroup.Addon>
				</InputGroup.Root>
			</div>
			<div class="flex items-center gap-2.5 py-1">
				<Checkbox id="signup-updates" bind:checked={wantsUpdates} />
				<Label for="signup-updates" class="cursor-pointer text-callout font-normal"
					>Email me about new features and announcements</Label
				>
			</div>

			<Button type="submit" size="lg" class="w-full" disabled={pending}
				>{pending ? 'Sending code…' : 'Continue'}</Button
			>

			<Button variant="outline" size="lg" class="w-full" onclick={() => goTo('email')}
				>Go back</Button
			>
		</form>
	{:else}
		<form
			method="POST"
			action="/sign-up?/verify"
			class="space-y-3"
			use:enhance={submitVerify}
			in:fade={{ duration: 150 }}
		>
			<input type="hidden" name="email" value={email} />
			<div class="text-center">
				<h1 class="text-title-2 font-semibold tracking-tight text-balance text-primary">
					Verify your email
				</h1>
				<p class="mt-3 text-callout text-pretty text-muted-foreground">
					We sent a one-time security code to <span class="font-medium text-foreground"
						>{email}</span
					>. If it doesn't arrive within a few minutes, check your spam folder. Enter the code
					below:
				</p>
			</div>

			<div class="flex justify-center pt-4">
				<InputOTP.Root
					name="code"
					maxlength={6}
					bind:value={code}
					pattern="^[0-9]*$"
					pasteTransformer={(text) => text.replace(/\D/g, '')}
					autofocus
				>
					{#snippet children({ cells })}
						<InputOTP.Group>
							{#each cells.slice(0, 3) as cell (cell)}
								<InputOTP.Slot {cell} class="size-11 text-base" />
							{/each}
						</InputOTP.Group>
						<InputOTP.Separator />
						<InputOTP.Group>
							{#each cells.slice(3, 6) as cell (cell)}
								<InputOTP.Slot {cell} class="size-11 text-base" />
							{/each}
						</InputOTP.Group>
					{/snippet}
				</InputOTP.Root>
			</div>

			<p class="text-center text-footnote text-muted-foreground">
				Didn't get the code?
				<Button
					type="submit"
					form="signup-resend"
					variant="link"
					class="h-auto cursor-pointer p-0 font-medium text-foreground disabled:font-normal disabled:text-muted-foreground disabled:no-underline"
					disabled={resendIn > 0 || pending}
					>Resend code{resendIn > 0 ? ` (${resendIn})` : ''}</Button
				>
			</p>

			<Button type="submit" size="lg" class="w-full" disabled={pending}>Verify email</Button>

			<Button variant="outline" size="lg" class="w-full" onclick={() => goTo('details')}
				>Go back</Button
			>
		</form>
	{/if}

	{#if step === 'details' || step === 'verify'}
		<div class="mt-4"><Turnstile bind:this={captcha} bind:token={captchaToken} /></div>
	{/if}

	{#if statusMessage}
		<p class="mt-5 text-center text-footnote text-muted-foreground" aria-live="polite">
			{statusMessage}
		</p>
	{/if}

	<p class="mt-8 text-center text-footnote text-muted-foreground">
		Already have an account?

		<a href="/sign-in" onclick={onSwitch} class="font-medium text-foreground hover:underline"
			>Log in</a
		>
	</p>

	<AuthTerms />
</div>
