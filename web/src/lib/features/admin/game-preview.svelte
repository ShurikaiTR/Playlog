<script lang="ts">
	import * as Field from '#lib/components/ui/field/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import ImageOffIcon from '@lucide/svelte/icons/image-off';
	import type { GameFormValues } from './game-form.js';

	// The artwork slots, top to bottom: cover (poster), backdrop, logo. Each shows its image and
	// takes the address, so what is typed is what is seen.
	let { values = $bindable() }: { values: GameFormValues } = $props();

	// An address that fails to load is shown as missing instead of as a broken image.
	let failed = $state<string[]>([]);
	const usable = (url: string) =>
		url.startsWith('https://') && !failed.includes(url) ? url : null;
	const fail = (url: string) => (failed = [...failed, url]);

	const cover = $derived(usable(values.coverUrl));
	const backdrop = $derived(usable(values.backdropUrl));
	const logo = $derived(usable(values.logoUrl));
</script>

{#snippet slot(
	name: 'coverUrl' | 'backdropUrl' | 'logoUrl',
	title: string,
	hint: string,
	url: string | null,
	box: string
)}
	<Field.Field>
		<Field.Label for={name}>{title}</Field.Label>
		<label
			for={name}
			class="relative flex cursor-text items-center justify-center overflow-hidden rounded-xl bg-muted ring-1 ring-border transition-shadow hover:ring-foreground/30 {box}"
		>
			{#if url}
				<img
					src={url}
					alt=""
					class="size-full {name === 'logoUrl' ? 'object-contain p-4' : 'object-cover'}"
					onerror={() => fail(url)}
					decoding="async"
				/>
			{:else}
				<span
					class="flex flex-col items-center gap-1.5 px-3 text-center text-footnote text-muted-foreground"
				>
					<ImageOffIcon class="size-5" aria-hidden="true" />
					{values[name] ? 'Could not load this image' : hint}
				</span>
			{/if}
		</label>
		<Input
			id={name}
			{name}
			type="url"
			bind:value={values[name]}
			class="rounded-xl"
			placeholder="https://…"
			autocomplete="off"
			aria-invalid={values[name] !== '' && !values[name].startsWith('https://') ? true : undefined}
		/>
	</Field.Field>
{/snippet}

<div class="grid gap-5 rounded-2xl bg-card p-4 ring-1 ring-foreground/10">
	<p class="text-footnote font-medium tracking-caps text-muted-foreground uppercase">Artwork</p>
	{@render slot('coverUrl', 'Cover', 'Poster, 2:3', cover, 'mx-auto aspect-2/3 w-48')}
	{@render slot(
		'backdropUrl',
		'Backdrop',
		'Wide header image, 16:9',
		backdrop,
		'aspect-video w-full'
	)}
	{@render slot('logoUrl', 'Logo', 'Transparent PNG or SVG', logo, 'h-24 w-full')}
</div>
