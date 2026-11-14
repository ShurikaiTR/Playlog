<script lang="ts">
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import { playStatuses, setStatus, statusOf, type PlayStatus } from './user-lists.svelte.js';
	import { cn } from '#lib/utils.js';
	import { toast } from 'svelte-sonner';
	import BanIcon from '@lucide/svelte/icons/ban';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import GamepadIcon from '@lucide/svelte/icons/gamepad-2';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	let { game, class: className }: { game: { slug: string; title: string }; class?: string } =
		$props();

	const icons = { Playing: GamepadIcon, Completed: CircleCheckIcon, Dropped: BanIcon };
	const status = $derived(statusOf(game.slug));

	function change(next: string) {
		const previous = status;
		if (next === previous) return;
		setStatus(game.slug, next as PlayStatus | '');
		toast(next ? `Marked as ${next.toLowerCase()}` : 'Status removed', {
			action: { label: 'Undo', onClick: () => setStatus(game.slug, previous) }
		});
	}
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger
		class={cn(
			buttonVariants({ variant: 'default', size: 'lg' }),
			'h-11 min-w-0 flex-1 gap-0 px-0 text-sm',
			className
		)}
		aria-label="Status of {game.title}"
	>
		<span class="flex min-w-0 flex-1 items-center gap-2 px-4">
			<span class="truncate">{status || 'Set status'}</span>
		</span>
		<span aria-hidden="true" class="h-5 w-px shrink-0 bg-current opacity-20"></span>
		<ChevronDownIcon class="mx-3 size-4" aria-hidden="true" />
	</DropdownMenu.Trigger>

	<DropdownMenu.Content align="start" sideOffset={8} class="w-56 p-2">
		<DropdownMenu.Label class="px-2 pt-1 pb-2 text-xs font-semibold text-primary"
			>Status</DropdownMenu.Label
		>
		<DropdownMenu.RadioGroup value={status} onValueChange={change}>
			{#each playStatuses as value (value)}
				{@const Icon = icons[value]}
				<DropdownMenu.RadioItem {value} closeOnSelect>
					<Icon class="size-4" aria-hidden="true" />
					{value}
				</DropdownMenu.RadioItem>
			{/each}
		</DropdownMenu.RadioGroup>

		{#if status}
			<DropdownMenu.Separator class="my-2" />
			<DropdownMenu.Item variant="destructive" onclick={() => change('')}>
				<Trash2Icon class="size-4" aria-hidden="true" />
				Clear status
			</DropdownMenu.Item>
		{/if}
	</DropdownMenu.Content>
</DropdownMenu.Root>
