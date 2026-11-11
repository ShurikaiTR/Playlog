<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';

	// "Report a problem" with a game's information.
	let { title, open = $bindable(false) }: { title: string; open: boolean } = $props();

	let reason = $state('');
	let details = $state('');
	let notice = $state('');

	// A notice from an earlier attempt doesn't carry over to the next time the dialog opens.
	$effect(() => {
		if (open) notice = '';
	});

	function submit(event: SubmitEvent) {
		event.preventDefault();
		notice = 'Bildirimler şu anda kullanılamıyor. Bildirimin gönderilmedi.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Sorun bildir</Dialog.Title>
			<Dialog.Description
				>{title} bilgilerinde neyin düzeltilmesi gerektiğini bize anlat.</Dialog.Description
			>
		</Dialog.Header>
		<form class="space-y-4" onsubmit={submit}>
			<div class="space-y-2">
				<Label for="report-reason">Neden</Label>
				<Select.Root type="single" bind:value={reason}>
					<Select.Trigger id="report-reason" class="h-10 w-full">
						<span data-slot="select-value">{reason || 'Bir neden seç'}</span>
					</Select.Trigger>
					<Select.Content class="rounded-xl">
						{#each ['Yanlış bilgi', 'Yanlış görsel', 'Eksik bilgi', 'Diğer'] as reason (reason)}
							<Select.Item value={reason} class="rounded-lg">{reason}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="space-y-2">
				<Label for="report-details">Ayrıntılar</Label>
				<Textarea
					id="report-details"
					bind:value={details}
					placeholder="Sorunu anlat…"
					class="min-h-24 resize-y"
				/>
			</div>
			{#if notice}<p class="text-xs leading-5 text-muted-foreground" role="status">{notice}</p>{/if}
			<Dialog.Footer>
				<Button
					type="button"
					variant="outline"
					class="cursor-pointer"
					onclick={() => (open = false)}>Vazgeç</Button
				>
				<Button type="submit" class="cursor-pointer">Bildirimi gönder</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
