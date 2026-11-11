<script lang="ts">
	import SignInFlow from '#lib/components/auth/sign-in-flow.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<svelte:head>
	<title>Log in | Playthru</title>
	<meta name="description" content="Log in to your Playthru account." />
</svelte:head>

<SignInFlow enabled={data.providers} notice={data.notice} returnTo={data.returnTo} />
