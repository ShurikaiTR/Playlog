<script lang="ts">
	import { onMount } from "svelte";
	import { setMode, userPrefersMode } from "mode-watcher";
	import { toast } from "svelte-sonner";
	import { Button, buttonVariants } from "#lib/components/ui/button/index.js";
	import * as Dialog from "#lib/components/ui/dialog/index.js";
	import * as Command from "#lib/components/ui/command/index.js";
	import * as DropdownMenu from "#lib/components/ui/dropdown-menu/index.js";
	import * as Tooltip from "#lib/components/ui/tooltip/index.js";
	import * as ToggleGroup from "#lib/components/ui/toggle-group/index.js";
	import { Label } from "#lib/components/ui/label/index.js";
	import { Textarea } from "#lib/components/ui/textarea/index.js";
	import { cn } from "#lib/utils.js";
	import SettingsIcon from "@lucide/svelte/icons/settings";
	import SunIcon from "@lucide/svelte/icons/sun";
	import MoonIcon from "@lucide/svelte/icons/moon";
	import MonitorIcon from "@lucide/svelte/icons/monitor";
	import LanguagesIcon from "@lucide/svelte/icons/languages";
	import FlagIcon from "@lucide/svelte/icons/flag";

	// A soft grey highlight instead of the menu's bold accent, so text and icons stay readable.
	const soft = "focus:bg-muted focus:text-foreground focus:**:text-foreground not-data-[variant=destructive]:focus:**:text-foreground";

	const themes = [
		{ value: "light", label: "Açık tema", icon: SunIcon },
		{ value: "dark", label: "Koyu tema", icon: MoonIcon },
		{ value: "system", label: "Sistem teması", icon: MonitorIcon },
	] as const;

	// TODO: placeholder, the app is Turkish only for now. The choice is just remembered until translations exist.
	const languages = [
		{ value: "tr", label: "Türkçe", turkish: "Türkçe" },
		{ value: "en", label: "English", turkish: "İngilizce" },
	];
	let language = $state("tr");
	let languageOpen = $state(false);
	onMount(() => {
		try {
			language = localStorage.getItem("playlog-language") ?? "tr";
		} catch {}
	});

	function chooseLanguage(value: string) {
		language = value;
		languageOpen = false;
		try {
			localStorage.setItem("playlog-language", value);
		} catch {}
		if (value !== "tr") toast("Dil desteği yakında");
	}

	const feedbackKinds = ["Hata", "Öneri", "Diğer"];
	let feedbackOpen = $state(false);
	let feedbackKind = $state("Öneri");
	let feedbackText = $state("");

	function openFeedback() {
		feedbackKind = "Öneri";
		feedbackText = "";
		feedbackOpen = true;
	}

	function sendFeedback(event: SubmitEvent) {
		event.preventDefault();
		if (!feedbackText.trim()) return;
		// TODO: placeholder, nothing is sent yet. Post { kind, text } to the server once there is a backend.
		feedbackOpen = false;
		toast("Geri bildirimin için teşekkürler");
	}
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger class={cn(buttonVariants({ variant: "ghost", size: "icon-lg" }), "cursor-pointer rounded-full")} aria-label="Ayarlar">
		<SettingsIcon class="size-5" strokeWidth={1.8} />
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end" sideOffset={10} class="w-56 rounded-2xl p-2">
		<div class="grid grid-cols-3 gap-1" role="group" aria-label="Tema seç">
			{#each themes as theme (theme.value)}
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<DropdownMenu.Item
								{...props}
								aria-label={theme.label}
								class={cn(
									"h-10 cursor-pointer justify-center rounded-xl p-0",
									soft,
									userPrefersMode.current === theme.value && "bg-muted text-foreground",
								)}
								onclick={() => setMode(theme.value)}
							>
								<theme.icon class="size-[18px]" strokeWidth={1.7} />
							</DropdownMenu.Item>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom" class="z-60">{theme.label}</Tooltip.Content>
				</Tooltip.Root>
			{/each}
		</div>
		<DropdownMenu.Group class="mt-1">
			<DropdownMenu.Item class={cn("cursor-pointer rounded-xl py-2", soft)} onSelect={() => (languageOpen = true)}>
				<LanguagesIcon />
				Dil
			</DropdownMenu.Item>
			<DropdownMenu.Item class={cn("cursor-pointer rounded-xl py-2", soft)} onSelect={openFeedback}>
				<FlagIcon />
				Geri bildirim
			</DropdownMenu.Item>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>

<Command.Dialog
	bind:open={languageOpen}
	title="Dil seçici"
	description="Sitenin dilini seç"
	class="top-1/2 flex max-h-[min(85dvh,640px)] w-[400px] max-w-[calc(100vw-2rem)] -translate-y-1/2 flex-col gap-4 rounded-2xl! p-0 ring-0 duration-200 sm:max-w-[400px] dark:ring-1"
>
	<Command.Input placeholder="Dil ara" />
	<Command.List class="max-h-none p-1">
		<Command.Empty>Dil bulunamadı</Command.Empty>
		{#each languages as option (option.value)}
			<!-- The highlight is an inset pill behind the row, not the row itself. -->
			<Command.Item
				value="{option.label} {option.turkish}"
				data-checked={option.value === language}
				class="not-data-[checked=true]:[&>svg]:hidden data-selected:bg-transparent after:bg-transparent data-selected:after:bg-muted relative isolate cursor-pointer gap-2 rounded-none px-3.5 py-3 after:absolute after:inset-x-1 after:inset-y-0 after:-z-10 after:rounded-lg after:transition-colors after:duration-75 in-data-[slot=dialog-content]:rounded-none"
				onSelect={() => chooseLanguage(option.value)}
			>
				{option.label}
				{#if option.value === language}
					<span class="sr-only">(seçili)</span>
				{:else}
					<span class="text-muted-foreground ms-auto">{option.turkish}</span>
				{/if}
			</Command.Item>
		{/each}
	</Command.List>
</Command.Dialog>

<Dialog.Root bind:open={feedbackOpen}>
	<Dialog.Content class="rounded-2xl sm:max-w-md [&_[data-slot=dialog-close]]:cursor-pointer">
		<Dialog.Header>
			<Dialog.Title class="text-primary text-lg font-medium tracking-tight">Geri bildirim gönder</Dialog.Title>
			<Dialog.Description>Bir hata mı buldun ya da bir fikrin mi var? Bize yaz.</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4" onsubmit={sendFeedback}>
			<div class="space-y-2">
				<Label id="feedback-kind-label">Tür</Label>
				<ToggleGroup.Root
					type="single"
					variant="outline"
					bind:value={() => feedbackKind, (value) => value && (feedbackKind = value)}
					aria-labelledby="feedback-kind-label"
					class="w-full"
				>
					{#each feedbackKinds as kind (kind)}
						<ToggleGroup.Item value={kind} class="flex-1 cursor-pointer">{kind}</ToggleGroup.Item>
					{/each}
				</ToggleGroup.Root>
			</div>
			<div class="space-y-2">
				<Label for="feedback-text">Mesajın</Label>
				<Textarea id="feedback-text" bind:value={feedbackText} placeholder="Neyi fark ettin, ne görmek istersin?" class="min-h-28 resize-none rounded-xl" />
			</div>
			<Dialog.Footer>
				<Button type="button" variant="outline" class="cursor-pointer rounded-full" onclick={() => (feedbackOpen = false)}>Vazgeç</Button>
				<Button type="submit" class="cursor-pointer rounded-full" disabled={!feedbackText.trim()}>Gönder</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
