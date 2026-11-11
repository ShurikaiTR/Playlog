<script lang="ts">
	import { cn, type WithElementRef, type WithoutChildren } from '#lib/utils.js';
	import type { HTMLTextareaAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		value = $bindable(),
		class: className,
		'data-slot': dataSlot = 'textarea',
		...restProps
	}: WithoutChildren<WithElementRef<HTMLTextareaAttributes>> = $props();
</script>

<textarea
	bind:this={ref}
	data-slot={dataSlot}
	class={cn(
		'flex field-sizing-content min-h-16 w-full resize-none rounded-2xl bg-transparent p-4 text-sm field-outline outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50 pointer-coarse:text-[16px]',
		className
	)}
	bind:value
	{...restProps}></textarea>
