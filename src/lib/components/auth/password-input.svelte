<script lang="ts">
	import EyeIcon from '@lucide/svelte/icons/eye';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import * as InputGroup from '#lib/components/ui/input-group/index.js';

	let {
		id,
		value = $bindable(''),
		autocomplete = 'current-password',
		placeholder
	}: {
		id: string;
		value?: string;
		autocomplete?: 'current-password' | 'new-password';
		placeholder?: string;
	} = $props();

	let visible = $state(false);
</script>

<InputGroup.Root class="h-11">
	<InputGroup.Input
		{id}
		bind:value
		type={visible ? 'text' : 'password'}
		{autocomplete}
		{placeholder}
		minlength={autocomplete === 'new-password' ? 8 : undefined}
		required
	/>
	<InputGroup.Addon align="inline-end">
		<InputGroup.Button
			size="icon-xs"
			aria-label={visible ? 'Şifreyi gizle' : 'Şifreyi göster'}
			aria-pressed={visible}
			onclick={() => (visible = !visible)}
		>
			{#if visible}<EyeOffIcon />{:else}<EyeIcon />{/if}
		</InputGroup.Button>
	</InputGroup.Addon>
</InputGroup.Root>
