export type ListGame = {
	slug: string;
	title: string;
	cover: string | null;
	year: string;
	added: number;
};

export type UserList = {
	id: string;
	kind: 'favorites' | 'play_later' | 'custom';
	name: string;
	description: string;
	visibility: 'private' | 'public';
	// Set on lists made by other people; the viewer's own lists have no owner.
	owner?: string;
	image: string;
	backdrop: string;
	items: ListGame[];
};

type GameSummary = Pick<ListGame, 'slug' | 'title' | 'cover' | 'year'>;

export const toListGame = (game: {
	slug: string;
	title: string;
	cover: string | null;
	releaseDate: string | null;
}): GameSummary => ({
	slug: game.slug,
	title: game.title,
	cover: game.cover,
	year: game.releaseDate?.slice(0, 4) ?? ''
});

// Shared by the game page menu, the list pages and the profile so a change shows up everywhere during the session.
// Not stored yet: lists live in the browser's memory and reset on a full reload.
export const userLists = $state<UserList[]>([
	{
		id: 'favorites',
		kind: 'favorites',
		name: 'Favorites',
		description: 'Some games stay with you. Keep them close.',
		visibility: 'public',
		image: '',
		backdrop: '',
		items: []
	},
	{
		id: 'backlog',
		kind: 'play_later',
		name: 'Play later',
		description: 'The games waiting for their turn.',
		visibility: 'private',
		image: '',
		backdrop: '',
		items: []
	}
]);

export const listHref = (list: Pick<UserList, 'id' | 'kind'>) =>
	list.kind === 'favorites'
		? '/favorites'
		: list.kind === 'play_later'
			? '/backlog'
			: `/lists/${list.id}`;

export const findList = (id: string) => userLists.find((list) => list.id === id);
export const listByKind = (kind: 'favorites' | 'play_later') =>
	userLists.find((list) => list.kind === kind)!;
export const customLists = () => userLists.filter((list) => list.kind === 'custom' && !list.owner);
export const ownLists = () => userLists.filter((list) => !list.owner);
export const listsWithGame = (slug: string) =>
	userLists.filter(
		(list) => (list.owner ? list.visibility === 'public' : true) && hasGame(list, slug)
	);

export const hasGame = (list: UserList, slug: string) =>
	list.items.some((item) => item.slug === slug);

export function setGame(list: UserList, game: GameSummary, on: boolean) {
	if (on === hasGame(list, game.slug)) return;
	list.items = on
		? [{ ...game, added: Date.now() }, ...list.items]
		: list.items.filter((item) => item.slug !== game.slug);
}

export function createList(name: string, game?: GameSummary): UserList {
	const list: UserList = {
		id: `${Date.now().toString(36)}-${name.toLocaleLowerCase('tr').replace(/[^a-z0-9ğüşıöç]+/g, '-')}`,
		kind: 'custom',
		name,
		description: '',
		visibility: 'private',
		image: '',
		backdrop: '',
		items: []
	};
	userLists.push(list);
	const created = userLists[userLists.length - 1];
	if (game) setGame(created, game, true);
	return created;
}

// Library status of a game: one per game, separate from the lists (a game can sit in any number of lists).
// Feeds the profile stats.
export const playStatuses = ['Playing', 'Completed', 'Dropped'] as const;
export type PlayStatus = (typeof playStatuses)[number];
export const gameStatus = $state<Record<string, PlayStatus | undefined>>({});

export const statusOf = (slug: string): PlayStatus | '' => gameStatus[slug] ?? '';

export function setStatus(slug: string, next: PlayStatus | '') {
	gameStatus[slug] = next || undefined;
}

// The viewer's own score (1-10) and review per game. Demo only, like the lists.
export const gameRating = $state<Record<string, number | undefined>>({});
export type MyReview = { text: string; spoiler: boolean };
export const myReviews = $state<Record<string, MyReview | undefined>>({});
