<script lang="ts">
	import { hideAuthTransition, showAuthTransition } from '#lib/auth-transition.svelte.js';
	import { applyAction, enhance, type SubmitFunction } from '$app/forms';
	import { onDestroy } from 'svelte';
	import { fade } from 'svelte/transition';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import Turnstile from './turnstile.svelte';
	import { authHref } from '#lib/return-to.js';
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
	import AuthShell from '#lib/components/auth/auth-shell.svelte';
	import PasswordInput from '#lib/components/auth/password-input.svelte';
	import MailIcon from '#lib/components/icons/mail.svelte';

	// Sign-up options, then email, then the account details (which mails the code), then the code.
	// The right code creates the account and signs in.
	type Props = {
		enabled: Record<'google' | 'apple' | 'discord', boolean>;
		/** Where to go after signing up (a same-origin path, already validated). */
		returnTo?: string;
		/** Set when the form sits in a modal, to swap to the other auth modal instead of navigating. */
		onSwitch?: (event: MouseEvent) => void;
	};
	let { enabled, returnTo = '/', onSwitch }: Props = $props();

	let step = $state<'providers' | 'details' | 'verify'>('providers');
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
		window.location.assign(`/auth/${provider}/start?return_to=${encodeURIComponent(returnTo)}`);
	}

	function goTo(next: typeof step) {
		statusMessage = '';
		step = next;
	}

	const emailValid = $derived(/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.trim()));

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
			if (typeof data?.code === 'string' && EMAIL_ERRORS.has(data.code)) step = 'providers';
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
			else if (data?.code === 'email_taken') step = 'providers';
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

	{#snippet status()}
		<div class="min-h-5 text-center text-xs text-muted-foreground" aria-live="polite">
			{statusMessage}
		</div>
	{/snippet}

	{#snippet loginRow()}
		<div class="flex h-12 flex-col items-center justify-center">
			<p class="text-center text-footnote text-muted-foreground">
				Already have an account?
				<a
					href={authHref('sign-in', returnTo)}
					onclick={onSwitch}
					class="font-medium text-primary underline underline-offset-4">Log in</a
				>
			</p>
		</div>
	{/snippet}

	{#if step === 'providers'}
		<div in:fade={{ duration: 150 }}>
			<AuthShell title="Create your account">
				<form class="flex flex-1 flex-col" onsubmit={submitEmail}>
					<div class="mt-auto flex flex-col gap-3">
						<div class="min-h-5"></div>
						{#if enabled.google}
							<Button
								variant="outline"
								size="lg"
								class="w-full"
								onclick={() => signUpWith('google')}><GoogleIcon />Sign up with Google</Button
							>
						{/if}
						{#if enabled.apple}
							<Button variant="outline" size="lg" class="w-full" onclick={() => signUpWith('apple')}
								><AppleIcon />Sign up with Apple</Button
							>
						{/if}
						{#if enabled.discord}
							<Button
								variant="outline"
								size="lg"
								class="w-full"
								onclick={() => signUpWith('discord')}><DiscordIcon />Sign up with Discord</Button
							>
						{/if}

						<div class="flex items-center gap-3" aria-hidden="true">
							<Separator class="flex-1" />
							<span class="text-xs text-muted-foreground">or</span>
							<Separator class="flex-1" />
						</div>

						<div class="flex flex-col gap-1.5">
							<Input
								id="signup-email"
								name="email"
								type="email"
								aria-label="Email"
								placeholder="Enter your email"
								bind:value={email}
								autocomplete="email"
								required
								class="h-9 rounded-xl"
							/>
							{@render status()}
						</div>
					</div>
					<div class="mt-auto flex flex-col gap-3 pt-5">
						<Button type="submit" size="lg" class="w-full" disabled={!emailValid}>Continue</Button>
						{@render loginRow()}
					</div>
				</form>
			</AuthShell>
		</div>
	{:else if step === 'details'}
		<div in:fade={{ duration: 150 }}>
			<AuthShell title="Finish signing up" onBack={() => goTo('providers')}>
				{#snippet subtitle()}<span class="font-medium text-foreground">{email}</span>{/snippet}
				<form
					method="POST"
					action="/sign-up?/start"
					class="flex flex-1 flex-col"
					use:enhance={submitStart}
				>
					<input type="hidden" name="cf-turnstile-response" value={captchaToken} />
					<input type="hidden" name="email" value={email} />
					<input type="hidden" name="marketing_opt_in" value={wantsUpdates ? 'true' : ''} />
					<div class="mt-auto flex flex-col gap-3">
						<div class="grid grid-cols-2 gap-3">
							<div class="flex flex-col gap-1.5">
								<Label for="signup-name" class="text-xs font-medium">Display name</Label>
								<Input
									id="signup-name"
									name="display_name"
									bind:value={displayName}
									maxlength={40}
									autocomplete="nickname"
									autofocus
									class="h-9 rounded-xl"
								/>
							</div>
							<div class="flex flex-col gap-1.5">
								<Label for="signup-username" class="text-xs font-medium">Username</Label>
								<Input
									id="signup-username"
									name="username"
									bind:value={username}
									minlength={3}
									maxlength={30}
									autocomplete="username"
									required
									class="h-9 rounded-xl"
								/>
							</div>
						</div>
						<div class="flex flex-col gap-1.5">
							<Label for="signup-password" class="text-xs font-medium">Password</Label>
							<PasswordInput
								id="signup-password"
								name="password"
								bind:value={password}
								autocomplete="new-password"
								class="h-9 rounded-xl"
							/>
							{@render status()}
						</div>
						<div class="flex items-center gap-2.5">
							<Checkbox id="signup-updates" bind:checked={wantsUpdates} />
							<Label for="signup-updates" class="cursor-pointer text-xs leading-4 font-normal"
								>Email me about new features and announcements</Label
							>
						</div>
						<div class="min-h-16">
							<Turnstile bind:this={captcha} bind:token={captchaToken} />
						</div>
					</div>
					<div class="mt-auto flex flex-col gap-3 pt-5">
						<Button type="submit" size="lg" class="w-full" disabled={pending}
							>{pending ? 'Sending code…' : 'Continue'}</Button
						>
						{@render loginRow()}
					</div>
				</form>
			</AuthShell>
		</div>
	{:else}
		<div in:fade={{ duration: 150 }}>
			<AuthShell title="Verify your email" onBack={() => goTo('details')}>
				{#snippet subtitle()}We sent a 6-digit code to
					<span class="font-medium text-foreground">{email}</span>. Check your spam folder if it
					doesn't arrive.{/snippet}
				<form
					method="POST"
					action="/sign-up?/verify"
					class="flex flex-1 flex-col"
					use:enhance={submitVerify}
				>
					<input type="hidden" name="return_to" value={returnTo} />
					<input type="hidden" name="email" value={email} />
					<div class="mt-auto flex flex-col gap-1.5">
						<InputOTP.Root
							name="code"
							maxlength={6}
							bind:value={code}
							pattern="^[0-9]*$"
							pasteTransformer={(text) => text.replace(/\D/g, '')}
							class="justify-center gap-1 sm:gap-2"
							autofocus
						>
							{#snippet children({ cells })}
								<InputOTP.Group class="gap-1 sm:gap-2">
									{#each cells.slice(0, 3) as cell (cell)}
										<InputOTP.Slot {cell} class="h-11 w-9 rounded-xl text-base sm:h-12 sm:w-11" />
									{/each}
								</InputOTP.Group>
								<InputOTP.Separator />
								<InputOTP.Group class="gap-1 sm:gap-2">
									{#each cells.slice(3, 6) as cell (cell)}
										<InputOTP.Slot {cell} class="h-11 w-9 rounded-xl text-base sm:h-12 sm:w-11" />
									{/each}
								</InputOTP.Group>
							{/snippet}
						</InputOTP.Root>
						{@render status()}
						<div class="min-h-16">
							<Turnstile bind:this={captcha} bind:token={captchaToken} />
						</div>
					</div>
					<div class="mt-auto flex flex-col gap-3 pt-5">
						<Button type="submit" size="lg" class="w-full" disabled={pending || code.length < 6}
							>Verify email</Button
						>
						<div class="flex h-12 flex-col">
							<Button
								type="submit"
								form="signup-resend"
								variant="outline"
								size="lg"
								class="w-full"
								disabled={resendIn > 0 || pending}
								>{resendIn > 0 ? `Resend code in ${resendIn}s` : 'Resend code'}</Button
							>
						</div>
					</div>
				</form>
			</AuthShell>
		</div>
	{/if}
</div>
