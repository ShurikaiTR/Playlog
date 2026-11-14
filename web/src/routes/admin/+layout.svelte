<script lang="ts">
	import { page } from '$app/state';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import AdminSidebar from '#lib/features/admin/admin-sidebar.svelte';
	import AdminSiteHeader from '#lib/features/admin/admin-site-header.svelte';
	import { sectionTitle } from '#lib/features/admin/admin-nav.js';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();
</script>

<svelte:head>
	<title>{sectionTitle(page.url.pathname)} · Admin · Playthru</title>
	<meta name="robots" content="noindex" />
</svelte:head>

<Sidebar.Provider
	style="--sidebar-width: calc(var(--spacing) * 72); --header-height: calc(var(--spacing) * 12);"
>
	<AdminSidebar variant="inset" staff={data.staff} />
	<Sidebar.Inset>
		<AdminSiteHeader title={sectionTitle(page.url.pathname)} />
		<div class="flex flex-1 flex-col">
			<div class="@container/main flex flex-1 flex-col gap-2">
				<div class="flex flex-col gap-4 px-4 py-4 md:gap-6 md:py-6 lg:px-6">
					{@render children()}
				</div>
			</div>
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
