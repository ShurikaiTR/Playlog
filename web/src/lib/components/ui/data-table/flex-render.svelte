<script
	lang="ts"
	generics="TData, TValue, TContext extends HeaderContext<TData, TValue> | CellContext<TData, TValue>"
>
	import type { CellContext, ColumnDefTemplate, HeaderContext } from '@tanstack/table-core';
	import { RenderComponentConfig, RenderSnippetConfig } from './render-helpers.js';

	type Props = {
		/** The `header`, `cell` or `footer` template of a column definition. */
		content?: TContext extends HeaderContext<TData, TValue>
			? ColumnDefTemplate<HeaderContext<TData, TValue>>
			: TContext extends CellContext<TData, TValue>
				? ColumnDefTemplate<CellContext<TData, TValue>>
				: never;
		context: TContext;
	};

	let { content, context }: Props = $props();
</script>

{#if typeof content === 'string'}
	{content}
{:else if content instanceof Function}
	{@const result = content(context as never)}
	{#if result instanceof RenderComponentConfig}
		{@const Component = result.component}
		<Component {...result.props} />
	{:else if result instanceof RenderSnippetConfig}
		{@render result.snippet(result.params)}
	{:else}
		{result}
	{/if}
{/if}
