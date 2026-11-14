<script lang="ts">
	import AdminActivityChart from '#lib/features/admin/admin-activity-chart.svelte';
	import AdminSectionCards from '#lib/features/admin/admin-section-cards.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

{#if data.stats}
	<AdminSectionCards stats={data.stats} isAdmin={data.isAdmin}>
		{#snippet chart()}
			<AdminActivityChart activity={data.stats!.activity} />
		{/snippet}
	</AdminSectionCards>
{:else}
	<p class="text-callout text-muted-foreground" role="alert">
		The numbers could not be loaded. Try again.
	</p>
{/if}
