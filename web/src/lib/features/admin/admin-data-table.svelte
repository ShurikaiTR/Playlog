<script lang="ts" generics="TData, TValue">
	import { getCoreRowModel, type ColumnDef } from '@tanstack/table-core';
	import { createSvelteTable, FlexRender } from '#lib/components/ui/data-table/index.js';
	import * as Table from '#lib/components/ui/table/index.js';

	let {
		columns,
		data,
		empty = 'Nothing here yet.'
	}: { columns: ColumnDef<TData, TValue>[]; data: TData[]; empty?: string } = $props();

	const table = createSvelteTable({
		get data() {
			return data;
		},
		get columns() {
			return columns;
		},
		getCoreRowModel: getCoreRowModel()
	});
</script>

<div class="overflow-hidden rounded-2xl ring-1 ring-border">
	<Table.Root>
		<Table.Header>
			{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
				<Table.Row class="hover:bg-transparent">
					{#each headerGroup.headers as header (header.id)}
						<Table.Head class="h-11 text-footnote font-medium text-muted-foreground">
							{#if !header.isPlaceholder}
								<FlexRender
									content={header.column.columnDef.header}
									context={header.getContext()}
								/>
							{/if}
						</Table.Head>
					{/each}
				</Table.Row>
			{/each}
		</Table.Header>
		<Table.Body>
			{#each table.getRowModel().rows as row (row.id)}
				<Table.Row>
					{#each row.getVisibleCells() as cell (cell.id)}
						<Table.Cell class="text-callout pointer-coarse:py-3">
							<FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
						</Table.Cell>
					{/each}
				</Table.Row>
			{:else}
				<Table.Row class="hover:bg-transparent">
					<Table.Cell colspan={columns.length} class="h-32 text-center text-muted-foreground">
						{empty}
					</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
</div>
