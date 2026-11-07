<script lang="ts">
	import type { Snippet } from 'svelte';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import * as Item from '#lib/components/ui/item/index.js';
	import { cn } from '#lib/utils.js';

	// One row of a settings group: label on the left, value or control on the right. With
	// `onclick` the whole row is a button; with `href` it is a link to another screen and gets a chevron.
	let {
		label,
		description,
		value,
		chevron = false,
		destructive = false,
		disabled = false,
		onclick,
		href,
		media,
		children
	}: {
		label: string;
		description?: string;
		value?: string;
		chevron?: boolean;
		destructive?: boolean;
		disabled?: boolean;
		onclick?: () => void;
		href?: string;
		media?: Snippet;
		children?: Snippet;
	} = $props();

	const rowClass = $derived(
		cn(
			'min-h-11 gap-3 px-4 py-2.5 text-callout',
			(onclick || href) &&
				'cursor-pointer text-left hover:bg-muted focus-visible:bg-muted disabled:cursor-default disabled:opacity-50 disabled:hover:bg-transparent'
		)
	);
</script>

{#snippet body()}
	{#if media}
		<Item.Media class="shrink-0">{@render media()}</Item.Media>
	{/if}
	<Item.Content class="min-w-0 gap-0.5">
		<Item.Title class={cn('text-callout font-normal', destructive && 'text-destructive')}
			>{label}</Item.Title
		>
		{#if description}
			<Item.Description class="line-clamp-none text-footnote">{description}</Item.Description>
		{/if}
	</Item.Content>
	{#if value || children || chevron || href}
		<Item.Actions class="ms-auto shrink-0 gap-2 text-muted-foreground">
			{#if value}<span class="max-w-48 truncate text-callout">{value}</span>{/if}
			{@render children?.()}
			{#if chevron || href}<ChevronRightIcon
					class="size-4 text-muted-foreground/70"
					aria-hidden="true"
				/>{/if}
		</Item.Actions>
	{/if}
{/snippet}

{#if href}
	<Item.Root class={rowClass}>
		{#snippet child({ props })}
			<!-- Callers pass a path made with resolve(). -->
			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
			<a {...props} {href}>{@render body()}</a>
		{/snippet}
	</Item.Root>
{:else if onclick}
	<Item.Root class={rowClass}>
		{#snippet child({ props })}
			<button type="button" {...props} {disabled} {onclick}>{@render body()}</button>
		{/snippet}
	</Item.Root>
{:else}
	<Item.Root class={rowClass}>{@render body()}</Item.Root>
{/if}
