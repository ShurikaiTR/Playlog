import type { Component, ComponentProps, Snippet } from 'svelte';

/** Wraps a Svelte component (and its props) so a TanStack column can render it in a cell. */
export class RenderComponentConfig<TComponent extends Component> {
	component: TComponent;
	props: ComponentProps<TComponent> | Record<string, never>;
	constructor(
		component: TComponent,
		props: ComponentProps<TComponent> | Record<string, never> = {}
	) {
		this.component = component;
		this.props = props;
	}
}

/** Wraps a snippet (and its parameters) so a TanStack column can render it in a cell. */
export class RenderSnippetConfig<TProps> {
	snippet: Snippet<[TProps]>;
	params: TProps;
	constructor(snippet: Snippet<[TProps]>, params: TProps) {
		this.snippet = snippet;
		this.params = params;
	}
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function renderComponent<T extends Component<any>, Props extends ComponentProps<T>>(
	component: T,
	props: Props = {} as Props
) {
	return new RenderComponentConfig(component, props);
}

export function renderSnippet<TProps>(snippet: Snippet<[TProps]>, params: TProps = {} as TProps) {
	return new RenderSnippetConfig(snippet, params);
}
