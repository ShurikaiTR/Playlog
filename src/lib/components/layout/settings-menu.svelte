<script lang="ts">
	import { setMode, userPrefersMode } from "mode-watcher";
	import { buttonVariants } from "#lib/components/ui/button/index.js";
	import * as DropdownMenu from "#lib/components/ui/dropdown-menu/index.js";
	import { cn } from "#lib/utils.js";
	import SettingsIcon from "@lucide/svelte/icons/settings";
	import SunIcon from "@lucide/svelte/icons/sun";
	import MoonIcon from "@lucide/svelte/icons/moon";
	import MonitorIcon from "@lucide/svelte/icons/monitor";

	const themes = [
		{ value: "light", label: "Açık tema", icon: SunIcon },
		{ value: "dark", label: "Koyu tema", icon: MoonIcon },
		{ value: "system", label: "Sistem teması", icon: MonitorIcon },
	] as const;
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger class={cn(buttonVariants({ variant: "ghost", size: "icon-lg" }), "cursor-pointer rounded-full")} aria-label="Ayarlar" title="Ayarlar">
		<SettingsIcon class="size-5" strokeWidth={1.8} />
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end" sideOffset={10} class="w-56 rounded-2xl p-2">
		<div class="grid grid-cols-3 gap-1" role="group" aria-label="Tema seç">
			{#each themes as theme (theme.value)}
				<DropdownMenu.Item
					aria-label={theme.label}
					title={theme.label}
					class={cn(
						"focus:bg-muted focus:text-foreground h-10 cursor-pointer justify-center rounded-xl p-0",
						userPrefersMode.current === theme.value && "bg-muted text-foreground",
					)}
					onclick={() => setMode(theme.value)}
				>
					<theme.icon class="size-[18px]" strokeWidth={1.7} />
				</DropdownMenu.Item>
			{/each}
		</div>
	</DropdownMenu.Content>
</DropdownMenu.Root>
