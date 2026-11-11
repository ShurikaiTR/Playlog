<script lang="ts">
	import type { Snippet } from "svelte";
	import { Button } from "#lib/components/ui/button/index.js";
	import * as Sidebar from "#lib/components/ui/sidebar/index.js";
	import AppSidebar from "./app-sidebar.svelte";
	import Footer from "./footer.svelte";
	import SettingsMenu from "#lib/features/settings/settings-menu.svelte";
	import SettingsDialogs from "#lib/features/settings/settings-dialogs.svelte";
	import SearchDialog from "#lib/features/search/search-dialog.svelte";
	import { page } from "$app/state";
	import { currentReturnPath, openAuthModal } from "#lib/features/auth/open-auth-modal.js";
	import { authHref } from "#lib/features/auth/return-to.js";

	let { children }: { children: Snippet } = $props();
	const returnPath = $derived(currentReturnPath());
</script>

<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset
		class="md:[--sidebar-offset:var(--sidebar-width)] md:peer-data-[state=collapsed]:[--sidebar-offset:calc(var(--sidebar-width-icon)+1rem)]"
	>
		<!-- Signed in, the header stays empty: account and settings live at the bottom of the sidebar. -->
		<header class="relative z-20 flex h-16 shrink-0 items-center gap-3 px-4 md:px-6">
			<Sidebar.Trigger class="md:hidden" />
			{#if !page.data.user}
				<div class="ml-auto flex items-center gap-2">
					<SettingsMenu />
					<Button href={authHref("sign-in", returnPath)} variant="outline" size="lg" onclick={(event) => openAuthModal(event, "sign-in")}>Log in</Button>
					<Button href={authHref("sign-up", returnPath)} size="lg" onclick={(event) => openAuthModal(event, "sign-up")}>Sign up</Button>
				</div>
			{/if}
		</header>

		<main>{@render children()}</main>
		<Footer />
	</Sidebar.Inset>
</Sidebar.Provider>

<SettingsDialogs />
<SearchDialog />
