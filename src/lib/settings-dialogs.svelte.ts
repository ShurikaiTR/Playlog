// The language picker and feedback form are rendered once in the app shell,
// so both the header settings menu and the sidebar account menu can open them.
export const settingsDialogs = $state({ language: false, feedback: false });

// TODO: placeholder, the app is English only for now. The choice is just remembered until translations exist.
export const languages = [
	{ value: 'en', label: 'English', native: 'English' },
	{ value: 'tr', label: 'Türkçe', native: 'Turkish' }
];

// The chosen language, shared so the settings dialog can show it next to the picker button.
export const languageChoice = $state({ value: 'en' });

export function openLanguageDialog() {
	settingsDialogs.language = true;
}

export function openFeedbackDialog() {
	settingsDialogs.feedback = true;
}
