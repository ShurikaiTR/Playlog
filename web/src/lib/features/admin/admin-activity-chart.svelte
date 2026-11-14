<script lang="ts">
	import { scaleUtc } from 'd3-scale';
	import { curveNatural } from 'd3-shape';
	import { Area, AreaChart } from 'layerchart';
	import * as Card from '#lib/components/ui/card/index.js';
	import * as Chart from '#lib/components/ui/chart/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import * as ToggleGroup from '#lib/components/ui/toggle-group/index.js';
	import type { AdminStats } from '#lib/server/admin.js';

	let { activity }: { activity: AdminStats['activity'] } = $props();

	let range = $state('90d');
	const ranges: Record<string, { label: string; days: number }> = {
		'90d': { label: 'Last 3 months', days: 90 },
		'30d': { label: 'Last 30 days', days: 30 },
		'7d': { label: 'Last 7 days', days: 7 }
	};

	// Days arrive as plain dates; the chart wants them as UTC midnights.
	const days = $derived(
		activity.slice(-ranges[range].days).map((day) => ({
			date: new Date(`${day.date}T00:00:00Z`),
			users: day.users,
			games: day.games
		}))
	);

	const chartConfig = {
		users: { label: 'New users', color: 'var(--primary)' },
		games: { label: 'New games', color: 'var(--chart-2)' }
	} satisfies Chart.ChartConfig;

	const dayLabel = (date: Date) =>
		date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', timeZone: 'UTC' });
</script>

<Card.Root class="@container/card">
	<Card.Header>
		<Card.Title>Growth</Card.Title>
		<Card.Description>New users and new games per day</Card.Description>
		<Card.Action>
			<ToggleGroup.Root
				type="single"
				value={range}
				onValueChange={(value) => value && (range = value)}
				variant="outline"
				class="hidden *:data-[slot=toggle-group-item]:!px-4 @[767px]/card:flex"
			>
				{#each Object.entries(ranges) as [value, { label }] (value)}
					<ToggleGroup.Item {value}>{label}</ToggleGroup.Item>
				{/each}
			</ToggleGroup.Root>
			<Select.Root type="single" bind:value={range}>
				<Select.Trigger size="sm" class="flex w-40 @[767px]/card:hidden" aria-label="Time range">
					{ranges[range].label}
				</Select.Trigger>
				<Select.Content>
					{#each Object.entries(ranges) as [value, { label }] (value)}
						<Select.Item {value} {label} />
					{/each}
				</Select.Content>
			</Select.Root>
		</Card.Action>
	</Card.Header>
	<Card.Content class="px-2 pt-4 sm:px-6 sm:pt-6">
		<Chart.Container config={chartConfig} class="aspect-auto h-[250px] w-full">
			<AreaChart
				legend
				data={days}
				x="date"
				xScale={scaleUtc()}
				series={[
					{ key: 'games', label: chartConfig.games.label, color: chartConfig.games.color },
					{ key: 'users', label: chartConfig.users.label, color: chartConfig.users.color }
				]}
				seriesLayout="stack"
				props={{
					xAxis: {
						ticks: range === '7d' ? 7 : undefined,
						format: (value: Date) => dayLabel(value)
					},
					yAxis: { format: (value: number) => (Number.isInteger(value) ? String(value) : '') }
				}}
			>
				{#snippet marks({ context })}
					<defs>
						<linearGradient id="fillUsers" x1="0" y1="0" x2="0" y2="1">
							<stop offset="5%" stop-color="var(--color-users)" stop-opacity={1.0} />
							<stop offset="95%" stop-color="var(--color-users)" stop-opacity={0.1} />
						</linearGradient>
						<linearGradient id="fillGames" x1="0" y1="0" x2="0" y2="1">
							<stop offset="5%" stop-color="var(--color-games)" stop-opacity={0.8} />
							<stop offset="95%" stop-color="var(--color-games)" stop-opacity={0.1} />
						</linearGradient>
					</defs>
					{#each context.series.visibleSeries as s (s.key)}
						<Area
							seriesKey={s.key}
							curve={curveNatural}
							fillOpacity={0.4}
							line={{ class: 'stroke-1' }}
							motion="tween"
							{...s.props}
							fill={s.key === 'users' ? 'url(#fillUsers)' : 'url(#fillGames)'}
						/>
					{/each}
				{/snippet}
				{#snippet tooltip()}
					<Chart.Tooltip labelFormatter={(value: Date) => dayLabel(value)} indicator="line" />
				{/snippet}
			</AreaChart>
		</Chart.Container>
	</Card.Content>
</Card.Root>
