// The settings window is rendered once in the sidebar; the account menu opens it at a section.
// Managing the account itself ("Yönet") happens on the /settings page.
export type SettingsSection = 'account' | 'appearance';

export const settingsDialog = $state<{
	open: boolean;
	section: SettingsSection;
	/** On phones the section list is a screen of its own, shown before a section. */
	showList: boolean;
}>({
	open: false,
	section: 'account',
	showList: false
});

export function openSettings(section: SettingsSection = 'account') {
	settingsDialog.section = section;
	settingsDialog.showList = false;
	settingsDialog.open = true;
}

export function showSection(section: SettingsSection) {
	settingsDialog.section = section;
	settingsDialog.showList = false;
}
