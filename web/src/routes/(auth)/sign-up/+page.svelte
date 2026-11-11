<script lang="ts">
	import SignUpFlow from '#lib/features/auth/sign-up-flow.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<svelte:head>
	<title>Sign up | Playthru</title>
	<meta name="description" content="Create a free Playthru account." />
</svelte:head>

<SignUpFlow enabled={data.providers} returnTo={data.returnTo} />
