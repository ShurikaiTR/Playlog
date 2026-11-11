// The search dialog is rendered once in the app shell; the sidebar button and Cmd/Ctrl+K open it.
export const searchDialog = $state({ open: false });

export function openSearch() {
	searchDialog.open = true;
}
