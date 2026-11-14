<script lang="ts">
	import AdminPageHeader from '#lib/features/admin/admin-page-header.svelte';
	import GameForm from '#lib/features/admin/game-form.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<AdminPageHeader title="New game" description="It starts as a draft unless you publish it." />
<GameForm
	initial={data.initial}
	platforms={data.platforms}
	submitLabel="Create game"
	error={form?.error ?? null}
/>
