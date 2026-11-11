<script lang="ts">
	import type { Snippet } from 'svelte';
	import * as Drawer from '#lib/components/ui/drawer/index.js';
	import * as Sheet from '#lib/components/ui/sheet/index.js';
	import { IsMobile } from '#lib/hooks/is-mobile.svelte.js';
	import { cn } from '#lib/utils.js';

	// A panel for secondary content. On wide screens it is a sheet at the right edge; on phones it
	// is a drawer that rises from the bottom and can be dragged shut. The body brings its own
	// scroll container: a plain `min-h-0 flex-1 overflow-y-auto` element. (ScrollArea needs a fixed
	// height around it, which the drawer, sized by its content, does not have.)
	type Props = {
		open: boolean;
		title: string;
		description?: string;
		/** Extra controls under the title, such as sort tabs. */
		header?: Snippet;
		children: Snippet;
		footer?: Snippet;
		/** Classes for the sheet only; the drawer always spans the screen width. */
		sheetClass?: string;
		headerClass?: string;
		titleClass?: string;
		footerClass?: string;
	};
	let {
		open = $bindable(false),
		title,
		description,
		header,
		children,
		footer,
		sheetClass,
		headerClass,
		titleClass,
		footerClass
	}: Props = $props();

	const isMobile = new IsMobile();
</script>

{#if isMobile.current}
	<Drawer.Root bind:open shouldScaleBackground={false}>
		<Drawer.Content class="data-[vaul-drawer-direction=bottom]:max-h-[88svh]">
			<Drawer.Header class={headerClass}>
				<Drawer.Title class={titleClass}>{title}</Drawer.Title>
				{#if description}<Drawer.Description>{description}</Drawer.Description>{/if}
				{@render header?.()}
			</Drawer.Header>
			{@render children()}
			{#if footer}
				<Drawer.Footer class={cn('pb-[max(1rem,env(safe-area-inset-bottom))]', footerClass)}>
					{@render footer()}
				</Drawer.Footer>
			{/if}
		</Drawer.Content>
	</Drawer.Root>
{:else}
	<Sheet.Root bind:open>
		<Sheet.Content side="right" class={cn('gap-0', sheetClass)}>
			<Sheet.Header class={cn('pr-12', headerClass)}>
				<Sheet.Title class={titleClass}>{title}</Sheet.Title>
				{#if description}<Sheet.Description>{description}</Sheet.Description>{/if}
				{@render header?.()}
			</Sheet.Header>
			{@render children()}
			{#if footer}
				<Sheet.Footer class={footerClass}>{@render footer()}</Sheet.Footer>
			{/if}
		</Sheet.Content>
	</Sheet.Root>
{/if}
