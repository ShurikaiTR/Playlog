<script lang="ts">
	import SignInFlow from '#lib/components/auth/sign-in-flow.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<svelte:head>
	<title>Giriş yap | Playthru</title>
	<meta name="description" content="Playthru hesabına giriş yap." />
</svelte:head>

<SignInFlow enabled={data.providers} notice={data.notice} />
