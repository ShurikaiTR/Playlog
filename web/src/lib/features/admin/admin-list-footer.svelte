<script lang="ts">
	import * as Pagination from '#lib/components/ui/pagination/index.js';

	// Server-side paging: the page number lives in the address, so a page can be shared and reloaded.
	let {
		total,
		perPage,
		page,
		onPage
	}: { total: number; perPage: number; page: number; onPage: (page: number) => void } = $props();

	const number = new Intl.NumberFormat('en');
	const first = $derived(total === 0 ? 0 : (page - 1) * perPage + 1);
	const last = $derived(Math.min(page * perPage, total));
</script>

<div class="mt-4 flex flex-col items-center justify-between gap-3 sm:flex-row">
	<p class="text-footnote text-muted-foreground numeric">
		{number.format(first)}–{number.format(last)} of {number.format(total)}
	</p>
	{#if total > perPage}
		<Pagination.Root count={total} {perPage} {page} onPageChange={onPage} class="mx-0 w-auto">
			{#snippet children({ pages, currentPage })}
				<Pagination.Content>
					<Pagination.Item><Pagination.PrevButton /></Pagination.Item>
					{#each pages as item (item.key)}
						{#if item.type === 'ellipsis'}
							<Pagination.Item><Pagination.Ellipsis /></Pagination.Item>
						{:else}
							<Pagination.Item>
								<Pagination.Link page={item} isActive={currentPage === item.value} />
							</Pagination.Item>
						{/if}
					{/each}
					<Pagination.Item><Pagination.NextButton /></Pagination.Item>
				</Pagination.Content>
			{/snippet}
		</Pagination.Root>
	{/if}
</div>
