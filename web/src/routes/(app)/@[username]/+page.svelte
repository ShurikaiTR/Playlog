<script lang="ts">
	import ListCard from '#lib/features/lists/list-card.svelte';
	import * as Avatar from '#lib/components/ui/avatar/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Item from '#lib/components/ui/item/index.js';
	import { ScrollArea } from '#lib/components/ui/scroll-area/index.js';
	import { listHref, ownLists } from '#lib/features/lists/user-lists.svelte.js';

	let { data } = $props();
	const profile = $derived(data.profile);

	const initial = (name: string) => name.charAt(0).toLocaleUpperCase('en');
	let following = $state(false);

	const connections = $derived([
		{
			key: 'following',
			label: 'Following',
			description: data.isOwner ? 'People you follow.' : `People @${profile.username} follows.`,
			empty: data.isOwner ? "You aren't following anyone yet." : "Isn't following anyone yet.",
			people: profile.following
		},
		{
			key: 'followers',
			label: 'Followers',
			description: data.isOwner
				? 'People who follow you.'
				: `People who follow @${profile.username}.`,
			empty: data.isOwner ? "You don't have any followers yet." : 'No followers yet.',
			people: profile.followers
		}
	] as const);

	// The owner sees their live lists (same data as the list pages); visitors see the profile's sample lists.
	const lists = $derived(
		data.isOwner
			? ownLists()
					// Only the two default lists show on the profile; the rest live in the library.
					.filter((list) => list.kind !== 'custom')
					.map((list) => ({
						id: list.id,
						kind: list.kind === 'play_later' ? ('play_later' as const) : list.kind,
						name: list.name,
						description: list.description || undefined,
						cover: list.backdrop,
						gameCount: list.items.length,
						href: listHref(list) as string | null
					}))
			: profile.lists.map((list) => ({ ...list, href: null as string | null }))
	);
</script>

<svelte:head>
	<title>{profile.name} | Playthru</title>
</svelte:head>

<div lang="en" class="flex min-w-0 flex-col gap-14 pb-16">
	<section aria-labelledby="profile-name" class="profile-hero relative isolate min-w-0">
		<!-- Full-bleed: slides under the header and the floating sidebar. -->
		<div
			class="pointer-events-none absolute inset-0 -z-10 overflow-hidden"
			aria-hidden="true"
			style="mask-image: linear-gradient(to bottom, black 55%, transparent 100%)"
		>
			{#if profile.cover}
				<img
					src={profile.cover}
					alt=""
					width="1600"
					height="900"
					fetchpriority="high"
					class="size-full object-cover object-[center_45%]"
				/>
			{/if}
			<div class="profile-scrim absolute inset-0"></div>
		</div>

		<div
			class="flex min-w-0 flex-col gap-6 px-4 pt-36 pb-2 sm:flex-row sm:items-end sm:justify-between md:px-8 md:pt-44"
		>
			<div class="flex min-w-0 items-end gap-4 sm:gap-6">
				<Avatar.Root class="size-24 shrink-0 shadow-sm ring-4 ring-background sm:size-32">
					{#if profile.image}<Avatar.Image src={profile.image} alt="" />{/if}
					<Avatar.Fallback class="bg-foreground text-3xl font-medium text-background sm:text-5xl"
						>{initial(profile.name)}</Avatar.Fallback
					>
				</Avatar.Root>
				<div class="min-w-0 pb-1">
					<h1
						id="profile-name"
						class="truncate text-[2rem] leading-tight font-medium tracking-[-0.025em] text-primary md:text-[2.5rem]"
					>
						{profile.name}
					</h1>
					<p class="mt-0.5 text-sm text-muted-foreground">@{profile.username}</p>
					<p class="mt-3 max-w-xl text-sm leading-6 text-muted-foreground">
						{profile.bio ?? 'No bio yet.'}
					</p>
				</div>
			</div>

			<div class="flex shrink-0 flex-wrap items-center gap-2 pb-1">
				{#each connections as connection (connection.key)}
					<Dialog.Root>
						<Dialog.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									variant="outline"
									class="h-auto cursor-pointer flex-col items-start gap-0 rounded-xl bg-background/70 px-4 py-2 text-left backdrop-blur-sm"
								>
									<span class="text-lg leading-tight font-semibold tabular-nums"
										>{connection.people.length}</span
									>
									<span class="text-xs font-normal text-muted-foreground">{connection.label}</span>
								</Button>
							{/snippet}
						</Dialog.Trigger>
						<Dialog.Content>
							<Dialog.Header class="text-left">
								<Dialog.Title>
									{connection.label}
									<span class="ml-1 text-sm text-muted-foreground">{connection.people.length}</span>
								</Dialog.Title>
								<Dialog.Description>{connection.description}</Dialog.Description>
							</Dialog.Header>
							<ScrollArea class="[&>[data-slot=scroll-area-viewport]]:max-h-[min(24rem,60dvh)]">
								{#if connection.people.length}
									<Item.Group class="gap-0">
										{#each connection.people as person, index (person.username)}
											{#if index > 0}<Item.Separator class="my-0" />{/if}
											{#snippet row()}
												<Item.Media>
													<Avatar.Root class="size-10">
														{#if person.image}<Avatar.Image src={person.image} alt="" />{/if}
														<Avatar.Fallback
															class="bg-foreground text-xs font-semibold text-background"
															>{initial(person.name)}</Avatar.Fallback
														>
													</Avatar.Root>
												</Item.Media>
												<Item.Content>
													<Item.Title>{person.name}</Item.Title>
													<Item.Description>@{person.username}</Item.Description>
												</Item.Content>
											{/snippet}
											{#if data.linkable.includes(person.username)}
												<Item.Root size="xs">
													{#snippet child({ props })}
														<a {...props} href="/@{person.username}">{@render row()}</a>
													{/snippet}
												</Item.Root>
											{:else}
												<Item.Root size="xs">{@render row()}</Item.Root>
											{/if}
										{/each}
									</Item.Group>
								{:else}
									<p class="py-4 text-sm text-muted-foreground">{connection.empty}</p>
								{/if}
							</ScrollArea>
						</Dialog.Content>
					</Dialog.Root>
				{/each}

				{#if data.isOwner}
					<Button
						variant="outline"
						class="h-10 cursor-pointer rounded-full bg-background/70 px-5 backdrop-blur-sm"
						>Edit profile</Button
					>
				{:else}
					<Button
						variant={following ? 'outline' : 'default'}
						class="h-10 cursor-pointer rounded-full px-5"
						onclick={() => (following = !following)}
					>
						{following ? 'Unfollow' : 'Follow'}
					</Button>
				{/if}
			</div>
		</div>
	</section>

	<section class="min-w-0 space-y-6 px-4 md:px-8" aria-labelledby="profile-lists-title">
		<div>
			<h2 id="profile-lists-title" class="truncate text-2xl font-medium text-primary">
				Game lists
			</h2>
			<p class="mt-1 text-sm text-muted-foreground">
				Games you've played and games waiting for their turn.
			</p>
		</div>

		{#if lists.length === 0}
			<p class="text-sm text-muted-foreground">No public lists.</p>
		{:else}
			<div class="grid gap-4 sm:grid-cols-2 xl:max-w-3xl">
				{#each lists as list (list.id)}
					<ListCard
						name={list.name}
						kind={list.kind}
						cover={list.cover}
						gameCount={list.gameCount}
						description={list.description}
						href={list.href}
					/>
				{/each}
			</div>
		{/if}
	</section>
</div>

<style>
	.profile-hero {
		--offset: var(--sidebar-offset, 0px);
		margin-top: -4rem;
		margin-left: calc(-1 * var(--offset));
		padding-left: var(--offset);
	}

	.profile-scrim {
		background:
			linear-gradient(
				0deg,
				var(--background) 0%,
				color-mix(in oklch, var(--background) 70%, transparent) 35%,
				transparent 75%
			),
			linear-gradient(
				90deg,
				color-mix(in oklch, var(--background) 70%, transparent) 0%,
				color-mix(in oklch, var(--background) 30%, transparent) 60%,
				color-mix(in oklch, var(--background) 20%, transparent) 100%
			),
			linear-gradient(
				180deg,
				color-mix(in oklch, var(--background) 55%, transparent) 0%,
				transparent 22%
			);
	}
</style>
