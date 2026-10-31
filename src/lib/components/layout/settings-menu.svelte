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
		{ value: "light", label: "Açık", icon: SunIcon },
		{ value: "dark", label: "Koyu", icon: MoonIcon },
		{ value: "system", label: "Sistem", icon: MonitorIcon },
	] as const;
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger class={cn(buttonVariants({ variant: "ghost", size: "icon-lg" }), "cursor-pointer rounded-full")} aria-label="Ayarlar">
		<SettingsIcon class="size-5" />
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end" sideOffset={8} class="w-44 rounded-2xl p-1.5">
		<DropdownMenu.Group>
			<DropdownMenu.Label class="text-muted-foreground text-xs">Tema</DropdownMenu.Label>
			<DropdownMenu.RadioGroup bind:value={() => userPrefersMode.current, (value) => setMode(value as "light" | "dark" | "system")}>
				{#each themes as theme (theme.value)}
					<DropdownMenu.RadioItem value={theme.value} class="cursor-pointer rounded-xl">
						<theme.icon />
						{theme.label}
					</DropdownMenu.RadioItem>
				{/each}
			</DropdownMenu.RadioGroup>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>
