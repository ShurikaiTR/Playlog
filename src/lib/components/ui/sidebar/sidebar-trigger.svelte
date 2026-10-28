<script lang="ts">
	import { mergeProps } from 'bits-ui';
	import SidebarToggleIcon from '#lib/components/icons/sidebar-toggle.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { cn } from '#lib/utils.js';
	import { useSidebar } from './context.svelte.js';
	import type { ComponentProps } from 'svelte';

	let {
		ref = $bindable(null),
		class: className,
		onclick,
		...restProps
	}: ComponentProps<typeof Button> & {
		onclick?: (e: MouseEvent) => void;
	} = $props();

	const sidebar = useSidebar();

	function handleClick(e: MouseEvent) {
		onclick?.(e);
		sidebar.toggle();
	}
</script>

<Tooltip.Root>
	<Tooltip.Trigger>
		{#snippet child({ props })}
			{@const mergedProps = mergeProps(restProps, props, { onclick: handleClick })}
			<Button
				bind:ref
				data-sidebar="trigger"
				data-slot="sidebar-trigger"
				variant="ghost"
				size="icon-xs"
				class={cn('text-muted-foreground hover:text-primary hover:bg-foreground/8 dark:hover:bg-foreground/12 size-8 cursor-pointer rounded-xl transition-colors duration-100', className)}
				type="button"
				{...mergedProps}
			>
				<SidebarToggleIcon class="cn-rtl-flip size-[18px]!" aria-hidden="true" />
				<span class="sr-only">{sidebar.state === 'expanded' ? 'Daralt' : 'Genişlet'}</span>
			</Button>
		{/snippet}
	</Tooltip.Trigger>
	<Tooltip.Content side="right" sideOffset={8}>
		{sidebar.state === 'expanded' ? 'Daralt' : 'Genişlet'}
	</Tooltip.Content>
</Tooltip.Root>
