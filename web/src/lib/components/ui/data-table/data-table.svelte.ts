import {
	createTable,
	type RowData,
	type TableOptions,
	type TableOptionsResolved,
	type TableState
} from '@tanstack/table-core';

/**
 * Svelte 5 adapter for TanStack Table v8 (the helper from the shadcn-svelte data table guide).
 * Options are read through their getters on every update, so reactive `data` and `columns`
 * passed from a component keep the table current.
 */
export function createSvelteTable<TData extends RowData>(options: TableOptions<TData>) {
	const resolvedOptions: TableOptionsResolved<TData> = mergeObjects(
		{
			state: {},
			onStateChange() {},
			renderFallbackValue: null,
			mergeOptions: (defaultOptions: TableOptions<TData>, options: Partial<TableOptions<TData>>) =>
				mergeObjects(defaultOptions, options)
		},
		options
	);

	const table = createTable(resolvedOptions);
	let state = $state<Partial<TableState>>(table.initialState);

	function updateOptions() {
		table.setOptions((previous) =>
			mergeObjects(previous, options, {
				state: mergeObjects(state, options.state || {}),
				onStateChange: (updater: unknown) => {
					if (updater instanceof Function) state = updater(state);
					else state = mergeObjects(state, updater as Partial<TableState>);
					options.onStateChange?.(updater as never);
				}
			})
		);
	}

	updateOptions();

	$effect.pre(() => {
		updateOptions();
	});

	return table;
}

type MaybeThunk<T extends object> = T | (() => T | null | undefined);
type Intersection<T extends readonly unknown[]> = (T extends [infer H, ...infer R]
	? H & Intersection<R>
	: unknown) &
	object;

/**
 * Merges objects without reading their getters, so reactive options stay lazy.
 * Later sources win; a source may be a thunk that returns the object.
 */
export function mergeObjects<Sources extends readonly MaybeThunk<object>[]>(
	...sources: Sources
): Intersection<{ [K in keyof Sources]: Sources[K] }> {
	const resolve = <T extends object>(src: MaybeThunk<T>): T | undefined =>
		typeof src === 'function' ? (src() ?? undefined) : src;

	const findSourceWithKey = (key: PropertyKey) => {
		for (let i = sources.length - 1; i >= 0; i--) {
			const obj = resolve(sources[i]);
			if (obj && key in obj) return obj;
		}
		return undefined;
	};

	return new Proxy(Object.create(null), {
		get(_, key) {
			const src = findSourceWithKey(key);
			return src?.[key as never];
		},
		has(_, key) {
			return !!findSourceWithKey(key);
		},
		ownKeys(): (string | symbol)[] {
			// Not reactive state: it only collects keys for one call.
			// eslint-disable-next-line svelte/prefer-svelte-reactivity
			const all = new Set<string | symbol>();
			for (const src of sources) {
				const obj = resolve(src);
				if (obj) for (const key of Reflect.ownKeys(obj)) all.add(key);
			}
			return [...all];
		},
		getOwnPropertyDescriptor(_, key) {
			const src = findSourceWithKey(key);
			if (!src) return undefined;
			return { configurable: true, enumerable: true, value: (src as never)[key], writable: true };
		}
	}) as Intersection<{ [K in keyof Sources]: Sources[K] }>;
}
