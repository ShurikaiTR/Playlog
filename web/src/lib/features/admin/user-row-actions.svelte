<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';

	let { disabled, onChange }: { disabled: boolean; onChange: () => void } = $props();
</script>

<div class="flex justify-end">
	<Button
		variant="outline"
		size="sm"
		class="pointer-coarse:h-11"
		{disabled}
		title={disabled ? 'You can’t change your own role' : undefined}
		onclick={onChange}>Change role</Button
	>
</div>
