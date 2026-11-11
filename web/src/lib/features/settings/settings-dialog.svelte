<script lang="ts">
	import { page } from '$app/state';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import PaletteIcon from '@lucide/svelte/icons/palette';
	import { resetMode, setMode, userPrefersMode } from 'mode-watcher';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { ScrollArea } from '#lib/components/ui/scroll-area/index.js';
	import AccountAvatar from '#lib/features/account/account-avatar.svelte';
	import AccountPane from './account-pane.svelte';
	import ScreenHeader from './screen-header.svelte';
	import SettingsGroup from './settings-group.svelte';
	import SettingsRow from './settings-row.svelte';
	import { IsMobile } from '#lib/hooks/is-mobile.svelte.js';
	import {
		settingsDialog,
		showSection,
		type SettingsSection
	} from './settings-dialog-state.svelte.js';
	import {
		languageChoice,
		languages,
		openLanguageDialog
	} from './settings-dialogs-state.svelte.js';
	import { cn } from '#lib/utils.js';

	// System Settings layout: the section list beside the open section on wide screens; on phones
	// the window fills the screen and the list is a screen of its own that sections push onto.
	const isMobile = new IsMobile();

	const user = $derived(page.data.user);
	const name = $derived(user ? user.displayName || user.username : '');

	const sections: {
		id: Exclude<SettingsSection, 'account'>;
		label: string;
		icon: typeof PaletteIcon;
	}[] = [{ id: 'appearance', label: 'Appearance', icon: PaletteIcon }];

	const themes = [
		{ id: 'light', label: 'Light' },
		{ id: 'dark', label: 'Dark' },
		{ id: 'system', label: 'System' }
	] as const;

	function changeTheme(value: (typeof themes)[number]['id']) {
		if (value === 'system') resetMode();
		else setMode(value);
	}

	const languageLabel = $derived(
		languages.find((option) => option.value === languageChoice.value)?.label ?? 'English'
	);

	const listVisible = $derived(!isMobile.current || settingsDialog.showList);
	const contentVisible = $derived(!isMobile.current || !settingsDialog.showList);
	// On phones each section leads back to the list.
	const backToList = $derived(isMobile.current);
</script>

<!-- A tiny app window drawn with the theme tokens of the surrounding .light/.dark scope. -->
{#snippet windowPreview()}
	<span class="flex h-full w-full gap-1.5 bg-background p-1.5">
		<span class="flex w-5 flex-col gap-1 rounded-sm bg-muted p-1">
			<span class="h-1 rounded-full bg-foreground/40"></span>
			<span class="h-1 rounded-full bg-foreground/20"></span>
			<span class="h-1 rounded-full bg-foreground/20"></span>
		</span>
		<span class="flex flex-1 flex-col gap-1 rounded-sm border border-border bg-card p-1.5">
			<span class="h-1 w-3/4 rounded-full bg-foreground/40"></span>
			<span class="h-1 w-1/2 rounded-full bg-foreground/20"></span>
			<span class="mt-auto ml-auto size-1.5 rounded-full bg-foreground"></span>
		</span>
	</span>
{/snippet}

<Dialog.Root bind:open={settingsDialog.open}>
	<Dialog.Content
		class="h-[min(40rem,calc(100dvh-2rem))] grid-rows-[minmax(0,1fr)] gap-0 overflow-hidden p-0 max-md:top-0 max-md:left-0 max-md:h-dvh max-md:max-h-dvh max-md:w-full max-md:max-w-full max-md:translate-x-0 max-md:translate-y-0 max-md:rounded-none sm:max-w-3xl md:grid-cols-[15rem_1fr]"
	>
		{#if listVisible}
			<ScrollArea class="min-h-0 md:border-r md:border-border">
				<nav class="flex flex-col gap-1 p-3 max-md:pt-14" aria-label="Settings sections">
					{#if isMobile.current}
						<Dialog.Title class="px-2.5 pb-3 text-large-title">Settings</Dialog.Title>
						<Dialog.Description class="sr-only">Your account and app settings.</Dialog.Description>
					{/if}
					{#if user}
						<!-- Like the Apple Account entry at the top of System Settings. -->
						<Button
							variant="ghost"
							class={cn(
								'h-auto min-h-14 cursor-pointer justify-start gap-3 rounded-xl px-2.5 py-2 text-left',
								!isMobile.current && settingsDialog.section === 'account' && 'bg-muted'
							)}
							aria-current={settingsDialog.section === 'account' ? 'page' : undefined}
							onclick={() => showSection('account')}
						>
							<AccountAvatar {name} class="size-10" />
							<span class="min-w-0 flex-1 leading-tight">
								<span class="block truncate text-callout font-medium">{name}</span>
								<span class="block truncate text-footnote font-normal text-muted-foreground"
									>Playthru Account</span
								>
							</span>
							{#if isMobile.current}<ChevronRightIcon class="size-4 text-muted-foreground" />{/if}
						</Button>
					{/if}
					<p class="px-2.5 pt-3 pb-1 text-footnote text-muted-foreground">General</p>
					{#each sections as item (item.id)}
						<Button
							variant="ghost"
							class={cn(
								'h-11 cursor-pointer justify-start gap-2.5 rounded-xl text-callout font-normal',
								!isMobile.current && settingsDialog.section === item.id && 'bg-muted'
							)}
							aria-current={settingsDialog.section === item.id ? 'page' : undefined}
							onclick={() => showSection(item.id)}
						>
							<item.icon class="size-4" />{item.label}
							{#if isMobile.current}<ChevronRightIcon
									class="ms-auto size-4 text-muted-foreground"
								/>{/if}
						</Button>
					{/each}
				</nav>
			</ScrollArea>
		{/if}

		{#if contentVisible}
			<ScrollArea class="min-h-0">
				<div class="mx-auto max-w-xl p-6 text-callout max-md:px-4 max-md:pt-3">
					{#if backToList}
						<Button
							variant="ghost"
							class="-ms-3 mb-2 h-11 cursor-pointer gap-0.5 rounded-xl ps-1.5 pe-3 text-callout text-primary"
							onclick={() => (settingsDialog.showList = true)}
						>
							<ChevronLeftIcon class="size-5" aria-hidden="true" />Ayarlar
						</Button>
					{/if}

					{#if settingsDialog.section === 'account' && user}
						<AccountPane />
					{:else}
						<ScreenHeader title="Appearance" description="The site theme and language." />

						<div class="space-y-6">
							<SettingsGroup title="Theme">
								<div
									class="flex justify-around gap-3 px-4 py-4"
									role="group"
									aria-label="Choose theme"
								>
									{#each themes as theme (theme.id)}
										{@const selected = userPrefersMode.current === theme.id}
										<button
											type="button"
											aria-pressed={selected}
											class="group flex min-h-11 cursor-pointer flex-col items-center gap-2 rounded-xl outline-none"
											onclick={() => changeTheme(theme.id)}
										>
											<span
												class={cn(
													'relative flex h-16 w-24 overflow-hidden rounded-xl border border-border transition-shadow group-focus-visible:ring-2 group-focus-visible:ring-ring/50',
													selected
														? 'ring-2 ring-foreground'
														: 'group-hover:ring-2 group-hover:ring-border'
												)}
											>
												{#if theme.id === 'system'}
													<span class="dark w-1/2 overflow-hidden"
														><span class="flex h-full w-24">{@render windowPreview()}</span></span
													>
													<span class="light w-1/2 overflow-hidden"
														><span class="-ml-12 flex h-full w-24">{@render windowPreview()}</span
														></span
													>
												{:else}
													<span class="{theme.id} flex w-full">{@render windowPreview()}</span>
												{/if}
											</span>
											<span
												class={cn(
													'text-footnote',
													selected ? 'text-foreground' : 'text-muted-foreground'
												)}>{theme.label}</span
											>
										</button>
									{/each}
								</div>
							</SettingsGroup>

							<SettingsGroup>
								<SettingsRow label="Language" value={languageLabel} onclick={openLanguageDialog}>
									<ChevronDownIcon class="size-4 text-muted-foreground/70" aria-hidden="true" />
								</SettingsRow>
							</SettingsGroup>
						</div>
					{/if}
				</div>
			</ScrollArea>
		{/if}
	</Dialog.Content>
</Dialog.Root>
