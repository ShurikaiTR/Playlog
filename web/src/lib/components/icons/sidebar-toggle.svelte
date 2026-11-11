<script lang="ts">
	import type { SVGAttributes } from "svelte/elements";

	let props: SVGAttributes<SVGSVGElement> = $props();
</script>

<svg width="1.125rem" height="1.125rem" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
	<path
		d="M9 17H8.09961C7.53982 17 7.25976 16.9996 7.0459 16.8906C6.85793 16.7948 6.70521 16.6421 6.60938 16.4541C6.50041 16.2402 6.5 15.9602 6.5 15.4004V8.59961C6.5 8.03982 6.50041 7.75976 6.60938 7.5459C6.70521 7.35793 6.85793 7.20521 7.0459 7.10938C7.25976 7.00041 7.53982 7 8.09961 7H9V17Z"
		fill="currentColor"
	/>
	<path
		fill-rule="evenodd"
		clip-rule="evenodd"
		d="M16.5 3C19.2614 3 21.5 5.23858 21.5 8V16C21.5 18.7614 19.2614 21 16.5 21H7.5C4.73858 21 2.5 18.7614 2.5 16V8C2.5 5.23858 4.73858 3 7.5 3H16.5ZM7.5 5C5.84315 5 4.5 6.34315 4.5 8V16C4.5 17.6569 5.84315 19 7.5 19H16.5C18.1569 19 19.5 17.6569 19.5 16V8C19.5 6.34315 18.1569 5 16.5 5H7.5Z"
		fill="currentColor"
	/>
</svg>
