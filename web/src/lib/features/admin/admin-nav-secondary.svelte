<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import type { WithoutChildren } from '#lib/utils.js';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';

	let { ...restProps }: WithoutChildren<ComponentProps<typeof Sidebar.Group>> = $props();
</script>

<Sidebar.Group {...restProps}>
	<Sidebar.GroupContent>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton tooltipContent="Back to Playthru">
					{#snippet child({ props })}
						<a {...props} href="/">
							<div data-sidebar="icon" aria-hidden="true"><ArrowLeftIcon /></div>
							<span>Back to Playthru</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.GroupContent>
</Sidebar.Group>
