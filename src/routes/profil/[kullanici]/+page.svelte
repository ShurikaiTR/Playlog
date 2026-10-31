<script lang="ts">
	import * as Avatar from "#lib/components/ui/avatar/index.js";
	import * as Card from "#lib/components/ui/card/index.js";
	import * as Dialog from "#lib/components/ui/dialog/index.js";
	import { AspectRatio } from "#lib/components/ui/aspect-ratio/index.js";
	import { Button } from "#lib/components/ui/button/index.js";
	import * as Item from "#lib/components/ui/item/index.js";
	import { ScrollArea } from "#lib/components/ui/scroll-area/index.js";
	import { listHref, ownLists } from "#lib/user-lists.svelte.js";
	import ClockIcon from "@lucide/svelte/icons/clock-3";
	import GamepadIcon from "@lucide/svelte/icons/gamepad-2";
	import HeartIcon from "@lucide/svelte/icons/heart";

	let { data } = $props();
	const profile = $derived(data.profile);

	const initial = (name: string) => name.charAt(0).toLocaleUpperCase("tr");
	let following = $state(false);

	const connections = $derived([
		{
			key: "following",
			label: "Takip edilen",
			description: data.isOwner ? "Takip ettiğin kişiler." : `@${profile.username} kullanıcısının takip ettiği kişiler.`,
			empty: data.isOwner ? "Henüz kimseyi takip etmiyorsun." : "Henüz kimseyi takip etmiyor.",
			people: profile.following,
		},
		{
			key: "followers",
			label: "Takipçi",
			description: data.isOwner ? "Seni takip eden kişiler." : `@${profile.username} kullanıcısını takip eden kişiler.`,
			empty: data.isOwner ? "Henüz takipçin yok." : "Henüz takipçisi yok.",
			people: profile.followers,
		},
	] as const);

	// The owner sees their live lists (same data as the list pages); visitors see the profile's sample lists.
	const lists = $derived(
		data.isOwner
			? ownLists().map((list) => ({
					id: list.id,
					kind: list.kind === "play_later" ? ("play_later" as const) : list.kind,
					name: list.name,
					description: list.description || undefined,
					cover: list.backdrop,
					gameCount: list.items.length,
					href: listHref(list) as string | null,
				}))
			: profile.lists.map((list) => ({ ...list, href: null as string | null })),
	);

	const listIcon = { favorites: HeartIcon, play_later: ClockIcon, custom: GamepadIcon };
</script>

<svelte:head>
	<title>{profile.name} Profili | Playlog</title>
</svelte:head>

<div lang="tr" class="flex min-w-0 flex-col gap-14 pb-16">
	<section aria-labelledby="profile-name" class="profile-hero relative isolate min-w-0">
		<!-- Full-bleed: slides under the header and the floating sidebar. -->
		<div class="pointer-events-none absolute inset-0 -z-10 overflow-hidden" aria-hidden="true" style="mask-image: linear-gradient(to bottom, black 55%, transparent 100%)">
			<img src={profile.cover} alt="" width="1600" height="900" fetchpriority="high" class="size-full object-cover object-[center_45%]" />
			<div class="profile-scrim absolute inset-0"></div>
		</div>

		<div class="flex min-w-0 flex-col gap-6 px-4 pt-36 pb-2 sm:flex-row sm:items-end sm:justify-between md:px-8 md:pt-44">
			<div class="flex min-w-0 items-end gap-4 sm:gap-6">
				<Avatar.Root class="ring-background size-24 shrink-0 shadow-sm ring-4 sm:size-32">
					{#if profile.image}<Avatar.Image src={profile.image} alt="" />{/if}
					<Avatar.Fallback class="bg-foreground text-background text-3xl font-medium sm:text-5xl">{initial(profile.name)}</Avatar.Fallback>
				</Avatar.Root>
				<div class="min-w-0 pb-1">
					<h1 id="profile-name" class="text-primary truncate text-[2rem] leading-tight font-medium tracking-[-0.025em] md:text-[2.5rem]">{profile.name}</h1>
					<p class="text-muted-foreground mt-0.5 text-sm">@{profile.username}</p>
					<p class="text-muted-foreground mt-3 max-w-xl text-sm leading-6">{profile.bio ?? "Henüz bir biyografi yok."}</p>
				</div>
			</div>

			<div class="flex shrink-0 flex-wrap items-center gap-2 pb-1">
				{#each connections as connection (connection.key)}
					<Dialog.Root>
						<Dialog.Trigger>
							{#snippet child({ props })}
								<Button {...props} variant="outline" class="bg-background/70 h-auto cursor-pointer flex-col items-start gap-0 rounded-xl px-4 py-2 text-left backdrop-blur-sm">
									<span class="text-lg leading-tight font-semibold tabular-nums">{connection.people.length}</span>
									<span class="text-muted-foreground text-xs font-normal">{connection.label}</span>
								</Button>
							{/snippet}
						</Dialog.Trigger>
						<Dialog.Content>
							<Dialog.Header class="text-left">
								<Dialog.Title>
									{connection.label}
									<span class="text-muted-foreground ml-1 text-sm">{connection.people.length}</span>
								</Dialog.Title>
								<Dialog.Description>{connection.description}</Dialog.Description>
							</Dialog.Header>
							<ScrollArea class="[&>[data-slot=scroll-area-viewport]]:max-h-[min(24rem,60dvh)]">
								{#if connection.people.length}
									<Item.Group class="gap-0">
										{#each connection.people as person, index (person.username)}
											{#if index > 0}<Item.Separator class="my-0" />{/if}
											{#snippet row()}
												<Item.Media>
													<Avatar.Root class="size-10">
														{#if person.image}<Avatar.Image src={person.image} alt="" />{/if}
														<Avatar.Fallback class="bg-foreground text-background text-xs font-semibold">{initial(person.name)}</Avatar.Fallback>
													</Avatar.Root>
												</Item.Media>
												<Item.Content>
													<Item.Title>{person.name}</Item.Title>
													<Item.Description>@{person.username}</Item.Description>
												</Item.Content>
											{/snippet}
											{#if data.linkable.includes(person.username)}
												<Item.Root size="xs">
													{#snippet child({ props })}
														<a {...props} href="/profil/{person.username}">{@render row()}</a>
													{/snippet}
												</Item.Root>
											{:else}
												<Item.Root size="xs">{@render row()}</Item.Root>
											{/if}
										{/each}
									</Item.Group>
								{:else}
									<p class="text-muted-foreground py-4 text-sm">{connection.empty}</p>
								{/if}
							</ScrollArea>
						</Dialog.Content>
					</Dialog.Root>
				{/each}

				{#if data.isOwner}
					<Button variant="outline" class="bg-background/70 h-10 cursor-pointer rounded-full px-5 backdrop-blur-sm">Profili düzenle</Button>
				{:else}
					<Button variant={following ? "outline" : "default"} class="h-10 cursor-pointer rounded-full px-5" onclick={() => (following = !following)}>
						{following ? "Takibi bırak" : "Takip et"}
					</Button>
				{/if}
			</div>
		</div>
	</section>

	<section class="min-w-0 space-y-6 px-4 md:px-8" aria-labelledby="profile-lists-title">
		<div>
			<h2 id="profile-lists-title" class="text-primary truncate text-2xl font-medium">Oyun listeleri</h2>
			<p class="text-muted-foreground mt-1 text-sm">Favoriler ve sırada bekleyenler.</p>
		</div>

		{#if lists.length === 0}
			<p class="text-muted-foreground text-sm">Herkese açık bir liste yok.</p>
		{:else}
			<div class="grid gap-4 sm:grid-cols-2 xl:max-w-3xl">
				{#each lists as list (list.id)}
					{@const Icon = listIcon[list.kind]}
					{#snippet listCard()}
						<div class="relative">
							<AspectRatio ratio={2.6} class="bg-muted">
								<img src={list.cover} alt="" width="1600" height="560" loading="lazy" class="size-full object-cover" />
							</AspectRatio>
							<Avatar.Root class="ring-card absolute -bottom-7 left-(--card-spacing) size-16 rounded-xl ring-4">
								<Avatar.Fallback class="rounded-xl"><Icon class="text-muted-foreground size-6" strokeWidth={1.5} /></Avatar.Fallback>
							</Avatar.Root>
						</div>
						<Card.Header class="pt-10">
							<Card.Title class="truncate">{list.name}</Card.Title>
							<Card.Description class="truncate">{list.gameCount} oyun{list.description ? ` · ${list.description}` : ""}</Card.Description>
						</Card.Header>
					{/snippet}
					<Card.Root class="group/list gap-0 pt-0 pb-5 {list.href ? 'transition-transform duration-300 hover:-translate-y-1' : ''}">
						{#if list.href}
							<a href={list.href} class="focus-visible:ring-ring/50 flex flex-col rounded-2xl outline-none focus-visible:ring-3" aria-label="{list.name} listesini gör">{@render listCard()}</a>
						{:else}
							<div class="flex flex-col">{@render listCard()}</div>
						{/if}
					</Card.Root>
				{/each}
			</div>
		{/if}
	</section>
</div>

<style>
	.profile-hero {
		--offset: var(--sidebar-offset, 0px);
		margin-top: -4rem;
		margin-left: calc(-1 * var(--offset));
		padding-left: var(--offset);
	}

	.profile-scrim {
		background:
			linear-gradient(0deg, var(--background) 0%, color-mix(in oklch, var(--background) 70%, transparent) 35%, transparent 75%),
			linear-gradient(90deg, color-mix(in oklch, var(--background) 70%, transparent) 0%, color-mix(in oklch, var(--background) 30%, transparent) 60%, color-mix(in oklch, var(--background) 20%, transparent) 100%),
			linear-gradient(180deg, color-mix(in oklch, var(--background) 55%, transparent) 0%, transparent 22%);
	}
</style>
