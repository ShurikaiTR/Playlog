<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Command from '#lib/components/ui/command/index.js';
	import { languageChoice, languages, settingsDialogs } from '#lib/settings-dialogs.svelte.js';
	import FeedbackForm from './feedback-form.svelte';

	onMount(() => {
		try {
			languageChoice.value = localStorage.getItem('playlog-language') ?? 'tr';
		} catch {}
	});

	function chooseLanguage(value: string) {
		languageChoice.value = value;
		settingsDialogs.language = false;
		try {
			localStorage.setItem('playlog-language', value);
		} catch {}
		if (value !== 'tr') toast('Dil desteği yakında');
	}
</script>

<Command.Dialog
	bind:open={settingsDialogs.language}
	title="Dil seçici"
	description="Sitenin dilini seç"
	class="z-60"
	overlayClass="z-60"
>
	<Command.Input placeholder="Dil ara" />
	<Command.List class="max-h-none p-1">
		<Command.Empty>Dil bulunamadı</Command.Empty>
		{#each languages as option (option.value)}
			<!-- The highlight is an inset pill behind the row, not the row itself. -->
			<Command.Item
				value="{option.label} {option.turkish}"
				data-checked={option.value === languageChoice.value}
				class="relative isolate cursor-pointer gap-2 rounded-none px-3.5 py-3 after:absolute after:inset-x-1 after:inset-y-0 after:-z-10 after:rounded-lg after:bg-transparent after:transition-colors after:duration-75 in-data-[slot=dialog-content]:rounded-none data-selected:bg-transparent data-selected:after:bg-muted not-data-[checked=true]:[&>svg]:hidden"
				onSelect={() => chooseLanguage(option.value)}
			>
				{option.label}
				{#if option.value === languageChoice.value}
					<span class="sr-only">(seçili)</span>
				{:else}
					<span class="ms-auto text-muted-foreground">{option.turkish}</span>
				{/if}
			</Command.Item>
		{/each}
	</Command.List>
</Command.Dialog>

<Dialog.Root bind:open={settingsDialogs.feedback}>
	<Dialog.Content class="w-[520px]">
		<Dialog.Header>
			<Dialog.Title>Geri bildirim</Dialog.Title>
			<Dialog.Description class="sr-only"
				>Bir sorunu bildir ya da fikrini paylaş.</Dialog.Description
			>
		</Dialog.Header>
		<FeedbackForm onsent={() => (settingsDialogs.feedback = false)} />
	</Dialog.Content>
</Dialog.Root>
