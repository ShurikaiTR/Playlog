<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import DiscordIcon from '#lib/components/icons/discord.svelte';
	import GoogleIcon from '#lib/components/icons/google.svelte';

	let { onselect }: { onselect: (provider: 'Google' | 'Discord') => void } = $props();
</script>

<div class="grid gap-3">
	<Button variant="outline" size="lg" class="h-11 w-full" onclick={() => onselect('Google')}>
		<GoogleIcon class="size-4.5" />
		Google ile devam et
	</Button>
	<Button variant="outline" size="lg" class="h-11 w-full" onclick={() => onselect('Discord')}>
		<DiscordIcon class="size-4.5" />
		Discord ile devam et
	</Button>
</div>
