<script lang="ts">
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import { cn } from '#lib/utils.js';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import SettingsItems from './settings-items.svelte';

	// Header settings for signed-out visitors. Signed-in users get the same rows in the sidebar account menu.
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger
		class={cn(buttonVariants({ variant: 'ghost', size: 'icon-lg' }), 'cursor-pointer rounded-full')}
		aria-label="Settings"
	>
		<SettingsIcon class="size-5" />
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end" sideOffset={10} class="w-56 rounded-2xl p-2">
		<SettingsItems />
	</DropdownMenu.Content>
</DropdownMenu.Root>
