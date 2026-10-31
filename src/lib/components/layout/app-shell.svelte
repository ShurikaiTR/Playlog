<script lang="ts">
	import type { Snippet } from "svelte";
	import { Button } from "#lib/components/ui/button/index.js";
	import * as Sidebar from "#lib/components/ui/sidebar/index.js";
	import AppSidebar from "./app-sidebar.svelte";
	import Footer from "#lib/components/footer.svelte";
	import SettingsMenu from "./settings-menu.svelte";
	import { ModeWatcher } from "mode-watcher";

	let { children }: { children: Snippet } = $props();
</script>

<ModeWatcher defaultMode="dark" />
<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset
		class="md:[--sidebar-offset:var(--sidebar-width)] md:peer-data-[state=collapsed]:[--sidebar-offset:calc(var(--sidebar-width-icon)+1rem)]"
	>
		<header class="relative z-20 flex h-16 shrink-0 items-center gap-3 px-4 md:px-6">
			<Sidebar.Trigger class="md:hidden" />
			<div class="ml-auto flex items-center gap-2">
				<SettingsMenu />
				<Button href="/giris" variant="outline" size="lg">Giriş yap</Button>
				<Button href="/kayit-ol" size="lg">Kayıt ol</Button>
			</div>
		</header>

		<main>{@render children()}</main>
		<Footer />
	</Sidebar.Inset>
</Sidebar.Provider>
