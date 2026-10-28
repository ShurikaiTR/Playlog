<script lang="ts">
	import { cn, type WithElementRef } from '#lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();

	function attachRef(node: HTMLDivElement) {
		ref = node;

		return () => {
			if (ref === node) ref = null;
		};
	}
</script>

<div
	{@attach attachRef}
	data-slot="drawer-footer"
	class={cn('mt-auto flex flex-col gap-2 p-4', className)}
	{...restProps}
>
	{@render children?.()}
</div>
