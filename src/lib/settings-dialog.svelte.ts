// The settings dialog is rendered once in the sidebar; the account menu opens it at a section.
// More sections (account, profile, data) are added here as they get built.
export type SettingsSection = 'appearance';

export const settingsDialog = $state<{ open: boolean; section: SettingsSection }>({
	open: false,
	section: 'appearance'
});

export function openSettings(section: SettingsSection = 'appearance') {
	settingsDialog.section = section;
	settingsDialog.open = true;
}
