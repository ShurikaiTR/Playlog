<script lang="ts">
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import PaletteIcon from '@lucide/svelte/icons/palette';
	import { resetMode, setMode, userPrefersMode } from 'mode-watcher';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { ScrollArea } from '#lib/components/ui/scroll-area/index.js';
	import { settingsDialog, type SettingsSection } from '#lib/settings-dialog.svelte.js';
	import { languageChoice, languages, openLanguageDialog } from '#lib/settings-dialogs.svelte.js';
	import { cn } from '#lib/utils.js';

	// Hesap, Profil and Veri ve gizlilik join here once the API has endpoints for them.
	const sections: { id: SettingsSection; label: string; icon: typeof PaletteIcon }[] = [
		{ id: 'appearance', label: 'Görünüm', icon: PaletteIcon }
	];

	const themes = [
		{ id: 'light', label: 'Açık' },
		{ id: 'dark', label: 'Koyu' },
		{ id: 'system', label: 'Sistem' }
	] as const;

	function changeTheme(value: (typeof themes)[number]['id']) {
		if (value === 'system') resetMode();
		else setMode(value);
	}

	const languageLabel = $derived(
		languages.find((option) => option.value === languageChoice.value)?.label ?? 'Türkçe'
	);
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
		class="h-[min(36rem,calc(100dvh-2rem))] grid-rows-[auto_1fr] gap-0 overflow-hidden p-0 sm:max-w-3xl md:grid-cols-[13rem_1fr] md:grid-rows-1"
	>
		<ScrollArea orientation="horizontal" class="border-b border-border md:border-r md:border-b-0">
			<nav class="flex gap-1 p-3 md:flex-col" aria-label="Ayar bölümleri">
				<p class="hidden px-2.5 pt-1 pb-1.5 text-footnote text-muted-foreground md:block">Genel</p>
				{#each sections as item (item.id)}
					<Button
						variant="ghost"
						class={cn(
							'cursor-pointer justify-start gap-2.5 rounded-lg text-callout',
							settingsDialog.section === item.id && 'bg-muted'
						)}
						aria-current={settingsDialog.section === item.id ? 'page' : undefined}
						onclick={() => (settingsDialog.section = item.id)}
					>
						<item.icon class="size-4" />{item.label}
					</Button>
				{/each}
			</nav>
		</ScrollArea>

		<ScrollArea class="min-h-0">
			<div class="p-6 text-callout">
				{#if settingsDialog.section === 'appearance'}
					<Dialog.Title class="mb-6 text-headline">Görünüm</Dialog.Title>
					<Dialog.Description class="sr-only">Sitenin teması ve dili.</Dialog.Description>

					<div class="flex flex-wrap items-start justify-between gap-4">
						<span>Tema</span>
						<div class="flex gap-3" role="group" aria-label="Tema seç">
							{#each themes as theme (theme.id)}
								{@const selected = userPrefersMode.current === theme.id}
								<button
									type="button"
									aria-pressed={selected}
									class="group flex cursor-pointer flex-col items-center gap-2 rounded-xl outline-none"
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
					</div>

					<div class="mt-6 flex items-center justify-between gap-4 border-t border-border pt-6">
						<span>Dil</span>
						<Button
							variant="outline"
							class="cursor-pointer gap-2 rounded-xl text-callout"
							aria-haspopup="dialog"
							onclick={openLanguageDialog}
						>
							{languageLabel}
							<ChevronDownIcon class="size-4 text-muted-foreground" aria-hidden="true" />
						</Button>
					</div>
				{/if}
			</div>
		</ScrollArea>
	</Dialog.Content>
</Dialog.Root>
