<script lang="ts">
	import EyeIcon from '@lucide/svelte/icons/eye';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import * as InputGroup from '#lib/components/ui/input-group/index.js';

	let {
		id,
		name,
		class: className,
		autofocus = false,
		value = $bindable(''),
		autocomplete = 'current-password',
		placeholder
	}: {
		id: string;
		name?: string;
		class?: string;
		autofocus?: boolean;
		value?: string;
		autocomplete?: 'current-password' | 'new-password';
		placeholder?: string;
	} = $props();

	let visible = $state(false);
</script>

<InputGroup.Root class={className}>
	<InputGroup.Input
		{id}
		{name}
		{autofocus}
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
			aria-label={visible ? 'Hide password' : 'Show password'}
			aria-pressed={visible}
			onclick={() => (visible = !visible)}
		>
			{#if visible}<EyeOffIcon />{:else}<EyeIcon />{/if}
		</InputGroup.Button>
	</InputGroup.Addon>
</InputGroup.Root>
