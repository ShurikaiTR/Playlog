<script lang="ts">
	import { page } from "$app/state";
	import * as Sidebar from "#lib/components/ui/sidebar/index.js";
	import type { Component } from "svelte";
	import CompassIcon from "@lucide/svelte/icons/compass";
	import HeartIcon from "@lucide/svelte/icons/heart";
	import HouseIcon from "@lucide/svelte/icons/house";
	import LayoutGridIcon from "@lucide/svelte/icons/layout-grid";
	import LibraryIcon from "@lucide/svelte/icons/library-big";
	import ListIcon from "@lucide/svelte/icons/list";
	import MessageSquareIcon from "@lucide/svelte/icons/message-square";
	import NewspaperIcon from "@lucide/svelte/icons/newspaper";
	import TagIcon from "@lucide/svelte/icons/tag";
	import TrendingUpIcon from "@lucide/svelte/icons/trending-up";
	import UserRoundIcon from "@lucide/svelte/icons/user-round";
	import UsersIcon from "@lucide/svelte/icons/users";

	interface Item {
		label: string;
		href: string;
		icon: Component;
	}

	// Placeholder destinations until the real pages exist.
	const groups: { label: string; items: Item[] }[] = [
		{
			label: "Göz at",
			items: [
				{ label: "Ana sayfa", href: "/", icon: HouseIcon },
				{ label: "Keşfet", href: "/kesfet", icon: CompassIcon },
				{ label: "Tüm oyunlar", href: "/oyunlar", icon: LayoutGridIcon },
				{ label: "Popüler", href: "/kesfet/populer", icon: TrendingUpIcon },
				{ label: "Türler", href: "/kesfet/turler", icon: TagIcon },
			],
		},
		{
			label: "Topluluk",
			items: [
				{ label: "İncelemeler", href: "/incelemeler", icon: MessageSquareIcon },
				{ label: "Listeler", href: "/listeler", icon: ListIcon },
				{ label: "Kullanıcılar", href: "/kullanicilar", icon: UsersIcon },
			],
		},
		{
			label: "Kütüphane",
			items: [
				{ label: "Kütüphanem", href: "/kutuphane", icon: LibraryIcon },
				{ label: "Favoriler", href: "/favoriler", icon: HeartIcon },
			],
		},
		{
			label: "Playlog",
			items: [{ label: "Blog", href: "/blog", icon: NewspaperIcon }],
		},
	];

	const sidebar = Sidebar.useSidebar();
	const isActive = (href: string) => (href === "/" ? page.url.pathname === "/" : page.url.pathname.startsWith(href));
</script>

<Sidebar.Root
	collapsible="icon"
	variant="floating"
	class="overflow-hidden [&_[data-slot=sidebar-inner]]:overflow-hidden [&_[data-slot=sidebar-inner]]:rounded-2xl"
>
	<Sidebar.Header class="p-2">
		<div class="group/brand relative flex h-10 items-center gap-2 overflow-hidden px-1">
			<a
				href="/"
				class="text-primary focus-visible:ring-ring/50 flex min-w-0 flex-1 items-center gap-2 rounded-full outline-none focus-visible:ring-[3px] group-data-[collapsible=icon]:hidden"
				aria-label="Playthru ana sayfa"
			>
				<img src="/logos/playthru.svg" alt="Playthru" width="282" height="56" class="h-8 w-auto dark:invert" />
			</a>
			<a
				href="/"
				class="text-primary hidden size-10 items-center justify-center transition-opacity group-hover/brand:opacity-0 group-data-[collapsible=icon]:flex"
				aria-label="Playthru ana sayfa"
			>
				<img src="/logos/playthru-mascot.svg" alt="Playthru" width="100" height="100" class="size-8 object-contain dark:invert" />
			</a>
			<Sidebar.Trigger
				class="shrink-0 group-hover/brand:!opacity-100 group-data-[collapsible=icon]:absolute group-data-[collapsible=icon]:left-1/2 group-data-[collapsible=icon]:-translate-x-1/2 group-data-[collapsible=icon]:opacity-0 group-data-[collapsible=icon]:transition-opacity"
			/>
		</div>
	</Sidebar.Header>

	<Sidebar.Content>
		{#each groups as group (group.label)}
			<Sidebar.Group>
				<Sidebar.GroupLabel>{group.label}</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each group.items as item (item.href)}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton
									isActive={isActive(item.href)}
									tooltipContent={item.label}
									class="font-[550]"
								>
									{#snippet child({ props })}
										<a
											{...props}
											href={item.href}
											aria-current={isActive(item.href) ? "page" : undefined}
											onclick={() => sidebar.isMobile && sidebar.setOpenMobile(false)}
										>
											<div
												data-sidebar="icon"
												class="flex size-6 shrink-0 items-center justify-center"
												aria-hidden="true"
											>
												<item.icon class="size-[18px]!" />
											</div>
											<span>{item.label}</span>
										</a>
									{/snippet}
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>

	<Sidebar.Footer class="p-3">
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton
					size="lg"
					tooltipContent="Giriş yap"
					class="h-10 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:p-0!"
				>
					{#snippet child({ props })}
						<a {...props} href="/giris">
							<div class="bg-secondary flex size-7 shrink-0 items-center justify-center rounded-full">
								<UserRoundIcon class="size-4!" aria-hidden="true" />
							</div>
							<span class="truncate font-[550] group-data-[collapsible=icon]:hidden">Giriş yap</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>

	<Sidebar.Rail />
</Sidebar.Root>
