import { createCn } from "cn/config";
import { createTV } from "tailwind-variants";

// The Apple type scale from layout.css. Registered so cn treats them as font sizes
// (replacing text-sm) rather than as text colors (which would drop text-muted-foreground).
const typeScale = {
	"font-size": [
		{ text: ["display", "lead", "large-title", "title-1", "title-2", "title-3", "headline", "body", "callout", "subheadline", "footnote"] }
	]
};

export const cn = createCn({ extend: { classGroups: typeScale } });

// tv merges conflicting classes with its own merger, so it needs the same scale (otherwise
// text-subheadline would delete a variant's text-primary-foreground).
export const tv = createTV({ twMergeConfig: { extend: { classGroups: typeScale } } });

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
