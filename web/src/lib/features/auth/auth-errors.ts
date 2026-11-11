// English messages for the Rust API's error codes on the sign-in and sign-up pages.
const messages: Record<string, string> = {
	invalid_credentials: 'Incorrect email or password.',
	invalid_email: 'Enter a valid email address.',
	invalid_password: 'Your password must be at least 8 characters.',
	invalid_username:
		'Your username must be 3-30 characters; letters, numbers, underscores and dots are allowed.',
	reserved_username: "That username isn't available. Try another one.",
	invalid_display_name: 'Your display name can be at most 40 characters.',
	username_taken: 'That username is taken. Try another one.',
	email_taken: 'An account with this email already exists. Try logging in.',
	invalid_code: 'That code is wrong or has expired. You can request a new one.',
	code_cooldown: 'Wait a minute before requesting a new code.',
	captcha_failed: 'Verification failed. Complete the verification and try again.',
	rate_limited: 'Too many attempts. Wait a bit and try again.',
	oauth_failed: "We couldn't complete the login. Please try again.",
	invalid_state: 'Your login session expired. Please try again.',
	invalid_pending: 'Your sign-up expired. Please start over.',
	access_denied: 'Login was cancelled.',
	// The provider sent no email, or one it has not verified (Discord: verify it in Discord first).
	email_unverified:
		"We couldn't get a verified email from that account. Verify your email with the provider, or use another way to sign up.",
	unknown_provider: "That login method isn't available right now. You can continue with email.",
	unavailable: "We can't reach the server right now. Try again in a moment."
};

export function authErrorMessage(code: string | undefined) {
	return (code && messages[code]) || 'Something went wrong. Please try again.';
}

/** The message a failed form action returned (`fail(…, { error })`), or a generic one. */
export const formErrorMessage = (data: Record<string, unknown> | undefined) =>
	typeof data?.error === 'string' ? data.error : 'Something went wrong. Please try again.';
