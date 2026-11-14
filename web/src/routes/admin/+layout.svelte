<script lang="ts">
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import AdminSidebar from '#lib/features/admin/admin-sidebar.svelte';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();
</script>

<svelte:head>
	<title>Admin · Playthru</title>
	<meta name="robots" content="noindex" />
</svelte:head>

<Sidebar.Provider>
	<AdminSidebar staff={data.staff} />
	<Sidebar.Inset
		class="md:[--sidebar-offset:var(--sidebar-width)] md:peer-data-[state=collapsed]:[--sidebar-offset:calc(var(--sidebar-width-icon)+1rem)]"
	>
		<header class="relative z-20 flex h-16 shrink-0 items-center gap-3 px-4 md:hidden">
			<Sidebar.Trigger />
		</header>
		<main class="mx-auto w-full max-w-6xl px-4 pb-12 md:px-6 md:pt-6">
			{@render children()}
		</main>
	</Sidebar.Inset>
</Sidebar.Provider>
