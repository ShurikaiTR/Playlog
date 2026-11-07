<script lang="ts">
	import { setMode, userPrefersMode } from 'mode-watcher';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { openFeedbackDialog, openLanguageDialog } from '#lib/settings-dialogs.svelte.js';
	import { cn } from '#lib/utils.js';
	import SunIcon from '@lucide/svelte/icons/sun';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import FlagIcon from '@lucide/svelte/icons/flag';

	// Theme, language and feedback rows, shared by the header settings menu and the sidebar account menu.
	// Signed-in users change the theme and language in the settings dialog instead, so the account menu hides those rows.
	let {
		theme: showTheme = true,
		language: showLanguage = true
	}: { theme?: boolean; language?: boolean } = $props();

	// A soft grey highlight instead of the menu's bold accent, so text and icons stay readable.
	const soft =
		'focus:bg-muted focus:text-foreground focus:**:text-foreground not-data-[variant=destructive]:focus:**:text-foreground';

	const themes = [
		{ value: 'light', label: 'Light theme', icon: SunIcon },
		{ value: 'dark', label: 'Dark theme', icon: MoonIcon },
		{ value: 'system', label: 'System theme', icon: MonitorIcon }
	] as const;
</script>

{#if showTheme}
	<div class="grid grid-cols-3 gap-1" role="group" aria-label="Choose theme">
		{#each themes as theme (theme.value)}
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<DropdownMenu.Item
							{...props}
							aria-label={theme.label}
							class={cn(
								'h-11 cursor-pointer justify-center rounded-xl p-0',
								soft,
								userPrefersMode.current === theme.value && 'bg-muted text-foreground'
							)}
							onclick={() => setMode(theme.value)}
						>
							<theme.icon class="size-4" />
						</DropdownMenu.Item>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="bottom" class="z-60">{theme.label}</Tooltip.Content>
			</Tooltip.Root>
		{/each}
	</div>
{/if}
<DropdownMenu.Group class={cn(showTheme && 'mt-1')}>
	{#if showLanguage}
		<DropdownMenu.Item
			class={cn('min-h-11 cursor-pointer rounded-xl py-2 text-callout', soft)}
			onSelect={openLanguageDialog}
		>
			<LanguagesIcon />
			Dil
		</DropdownMenu.Item>
	{/if}
	<DropdownMenu.Item
		class={cn('min-h-11 cursor-pointer rounded-xl py-2 text-callout', soft)}
		onSelect={openFeedbackDialog}
	>
		<FlagIcon />
		Geri bildirim
	</DropdownMenu.Item>
</DropdownMenu.Group>
