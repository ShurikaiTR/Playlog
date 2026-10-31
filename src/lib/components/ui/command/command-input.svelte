<script lang="ts">
	import { Command as CommandPrimitive } from "bits-ui";
	import SearchIcon from '@lucide/svelte/icons/search';
	import { cn } from "#lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		value = $bindable(""),
		...restProps
	}: CommandPrimitive.InputProps = $props();
</script>

<div data-slot="command-input-wrapper" class="border-input flex items-center border-b-2 px-3">
	<SearchIcon class="me-2 size-4 shrink-0 opacity-50" aria-hidden="true" />
	<CommandPrimitive.Input
		data-slot="command-input"
		class={cn(
			"placeholder:text-muted-foreground flex h-10 w-full bg-transparent text-sm outline-hidden disabled:cursor-not-allowed disabled:opacity-50",
			className
		)}
		bind:ref
		bind:value
		{...restProps}
	/>
</div>
