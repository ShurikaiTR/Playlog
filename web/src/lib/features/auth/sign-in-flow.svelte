<script lang="ts">
	import GoogleIcon from '#lib/components/icons/google.svelte';
	import AppleIcon from '#lib/components/icons/apple.svelte';
	import DiscordIcon from '#lib/components/icons/discord.svelte';
	import SteamIcon from '#lib/components/icons/steam.svelte';
	import SignInOptions from './sign-in-options.svelte';
	import MailIcon from '#lib/components/icons/mail.svelte';
	import { hideAuthTransition, showAuthTransition } from './auth-transition.svelte.js';
	import { applyAction, enhance, type SubmitFunction } from '$app/forms';
	import { onDestroy, onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import Turnstile from './turnstile.svelte';
	import { authHref } from './return-to.js';
	import AuthShell from './auth-shell.svelte';
	import PasswordInput from './password-input.svelte';
	import { TURNSTILE_SITE_KEY } from '$app/env/public';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as InputOTP from '#lib/components/ui/input-otp/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Separator } from '#lib/components/ui/separator/index.js';

	type Props = {
		enabled: Record<'google' | 'apple' | 'discord' | 'steam', boolean>;
		notice?: string | null;
		/** Where to go after logging in (a same-origin path, already validated). */
		returnTo?: string;
		/** Set when the form sits in a modal, to swap to the other auth modal instead of navigating. */
		onSwitch?: (event: MouseEvent) => void;
	};
	let { enabled, notice = null, returnTo = '/', onSwitch }: Props = $props();

	// The page moves from the provider list to an email step, then a password step.
	// "Forgot password?" mails a code and opens the reset step.
	let step = $state<'providers' | 'password' | 'reset' | 'code'>('providers');
	// A message from a Google/Discord attempt that sent the visitor back here.
	// svelte-ignore state_referenced_locally
	let statusMessage = $state(notice ?? '');
	let email = $state('');
	let password = $state('');
	let resetCode = $state('');
	let loginCode = $state('');
	let newPassword = $state('');
	let resendIn = $state(0);
	let pending = $state(false);

	// Cloudflare Turnstile: one widget per flow, its token posted with the forms that need it.
	let captchaToken = $state('');
	let captcha = $state<ReturnType<typeof Turnstile>>();
	function captchaReady() {
		if (!TURNSTILE_SITE_KEY || captchaToken) return true;
		statusMessage = 'Verifying you are human, one moment.';
		return false;
	}

	type ProviderId = 'google' | 'apple' | 'discord' | 'steam' | 'email';
	const providers: { id: ProviderId; label: string; icon: typeof GoogleIcon }[] = [
		{ id: 'google', label: 'Google', icon: GoogleIcon },
		{ id: 'apple', label: 'Apple', icon: AppleIcon },
		{ id: 'discord', label: 'Discord', icon: DiscordIcon },
		{ id: 'steam', label: 'Steam', icon: SteamIcon },
		{ id: 'email', label: 'Email', icon: MailIcon }
	];
	// E-mail always works; the others only when their keys are configured on the API.
	const shownProviders = $derived(
		providers.filter((provider) => provider.id !== 'email' && enabled[provider.id])
	);
	const LAST_PROVIDER_KEY = 'playthru-last-login-provider';
	let lastProvider = $state<ProviderId | null>(null);

	onMount(() => {
		try {
			const saved = localStorage.getItem(LAST_PROVIDER_KEY);
			if (providers.some((provider) => provider.id === saved)) lastProvider = saved as ProviderId;
		} catch {
			// Storage can be unavailable (private mode); the hint is optional.
		}
	});

	function rememberProvider(id: ProviderId) {
		lastProvider = id;
		try {
			localStorage.setItem(LAST_PROVIDER_KEY, id);
		} catch {
			// Ignore: the hint just won't persist.
		}
	}

	async function chooseProvider(id: ProviderId) {
		if (id === 'email') {
			goTo('password');
			return;
		}
		rememberProvider(id);
		// A full page load: the server route sends the visitor on to the provider.
		window.location.assign(`/auth/${id}/start?return_to=${encodeURIComponent(returnTo)}`);
	}

	function goTo(next: typeof step) {
		statusMessage = '';
		step = next;
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

	// Signs in from the password step.
	const submitLogin: SubmitFunction = ({ cancel }) => {
		if (!captchaReady()) {
			cancel();
			return;
		}
		pending = true;
		statusMessage = '';
		showAuthTransition({
			title: 'Logging in',
			description: 'Checking your account, please wait…'
		});
		return async ({ result }) => {
			captcha?.reset();
			pending = false;
			if (result.type === 'redirect') {
				rememberProvider('email');
				await applyAction(result);
				hideAuthTransition();
				return;
			}
			hideAuthTransition();
			statusMessage = failure(result.type === 'failure' ? result.data : undefined);
		};
	};

	// "Forgot password?" and "Resend code" both mail a reset code through the hidden form
	// below, so pressing Enter in a step's own form never triggers them.
	const submitResetStart: SubmitFunction = ({ cancel }) => {
		if (!captchaReady()) {
			cancel();
			return;
		}
		if (!email.trim()) {
			statusMessage = 'Enter your email address first.';
			cancel();
			return;
		}
		pending = true;
		statusMessage = '';
		return async ({ result }) => {
			captcha?.reset();
			pending = false;
			if (result.type !== 'success') {
				statusMessage = failure(result.type === 'failure' ? result.data : undefined);
				return;
			}
			startResendCountdown();
			if (step === 'reset') {
				statusMessage = 'A new code was sent.';
			} else {
				resetCode = '';
				newPassword = '';
				goTo('reset');
			}
		};
	};

	// "Email login code" and "Resend code" mail a sign-in code.
	const submitCodeStart: SubmitFunction = ({ cancel }) => {
		if (!captchaReady()) {
			cancel();
			return;
		}
		if (!email.trim()) {
			statusMessage = 'Enter your email address first.';
			cancel();
			return;
		}
		pending = true;
		statusMessage = '';
		return async ({ result }) => {
			captcha?.reset();
			pending = false;
			if (result.type !== 'success') {
				statusMessage = failure(result.type === 'failure' ? result.data : undefined);
				return;
			}
			startResendCountdown();
			if (step === 'code') {
				statusMessage = 'A new code was sent.';
			} else {
				loginCode = '';
				goTo('code');
			}
		};
	};

	// Signs in with the emailed code.
	const submitCodeVerify: SubmitFunction = ({ cancel }) => {
		if (loginCode.length < 6) {
			cancel();
			return;
		}
		pending = true;
		statusMessage = '';
		showAuthTransition({
			title: 'Logging in',
			description: 'Checking your code, please wait…'
		});
		return async ({ result }) => {
			pending = false;
			if (result.type === 'redirect') {
				rememberProvider('email');
				await applyAction(result);
				hideAuthTransition();
				return;
			}
			hideAuthTransition();
			const data = result.type === 'failure' ? result.data : undefined;
			if (data?.code === 'invalid_code') loginCode = '';
			statusMessage = failure(data);
		};
	};

	// Sets the new password from the reset step.
	const submitReset: SubmitFunction = ({ cancel }) => {
		if (resetCode.length < 6) {
			statusMessage = 'Enter the 6-digit code from your email.';
			cancel();
			return;
		}
		pending = true;
		statusMessage = '';
		return async ({ result }) => {
			pending = false;
			if (result.type === 'success') {
				password = '';
				goTo('password');
				statusMessage = 'Your password was changed. You can now log in with it.';
			} else {
				const data = result.type === 'failure' ? result.data : undefined;
				if (data?.code === 'invalid_code') resetCode = '';
				statusMessage = failure(data);
			}
		};
	};
</script>

<div lang="en">
	<form
		id="reset-start"
		method="POST"
		action="/sign-in?/resetStart"
		hidden
		use:enhance={submitResetStart}
	>
		<input type="hidden" name="cf-turnstile-response" value={captchaToken} />
		<input type="hidden" name="email" value={email} />
	</form>

	<form
		id="code-start"
		method="POST"
		action="/sign-in?/codeStart"
		hidden
		use:enhance={submitCodeStart}
	>
		<input type="hidden" name="cf-turnstile-response" value={captchaToken} />
		<input type="hidden" name="email" value={email} />
	</form>

	{#snippet status()}
		<div class="min-h-5 text-center text-xs text-muted-foreground" aria-live="polite">
			{statusMessage}
		</div>
	{/snippet}

	{#snippet signUpRow()}
		<div class="flex h-12 flex-col items-center justify-center">
			<p class="text-center text-footnote text-muted-foreground">
				Don't have an account?
				<a
					href={authHref('sign-up', returnTo)}
					onclick={onSwitch}
					class="font-medium text-primary underline underline-offset-4">Sign up</a
				>
			</p>
		</div>
	{/snippet}

	{#if step === 'providers'}
		<div in:fade={{ duration: 150 }}>
			<AuthShell title="Log in">
				<SignInOptions
					providers={shownProviders}
					{lastProvider}
					bind:email
					message={statusMessage}
					onProvider={(id) => chooseProvider(id as ProviderId)}
					onEmail={() => goTo('password')}
					signUpHref={authHref('sign-up', returnTo)}
					{onSwitch}
				/>
			</AuthShell>
		</div>
	{:else if step === 'password'}
		<div in:fade={{ duration: 150 }}>
			<AuthShell title="Log in with email" onBack={() => goTo('providers')}>
				{#snippet subtitle()}Enter your password to log in.{/snippet}
				<form
					method="POST"
					action="/sign-in?/login"
					class="flex flex-1 flex-col"
					use:enhance={submitLogin}
				>
					<input type="hidden" name="return_to" value={returnTo} />
					<input type="hidden" name="cf-turnstile-response" value={captchaToken} />
					<input type="hidden" name="email" value={email} />
					<div class="mt-auto flex flex-col gap-3">
						<Button
							type="submit"
							form="code-start"
							variant="outline"
							size="lg"
							class="w-full"
							disabled={pending}><MailIcon />Email login code</Button
						>

						<div class="flex items-center gap-3" aria-hidden="true">
							<Separator class="flex-1" />
							<span class="text-xs text-muted-foreground">or</span>
							<Separator class="flex-1" />
						</div>

						<div class="flex flex-col gap-1.5">
							<div class="flex min-h-5 items-center justify-between gap-2">
								<Label for="login-password" class="text-xs font-medium">Password</Label>
								<Button
									type="submit"
									form="reset-start"
									variant="link"
									class="h-auto cursor-pointer p-0 text-xs font-normal text-muted-foreground underline-offset-2 hover:text-foreground hover:underline"
									disabled={pending}>Forgot password?</Button
								>
							</div>
							<PasswordInput
								id="login-password"
								name="password"
								bind:value={password}
								autocomplete="current-password"
								class="h-9 rounded-xl"
								autofocus
							/>
							{@render status()}
						</div>

						<div class="min-h-16">
							<Turnstile bind:this={captcha} bind:token={captchaToken} />
						</div>
					</div>

					<div class="mt-auto flex flex-col gap-3 pt-5">
						<Button type="submit" size="lg" class="w-full" disabled={pending}
							>{pending ? 'Logging in…' : 'Log in'}</Button
						>
						{@render signUpRow()}
					</div>
				</form>
			</AuthShell>
		</div>
	{:else if step === 'code'}
		<div in:fade={{ duration: 150 }}>
			<AuthShell title="Enter your login code" onBack={() => goTo('password')}>
				{#snippet subtitle()}We sent a 6-digit code to
					<span class="font-medium text-foreground">{email}</span>.{/snippet}
				<form
					method="POST"
					action="/sign-in?/codeVerify"
					class="flex flex-1 flex-col"
					use:enhance={submitCodeVerify}
				>
					<input type="hidden" name="return_to" value={returnTo} />
					<input type="hidden" name="email" value={email} />
					<div class="mt-auto flex flex-col gap-1.5">
						<InputOTP.Root
							name="code"
							maxlength={6}
							bind:value={loginCode}
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
						<Button
							type="submit"
							size="lg"
							class="w-full"
							disabled={pending || loginCode.length < 6}>Log in</Button
						>
						<div class="flex h-12 flex-col">
							<Button
								type="submit"
								form="code-start"
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
	{:else}
		<div in:fade={{ duration: 150 }}>
			<AuthShell title="Reset your password" onBack={() => goTo('password')}>
				{#snippet subtitle()}If <span class="font-medium text-foreground">{email}</span> is linked to
					an account, we sent a one-time code to that address.{/snippet}
				<form
					method="POST"
					action="/sign-in?/resetConfirm"
					class="flex flex-1 flex-col"
					use:enhance={submitReset}
				>
					<input type="hidden" name="email" value={email} />
					<div class="mt-auto flex flex-col gap-3">
						<InputOTP.Root
							name="code"
							maxlength={6}
							bind:value={resetCode}
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

						<div class="flex flex-col gap-1.5">
							<Label for="reset-password" class="text-xs font-medium">New password</Label>
							<PasswordInput
								id="reset-password"
								name="password"
								bind:value={newPassword}
								autocomplete="new-password"
								class="h-9 rounded-xl"
							/>
							{@render status()}
						</div>

						<div class="min-h-16">
							<Turnstile bind:this={captcha} bind:token={captchaToken} />
						</div>
					</div>

					<div class="mt-auto flex flex-col gap-3 pt-5">
						<Button type="submit" size="lg" class="w-full" disabled={pending}
							>{pending ? 'Saving…' : 'Change password'}</Button
						>
						<div class="flex h-12 flex-col">
							<Button
								type="submit"
								form="reset-start"
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
