<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import * as Empty from '#lib/components/ui/empty/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import { cn } from '#lib/utils.js';
	import UploadIcon from '@lucide/svelte/icons/upload';
	import XIcon from '@lucide/svelte/icons/x';

	// Rendered inside the dialog content, so the form starts empty every time the dialog opens.
	let { onsent }: { onsent: () => void } = $props();

	const soft =
		'focus:bg-muted focus:text-foreground focus:**:text-foreground not-data-[variant=destructive]:focus:**:text-foreground';
	const feedbackReasons = ['Hata', 'Öneri', 'Diğer'];
	const maxAttachments = 5;
	let feedbackReason = $state('');
	let feedbackText = $state('');
	let attachments = $state<File[]>([]);
	let fileInput = $state<HTMLInputElement | null>(null);
	let dragging = $state(false);
	const canSend = $derived(!!feedbackReason && !!feedbackText.trim());

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
		onsent();
		toast('Geri bildirimin için teşekkürler');
	}
</script>

<form class="flex flex-col gap-4" onsubmit={sendFeedback}>
	<div>
		<Label for="feedback-reason" class="mb-2 font-semibold">Tür</Label>
		<Select.Root type="single" bind:value={feedbackReason}>
			<Select.Trigger id="feedback-reason" class="h-10! w-full cursor-pointer rounded-xl px-4">
				<span data-slot="select-value" class={cn(!feedbackReason && 'text-muted-foreground')}
					>{feedbackReason || 'Tür seç'}</span
				>
			</Select.Trigger>
			<Select.Content class="rounded-xl">
				{#each feedbackReasons as reason (reason)}
					<Select.Item
						value={reason}
						class={cn(
							'cursor-pointer rounded-lg data-highlighted:bg-muted data-highlighted:text-foreground',
							soft
						)}>{reason}</Select.Item
					>
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
		<p class="mb-2 text-sm font-semibold">
			Ekler<span class="ms-1 font-normal text-muted-foreground tabular-nums"
				>({attachments.length}/{maxAttachments})</span
			>
		</p>
		<Empty.Root
			role="button"
			tabindex={0}
			aria-label="Dosya ekle"
			class={cn(
				'cursor-pointer gap-2 rounded-2xl border border-dashed p-4 transition-colors hover:border-foreground/25 hover:bg-muted/50 md:p-4',
				dragging && 'border-foreground/30 bg-muted/50'
			)}
			onclick={() => fileInput?.click()}
			onkeydown={(event) =>
				(event.key === 'Enter' || event.key === ' ') &&
				(event.preventDefault(), fileInput?.click())}
			ondragover={(event) => (event.preventDefault(), (dragging = true))}
			ondragleave={() => (dragging = false)}
			ondrop={(event) => {
				event.preventDefault();
				dragging = false;
				addFiles(event.dataTransfer?.files);
			}}
		>
			<Empty.Header class="gap-2">
				<Empty.Media class="mb-0 rounded-full p-2 text-muted-foreground"
					><UploadIcon class="size-5" /></Empty.Media
				>
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
				event.currentTarget.value = '';
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
								<XIcon class="size-3.5" />
							</Button>
						</Badge>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
	<div class="flex justify-end">
		<Button
			type="submit"
			size="sm"
			class="h-8 cursor-pointer rounded-full px-2.5 font-semibold"
			disabled={!canSend}>Gönder</Button
		>
	</div>
</form>
