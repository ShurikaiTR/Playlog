<script lang="ts">
	import type { Snippet } from "svelte";
	import * as Sidebar from "#lib/components/ui/sidebar/index.js";
	import AppSidebar from "./app-sidebar.svelte";
	import Footer from "#lib/components/footer.svelte";

	let { children }: { children: Snippet } = $props();
</script>

<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset
		class="md:[--sidebar-offset:var(--sidebar-width)] md:peer-data-[state=collapsed]:[--sidebar-offset:calc(var(--sidebar-width-icon)+1rem)]"
	>
		<header class="relative z-20 flex h-14 shrink-0 items-center px-4 md:hidden">
			<Sidebar.Trigger />
		</header>

		<main>{@render children()}</main>
		<Footer />
	</Sidebar.Inset>
</Sidebar.Provider>
