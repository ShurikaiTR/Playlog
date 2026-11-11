<script lang="ts">
	import type { SVGAttributes } from "svelte/elements";

	let props: SVGAttributes<SVGSVGElement> = $props();
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	width="24"
	height="24"
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="2"
	stroke-linecap="round"
	stroke-linejoin="round"
	{...props}
>
	<rect width="20" height="14" x="2" y="3" rx="2" />
	<path d="M8 21h8M12 17v4" />
	<text
		x="12"
		y="10.2"
		fill="currentColor"
		stroke="none"
		font-size="7.5"
		font-weight="800"
		text-anchor="middle"
		dominant-baseline="central">PC</text
	>
</svg>
