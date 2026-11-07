<script lang="ts">
	import SignUpFlow from '#lib/components/auth/sign-up-flow.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<svelte:head>
	<title>Kayıt ol | Playthru</title>
	<meta name="description" content="Ücretsiz bir Playthru hesabı oluştur." />
</svelte:head>

<SignUpFlow enabled={data.providers} />
