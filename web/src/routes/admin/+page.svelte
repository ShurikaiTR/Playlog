<script lang="ts">
	import AdminActivityChart from '#lib/features/admin/admin-activity-chart.svelte';
	import AdminSectionCards from '#lib/features/admin/admin-section-cards.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

{#if data.stats}
	<AdminSectionCards stats={data.stats} />
	<AdminActivityChart activity={data.stats.activity} />
{:else}
	<p class="text-callout text-muted-foreground" role="alert">
		The numbers could not be loaded. Try again.
	</p>
{/if}
