<script lang="ts">
	import { Badge } from "#lib/components/ui/badge/index.js";
	import { Button, buttonVariants } from "#lib/components/ui/button/index.js";
	import * as Dialog from "#lib/components/ui/dialog/index.js";
	import * as DropdownMenu from "#lib/components/ui/dropdown-menu/index.js";
	import * as Empty from "#lib/components/ui/empty/index.js";
	import * as InputGroup from "#lib/components/ui/input-group/index.js";
	import { Input } from "#lib/components/ui/input/index.js";
	import { Label } from "#lib/components/ui/label/index.js";
	import * as Select from "#lib/components/ui/select/index.js";
	import { Textarea } from "#lib/components/ui/textarea/index.js";
	import { setGame, type UserList } from "#lib/user-lists.svelte.js";
	import { toast } from "svelte-sonner";
	import ArrowDownWideNarrowIcon from "@lucide/svelte/icons/arrow-down-wide-narrow";
	import ArrowUpRightIcon from "@lucide/svelte/icons/arrow-up-right";
	import ClockIcon from "@lucide/svelte/icons/clock-3";
	import GamepadIcon from "@lucide/svelte/icons/gamepad-2";
	import GlobeIcon from "@lucide/svelte/icons/globe";
	import HeartIcon from "@lucide/svelte/icons/heart";
	import LockIcon from "@lucide/svelte/icons/lock";
	import PencilIcon from "@lucide/svelte/icons/pencil";
	import SearchIcon from "@lucide/svelte/icons/search";
	import StarIcon from "@lucide/svelte/icons/star";
	import XIcon from "@lucide/svelte/icons/x";

	let { list }: { list: UserList } = $props();

	const copy = {
		favorites: {
			removeLabel: "Favorilerden çıkar",
			emptyTitle: "Favorilerin burada başlar.",
			emptyDescription: "Sevdiğin bir oyun bul ve koleksiyonunda ona yer aç.",
			meta: "Koleksiyonundaki favori oyunlar.",
		},
		play_later: {
			removeLabel: "Listeden çıkar",
			emptyTitle: "Planladığın bir oyun yok.",
			emptyDescription: "İlgini çeken bir oyunu planla, sırası geldiğinde burada seni beklesin.",
			meta: "Oynamayı planladığın oyunlar.",
		},
		custom: {
			removeLabel: "Listeden çıkar",
			emptyTitle: "Bu liste henüz boş.",
			emptyDescription: "Bir oyunun sayfasından \"Listeme ekle\" ile buraya oyun ekleyebilirsin.",
			meta: "Kendi derlemen.",
		},
	} as const;

	const text = $derived(copy[list.kind]);
	const sortOptions = [
		{ value: "recent", label: "Son eklenen" },
		{ value: "title", label: "Ada göre (A–Z)" },
		{ value: "rating", label: "En yüksek puan" },
	];

	let query = $state("");
	let selectedSort = $state("recent");
	let flippedSlug = $state<string | null>(null);
	let editOpen = $state(false);
	let editMode = $state<"details" | "image" | "backdrop">("details");
	let draftName = $state("");
	let draftDescription = $state("");
	let draftVisibility = $state<string>("private");
	let draftImage = $state("");
	let draftBackdrop = $state("");
	let nameError = $state("");
	let imageError = $state("");
	let pendingImages = $state(0);

	const selectedSortLabel = $derived(sortOptions.find((option) => option.value === selectedSort)?.label);
	const visibleItems = $derived.by(() => {
		const search = query.trim().toLocaleLowerCase("tr-TR");
		return list.items
			.filter((game) => game.title.toLocaleLowerCase("tr-TR").includes(search))
			.toSorted((a, b) => {
				if (selectedSort === "title") return a.title.localeCompare(b.title, "tr");
				if (selectedSort === "rating") return b.score - a.score || a.title.localeCompare(b.title, "tr");
				return b.added - a.added;
			});
	});

	function selectImage(event: Event, target: "image" | "backdrop") {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		imageError = "";
		if (!["image/jpeg", "image/png", "image/webp", "image/gif"].includes(file.type) || file.size > 5 * 1024 * 1024) {
			imageError = "5 MB altında bir JPG, PNG, WebP ya da GIF seç.";
			input.value = "";
			return;
		}
		pendingImages += 1;
		const reader = new FileReader();
		reader.onload = () => {
			if (typeof reader.result === "string") {
				if (target === "image") draftImage = reader.result;
				else draftBackdrop = reader.result;
			}
		};
		reader.onerror = () => (imageError = "Bu görsel açılamadı. Başka bir dosya dene.");
		reader.onloadend = () => (pendingImages -= 1);
		reader.readAsDataURL(file);
	}

	function openSettings(mode: "details" | "image" | "backdrop" = "details") {
		editMode = mode;
		draftName = list.name;
		draftDescription = list.description;
		draftVisibility = list.visibility;
		draftImage = list.image;
		draftBackdrop = list.backdrop;
		imageError = "";
		nameError = "";
		editOpen = true;
	}

	function saveSettings(event: SubmitEvent) {
		event.preventDefault();
		if (pendingImages) return;
		if (editMode === "details") {
			if (!draftName.trim()) {
				nameError = "Listen için bir ad gir.";
				return;
			}
			list.name = draftName.trim();
			list.description = draftDescription.trim();
			list.visibility = draftVisibility === "public" ? "public" : "private";
		} else if (editMode === "image") {
			list.image = draftImage;
		} else {
			list.backdrop = draftBackdrop;
		}
		editOpen = false;
		toast("Liste ayarları güncellendi");
	}

	function removeItem(game: UserList["items"][number]) {
		const { added, ...summary } = game;
		setGame(list, summary, false);
		flippedSlug = null;
		toast(`${game.title} listeden çıkarıldı`, { action: { label: "Geri al", onClick: () => setGame(list, summary, true) } });
	}
</script>

{#snippet listIcon(className: string)}
	{#if list.kind === "favorites"}<HeartIcon class={className} strokeWidth={1.5} />
	{:else if list.kind === "play_later"}<ClockIcon class={className} strokeWidth={1.5} />
	{:else}<GamepadIcon class={className} strokeWidth={1.5} />{/if}
{/snippet}

<svelte:head>
	<title>{list.name} | Playlog</title>
</svelte:head>

<div lang="tr" class="flex min-w-0 flex-col pb-16">
	<section class="list-hero relative isolate min-w-0" aria-labelledby="list-title">
		<!-- Full-bleed cover like the profile and game pages. -->
		<div class="pointer-events-none absolute inset-0 -z-10 overflow-hidden" aria-hidden="true" style="mask-image: linear-gradient(to bottom, black 55%, transparent 100%)">
			<img src={list.backdrop} alt="" width="1600" height="560" class="size-full object-cover object-right" />
			<div class="list-scrim absolute inset-0"></div>
		</div>

		<div class="flex flex-col gap-6 px-4 pt-32 pb-2 sm:flex-row sm:items-end sm:gap-8 md:px-8 md:pt-40">
			<div class="bg-muted ring-background relative flex size-32 shrink-0 items-center justify-center overflow-hidden rounded-xl shadow-sm ring-4 sm:size-40">
				{#if list.image}<img src={list.image} alt="" class="size-full object-cover" />{:else}
					{@render listIcon("size-8 text-muted-foreground")}
				{/if}
			</div>
			<div class="min-w-0 flex-1 pb-1">
				<Badge variant="secondary" class="mb-3 gap-1.5">
					{#if list.visibility === "private"}<LockIcon class="size-3" />Gizli liste{:else}<GlobeIcon class="size-3" />Herkese açık liste{/if}
				</Badge>
				<div class="flex items-center gap-2">
					<h1 id="list-title" class="text-primary min-w-0 text-[2rem] leading-tight font-medium tracking-[-0.025em] wrap-break-word md:text-[2.5rem]">{list.name}</h1>
					<DropdownMenu.Root>
						<DropdownMenu.Trigger
							class={buttonVariants({ variant: "ghost", size: "icon-sm", class: "text-muted-foreground hover:text-foreground shrink-0 cursor-pointer rounded-full" })}
							aria-label="Liste seçenekleri"><PencilIcon class="size-4" /></DropdownMenu.Trigger
						>
						<DropdownMenu.Content align="start" class="w-48 rounded-xl p-1.5">
							<DropdownMenu.Item class="cursor-pointer rounded-lg" onclick={() => openSettings("details")}>Listeyi düzenle</DropdownMenu.Item>
							<DropdownMenu.Item class="cursor-pointer rounded-lg" onclick={() => openSettings("backdrop")}>Kapağı değiştir</DropdownMenu.Item>
							<DropdownMenu.Item class="cursor-pointer rounded-lg" onclick={() => openSettings("image")}>Görseli değiştir</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				</div>
				{#if list.description}
					<p class="text-muted-foreground mt-2 max-w-xl text-sm leading-6 wrap-break-word whitespace-pre-line">{list.description}</p>
				{/if}
				<p class="text-muted-foreground mt-3 text-xs font-medium">{list.items.length} oyun · {text.meta}</p>
			</div>
		</div>
	</section>

	<section aria-label="Liste içeriği" class="mt-12 min-w-0 px-4 md:px-8">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<p class="text-sm font-medium tabular-nums" aria-live="polite">{visibleItems.length} / {list.items.length} oyun</p>
			<div class="flex w-full flex-col gap-2 sm:w-auto sm:flex-row sm:items-center">
				<InputGroup.Root class="h-10 rounded-xl sm:w-64">
					<InputGroup.Addon><SearchIcon /></InputGroup.Addon>
					<InputGroup.Input bind:value={query} aria-label="Listede ara" placeholder="Listede ara…" />
					{#if query}
						<InputGroup.Addon align="inline-end">
							<InputGroup.Button size="icon-xs" class="cursor-pointer rounded-full" aria-label="Aramayı temizle" onclick={() => (query = "")}><XIcon /></InputGroup.Button>
						</InputGroup.Addon>
					{/if}
				</InputGroup.Root>
				<Select.Root type="single" bind:value={selectedSort}>
					<Select.Trigger class="h-10 w-48" aria-label="Sırala: {selectedSortLabel}">
						<span data-slot="select-value"><ArrowDownWideNarrowIcon class="text-muted-foreground" />{selectedSortLabel}</span>
					</Select.Trigger>
					<Select.Content align="end" class="rounded-xl">
						{#each sortOptions as option (option.value)}
							<Select.Item value={option.value} class="rounded-lg">{option.label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
		</div>

		<div class="pt-6">
			{#if visibleItems.length}
				<div class="grid grid-cols-2 gap-x-4 gap-y-8 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
					{#each visibleItems as game (game.slug)}
						<article
							class="min-w-0"
							onpointerenter={(event) => {
								if (event.pointerType === "mouse") flippedSlug = game.slug;
							}}
							onpointerleave={(event) => {
								if (event.pointerType === "mouse" && flippedSlug === game.slug) flippedSlug = null;
							}}
						>
							<div class="flip-perspective relative aspect-2/3">
								<div class="flip-turn relative size-full" class:is-flipped={flippedSlug === game.slug}>
									<button
										type="button"
										class="flip-face bg-muted ring-foreground/10 focus-visible:ring-ring/50 absolute inset-0 cursor-pointer overflow-hidden rounded-xl ring-1 outline-none focus-visible:ring-3"
										inert={flippedSlug === game.slug}
										aria-label="{game.title} seçenekleri"
										onclick={() => (flippedSlug = game.slug)}
									>
										<img src={game.cover} alt="{game.title} kapağı" width="1024" height="1536" loading="lazy" class="size-full object-cover" />
									</button>
									<div
										class="flip-face flip-back bg-card text-card-foreground ring-foreground/10 absolute inset-0 flex flex-col items-center justify-center gap-3 rounded-xl p-4 text-center ring-1"
										inert={flippedSlug !== game.slug}
									>
										{@render listIcon("size-6 text-muted-foreground")}
										<p class="line-clamp-2 text-sm font-medium">{game.title}</p>
										<Button variant="default" size="sm" href="/oyun/{game.slug}" class="w-full cursor-pointer rounded-full text-xs">Oyuna git</Button>
										<Button variant="outline" size="sm" class="w-full cursor-pointer rounded-full text-xs" onclick={() => removeItem(game)}>{text.removeLabel}</Button>
									</div>
								</div>
							</div>
							<h3 class="text-primary truncate px-0.5 pt-3 text-sm font-medium md:text-[15px]">{game.title}</h3>
							<p class="text-muted-foreground mt-0.5 flex items-center gap-1.5 px-0.5 text-xs">
								<StarIcon class="fill-current text-foreground size-3" aria-hidden="true" />
								<span class="text-foreground font-medium tabular-nums">{game.score.toFixed(1).replace(".", ",")}</span>
								<span>· {game.year}</span>
							</p>
						</article>
					{/each}
				</div>
			{:else}
				<Empty.Root class="min-h-80 rounded-xl border border-dashed">
					<Empty.Header>
						<Empty.Media variant="icon">
							{#if list.items.length}<SearchIcon />{:else if list.kind === "favorites"}<HeartIcon />{:else if list.kind === "play_later"}<ClockIcon />{:else}<GamepadIcon />{/if}
						</Empty.Media>
						<Empty.Title>{list.items.length ? "Eşleşen oyun yok" : text.emptyTitle}</Empty.Title>
						<Empty.Description>{list.items.length ? "Başka bir ad dene ya da aramayı temizle." : text.emptyDescription}</Empty.Description>
					</Empty.Header>
					<Empty.Content>
						{#if list.items.length}
							<Button variant="outline" class="cursor-pointer rounded-full" onclick={() => (query = "")}>Aramayı temizle</Button>
						{:else}
							<Button href="/oyunlar" class="cursor-pointer rounded-full">
								Oyun keşfet
								<ArrowUpRightIcon data-icon="inline-end" class="size-4" />
							</Button>
						{/if}
					</Empty.Content>
				</Empty.Root>
			{/if}
		</div>
		<p class="text-muted-foreground mt-8 text-xs leading-5">Koleksiyonunun önizlemesi. Sayfayı yenilediğinde değişiklikler sıfırlanır.</p>
	</section>
</div>

<Dialog.Root bind:open={editOpen}>
	<Dialog.Content>
		<Dialog.Header class="text-left">
			<Dialog.Title>
				{editMode === "details" ? "Listeyi düzenle" : editMode === "image" ? "Görseli değiştir" : "Kapağı değiştir"}
			</Dialog.Title>
			<Dialog.Description>
				{editMode === "details" ? "Bu listeyi kendine göre ayarla." : editMode === "image" ? "Koleksiyonun için bir görsel seç." : "Koleksiyonunun arka planını belirle."}
			</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-5" onsubmit={saveSettings}>
			{#if editMode !== "details"}
				<div class="space-y-3">
					<div class={editMode === "image" ? "bg-muted mx-auto flex size-36 items-center justify-center overflow-hidden rounded-xl border" : "bg-muted h-36 overflow-hidden rounded-xl border"}>
						{#if editMode === "backdrop"}
							<img src={draftBackdrop} alt="Kapak önizlemesi" class="size-full object-cover" />
						{:else if draftImage}
							<img src={draftImage} alt="Liste görseli önizlemesi" class="size-full object-cover" />
						{:else}
							{@render listIcon("size-8 text-muted-foreground")}
						{/if}
					</div>
					<Label for="list-artwork">{editMode === "image" ? "Liste görseli" : "Kapak görseli"}</Label>
					<Input
						id="list-artwork"
						type="file"
						accept="image/jpeg,image/png,image/webp,image/gif"
						disabled={pendingImages > 0}
						onchange={(event) => selectImage(event, editMode === "image" ? "image" : "backdrop")}
						aria-describedby="list-image-help"
					/>
					<p id="list-image-help" class="text-muted-foreground text-xs">JPG, PNG, WebP ya da GIF, en fazla 5 MB.</p>
					{#if imageError}<p class="text-destructive text-sm" role="alert">{imageError}</p>{/if}
				</div>
			{:else}
				<div class="space-y-2">
					<Label for="list-name">Liste adı</Label>
					<Input id="list-name" bind:value={draftName} maxlength={80} required aria-invalid={nameError ? true : undefined} />
					{#if nameError}<p class="text-destructive text-sm" role="alert">{nameError}</p>{/if}
				</div>
				<div class="space-y-2">
					<Label for="list-description">Açıklama <span class="text-muted-foreground font-normal">(isteğe bağlı)</span></Label>
					<Textarea id="list-description" bind:value={draftDescription} maxlength={280} rows={3} placeholder="Bu koleksiyonu özel yapan ne?" class="max-h-40 resize-y" />
				</div>
				<div class="space-y-2">
					<Label id="list-visibility-label">Görünürlük</Label>
					<Select.Root type="single" bind:value={draftVisibility}>
						<Select.Trigger class="h-10 w-full" aria-labelledby="list-visibility-label">
							<span data-slot="select-value">
								{#if draftVisibility === "private"}<LockIcon />Gizli{:else}<GlobeIcon />Herkese açık{/if}
							</span>
						</Select.Trigger>
						<Select.Content class="rounded-xl">
							<Select.Item value="private" class="rounded-lg"><LockIcon class="size-4" />Gizli</Select.Item>
							<Select.Item value="public" class="rounded-lg"><GlobeIcon class="size-4" />Herkese açık</Select.Item>
						</Select.Content>
					</Select.Root>
					<p class="text-muted-foreground text-xs leading-5">
						{draftVisibility === "private" ? "Bu listeyi yalnızca sen görebilirsin." : "Bu liste profilinde herkese görünür."}
					</p>
				</div>
			{/if}
			<Dialog.Footer>
				<Button type="button" variant="outline" class="cursor-pointer" onclick={() => (editOpen = false)}>Vazgeç</Button>
				<Button type="submit" class="cursor-pointer" disabled={pendingImages > 0}>Değişiklikleri kaydet</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<style>
	.list-hero {
		--offset: var(--sidebar-offset, 0px);
		margin-top: -4rem;
		margin-left: calc(-1 * var(--offset));
		padding-left: var(--offset);
	}

	.list-scrim {
		background:
			linear-gradient(0deg, var(--background) 0%, color-mix(in oklch, var(--background) 70%, transparent) 35%, transparent 75%),
			linear-gradient(90deg, color-mix(in oklch, var(--background) 70%, transparent) 0%, color-mix(in oklch, var(--background) 30%, transparent) 60%, color-mix(in oklch, var(--background) 20%, transparent) 100%),
			linear-gradient(180deg, color-mix(in oklch, var(--background) 55%, transparent) 0%, transparent 22%);
	}

	.flip-perspective {
		perspective: 1000px;
	}

	.flip-turn {
		transform-style: preserve-3d;
		transition: transform 450ms cubic-bezier(0.2, 0.7, 0.2, 1);
	}

	.flip-turn.is-flipped {
		transform: rotateY(180deg);
	}

	.flip-face {
		backface-visibility: hidden;
	}

	.flip-back {
		transform: rotateY(180deg);
	}

	@media (prefers-reduced-motion: reduce) {
		.flip-turn {
			transition: none;
		}
	}
</style>
