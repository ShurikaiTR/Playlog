// The language picker and feedback form are rendered once in the app shell,
// so both the header settings menu and the sidebar account menu can open them.
export const settingsDialogs = $state({ language: false, feedback: false });

export function openLanguageDialog() {
	settingsDialogs.language = true;
}

export function openFeedbackDialog() {
	settingsDialogs.feedback = true;
}
