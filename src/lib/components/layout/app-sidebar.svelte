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
	import UsersIcon from "@lucide/svelte/icons/users";
	import SearchIcon from "@lucide/svelte/icons/search";
	import { Button } from "#lib/components/ui/button/index.js";
	import { Kbd, KbdGroup } from "#lib/components/ui/kbd/index.js";
	import * as Tooltip from "#lib/components/ui/tooltip/index.js";
	import { openSearch } from "#lib/features/search/search-state.svelte.js";
	import UserMenu from "./user-menu.svelte";
	import SettingsDialog from "./settings-dialog.svelte";

	interface Item {
		label: string;
		href: string;
		icon: Component;
	}

	// Placeholder destinations until the real pages exist.
	const groups: { label: string; items: Item[] }[] = [
		{
			label: "Browse",
			items: [
				{ label: "Home", href: "/", icon: HouseIcon },
				{ label: "Discover", href: "/kesfet", icon: CompassIcon },
				{ label: "All games", href: "/oyunlar", icon: LayoutGridIcon },
				{ label: "Popular", href: "/kesfet/populer", icon: TrendingUpIcon },
				{ label: "Genres", href: "/kesfet/turler", icon: TagIcon },
			],
		},
		{
			label: "Community",
			items: [
				{ label: "Reviews", href: "/incelemeler", icon: MessageSquareIcon },
				{ label: "Lists", href: "/listeler", icon: ListIcon },
				{ label: "Users", href: "/kullanicilar", icon: UsersIcon },
			],
		},
		{
			label: "Library",
			items: [
				{ label: "My library", href: "/kutuphane", icon: LibraryIcon },
				{ label: "Favorites", href: "/favoriler", icon: HeartIcon },
			],
		},
		{
			label: "Playlog",
			items: [{ label: "Blog", href: "/blog", icon: NewspaperIcon }],
		},
	];

	const sidebar = Sidebar.useSidebar();
	// Only read while a tooltip is open, so it never runs during server rendering.
	const isMac = () => /Mac|iPhone|iPad/.test(navigator.userAgent);

	function search() {
		if (sidebar.isMobile) sidebar.setOpenMobile(false);
		openSearch();
	}

	const isActive = (href: string) => (href === "/" ? page.url.pathname === "/" : page.url.pathname.startsWith(href));
</script>

<Sidebar.Root
	collapsible="icon"
	variant="floating"
	class="overflow-hidden [&_[data-slot=sidebar-inner]]:overflow-hidden [&_[data-slot=sidebar-inner]]:rounded-2xl"
>
	<Sidebar.Header class="p-2">
		<div class="group/brand relative flex h-10 items-center pointer-coarse:h-11 gap-2 overflow-hidden px-1">
			<a
				href="/"
				class="text-primary focus-visible:ring-ring/50 flex min-w-0 flex-1 items-center gap-2 rounded-full outline-none focus-visible:ring-[3px] group-data-[collapsible=icon]:hidden"
				aria-label="Playthru home"
			>
				<img src="/logos/playthru.svg" alt="Playthru" width="282" height="56" class="h-8 w-auto dark:invert" />
			</a>
			<a
				href="/"
				class="text-primary hidden size-10 items-center justify-center transition-opacity group-hover/brand:opacity-0 group-data-[collapsible=icon]:flex"
				aria-label="Playthru home"
			>
				<img src="/logos/playthru-mascot.svg" alt="Playthru" width="100" height="100" class="size-8 object-contain dark:invert" />
			</a>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="ghost"
							size="icon-xs"
							class="text-muted-foreground hover:text-primary hover:bg-foreground/8 dark:hover:bg-foreground/12 size-8 shrink-0 cursor-pointer rounded-xl transition-colors duration-100 pointer-coarse:size-11 group-data-[collapsible=icon]:hidden"
							aria-label="Search"
							aria-keyshortcuts="Meta+K Control+K"
							onclick={search}
						>
							<SearchIcon class="size-[18px]!" aria-hidden="true" />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="bottom" sideOffset={8} class="flex items-center gap-2">
					Ara
					<KbdGroup><Kbd>{isMac() ? "⌘" : "Ctrl"}</Kbd><Kbd>K</Kbd></KbdGroup>
				</Tooltip.Content>
			</Tooltip.Root>
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
						{#if group === groups[0]}
							<!-- On the icon rail the header button is hidden, so search gets a row of its own. -->
							<Sidebar.MenuItem class="hidden group-data-[collapsible=icon]:block">
								<Sidebar.MenuButton tooltipContent="Ara" class="cursor-pointer font-[550]" onclick={search}>
									<div data-sidebar="icon" class="flex size-6 shrink-0 items-center justify-center" aria-hidden="true">
										<SearchIcon class="size-4!" />
									</div>
									<span>Search</span>
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/if}
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
												<item.icon class="size-4!" />
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

	{#if page.data.user}
		<Sidebar.Footer class="p-2">
			<UserMenu user={page.data.user} />
		</Sidebar.Footer>
		<SettingsDialog />
	{/if}

	<Sidebar.Rail />
</Sidebar.Root>
