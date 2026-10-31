<script lang="ts">
	import { onMount } from "svelte";
	import { setMode, userPrefersMode } from "mode-watcher";
	import { toast } from "svelte-sonner";
	import { Button, buttonVariants } from "#lib/components/ui/button/index.js";
	import * as Dialog from "#lib/components/ui/dialog/index.js";
	import * as Command from "#lib/components/ui/command/index.js";
	import * as DropdownMenu from "#lib/components/ui/dropdown-menu/index.js";
	import * as Tooltip from "#lib/components/ui/tooltip/index.js";
	import * as Select from "#lib/components/ui/select/index.js";
	import * as Empty from "#lib/components/ui/empty/index.js";
	import { Badge } from "#lib/components/ui/badge/index.js";
	import { Label } from "#lib/components/ui/label/index.js";
	import { Textarea } from "#lib/components/ui/textarea/index.js";
	import { cn } from "#lib/utils.js";
	import SettingsIcon from "@lucide/svelte/icons/settings";
	import SunIcon from "@lucide/svelte/icons/sun";
	import MoonIcon from "@lucide/svelte/icons/moon";
	import MonitorIcon from "@lucide/svelte/icons/monitor";
	import LanguagesIcon from "@lucide/svelte/icons/languages";
	import FlagIcon from "@lucide/svelte/icons/flag";
	import UploadIcon from "@lucide/svelte/icons/upload";
	import XIcon from "@lucide/svelte/icons/x";

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

	const feedbackReasons = ["Hata", "Öneri", "Diğer"];
	const maxAttachments = 5;
	let feedbackOpen = $state(false);
	let feedbackReason = $state("");
	let feedbackText = $state("");
	let attachments = $state<File[]>([]);
	let fileInput = $state<HTMLInputElement | null>(null);
	let dragging = $state(false);
	const canSend = $derived(!!feedbackReason && !!feedbackText.trim());

	function openFeedback() {
		feedbackReason = "";
		feedbackText = "";
		attachments = [];
		feedbackOpen = true;
	}

	function addFiles(files: FileList | null | undefined) {
		if (!files) return;
		const all = [...attachments, ...files];
		if (all.length > maxAttachments) toast(`En fazla ${maxAttachments} dosya ekleyebilirsin`);
		attachments = all.slice(0, maxAttachments);
	}

	function sendFeedback(event: SubmitEvent) {
		event.preventDefault();
		if (!canSend) return;
		// TODO: placeholder, nothing is sent yet. Post { reason, text, attachments } to the server once there is a backend.
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
	<Dialog.Content class="w-[520px]">
		<Dialog.Header>
			<Dialog.Title>Geri bildirim</Dialog.Title>
			<Dialog.Description class="sr-only">Bir sorunu bildir ya da fikrini paylaş.</Dialog.Description>
		</Dialog.Header>
		<form class="flex flex-col gap-4" onsubmit={sendFeedback}>
			<div>
				<Label for="feedback-reason" class="mb-2 font-semibold">Tür</Label>
				<Select.Root type="single" bind:value={feedbackReason}>
					<Select.Trigger id="feedback-reason" class="h-10! w-full cursor-pointer rounded-xl px-4">
						<span data-slot="select-value" class={cn(!feedbackReason && "text-muted-foreground")}>{feedbackReason || "Tür seç"}</span>
					</Select.Trigger>
					<Select.Content class="rounded-xl">
						{#each feedbackReasons as reason (reason)}
							<Select.Item value={reason} class={cn("data-highlighted:bg-muted data-highlighted:text-foreground cursor-pointer rounded-lg", soft)}>{reason}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<Textarea
				bind:value={feedbackText}
				rows={8}
				aria-label="Mesajın"
				placeholder="Bize neyin yanlış gittiğini ya da ne istediğini anlat"
				class="min-h-40"
			/>
			<div>
				<p class="mb-2 text-sm font-semibold">Ekler<span class="text-muted-foreground ms-1 font-normal tabular-nums">({attachments.length}/{maxAttachments})</span></p>
				<Empty.Root
					role="button"
					tabindex={0}
					aria-label="Dosya ekle"
					class={cn("hover:bg-muted/50 hover:border-foreground/25 cursor-pointer gap-2 rounded-2xl border border-dashed p-4 transition-colors md:p-4", dragging && "bg-muted/50 border-foreground/30")}
					onclick={() => fileInput?.click()}
					onkeydown={(event) => (event.key === "Enter" || event.key === " ") && (event.preventDefault(), fileInput?.click())}
					ondragover={(event) => (event.preventDefault(), (dragging = true))}
					ondragleave={() => (dragging = false)}
					ondrop={(event) => {
						event.preventDefault();
						dragging = false;
						addFiles(event.dataTransfer?.files);
					}}
				>
					<Empty.Header class="gap-2">
						<Empty.Media class="text-muted-foreground mb-0 rounded-full p-2"><UploadIcon class="size-5" /></Empty.Media>
						<Empty.Title class="text-sm font-medium">Sürükle bırak ya da seçmek için tıkla</Empty.Title>
					</Empty.Header>
				</Empty.Root>
				<input
					bind:this={fileInput}
					type="file"
					multiple
					accept="image/png,image/jpeg,image/gif,image/webp,application/pdf,text/plain,application/json"
					class="hidden"
					aria-hidden="true"
					tabindex={-1}
					onchange={(event) => {
						addFiles(event.currentTarget.files);
						event.currentTarget.value = "";
					}}
				/>
				{#if attachments.length}
					<ul class="mt-2 flex flex-wrap gap-1.5" aria-label="Eklenen dosyalar">
						{#each attachments as file, index (index)}
							<li>
								<Badge variant="secondary" class="max-w-56 pe-1">
									<span class="truncate">{file.name}</span>
									<Button
										type="button"
										variant="ghost"
										size="icon-xs"
										class="size-4 cursor-pointer rounded-full"
										aria-label="{file.name} dosyasını kaldır"
										onclick={() => (attachments = attachments.filter((_, i) => i !== index))}
									>
										<XIcon class="size-3" />
									</Button>
								</Badge>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
			<div class="flex justify-end">
				<Button type="submit" size="sm" class="h-8 cursor-pointer rounded-full px-2.5 font-semibold" disabled={!canSend}>Gönder</Button>
			</div>
		</form>
	</Dialog.Content>
</Dialog.Root>
