// Full-screen "please wait" message shown while signing in, signing up or signing out.
type AuthTransition = { title: string; description: string };

export const authTransition = $state<{ current: AuthTransition | null }>({ current: null });

// Keeps the screen up for a moment so a fast request doesn't just flash it.
const MIN_VISIBLE_MS = 600;

export async function withAuthTransition<T>(transition: AuthTransition, task: () => Promise<T>) {
	authTransition.current = transition;
	const [result] = await Promise.all([
		task(),
		new Promise((resolve) => setTimeout(resolve, MIN_VISIBLE_MS))
	]);
	return result;
}

export function showAuthTransition(transition: AuthTransition) {
	authTransition.current = transition;
}

export function hideAuthTransition() {
	authTransition.current = null;
}
