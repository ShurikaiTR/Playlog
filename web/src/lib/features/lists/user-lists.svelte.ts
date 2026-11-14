import { invalidate } from '$app/navigation';
import { failToast, promptSignIn, socialRequest } from './social-api.js';

export type ListGame = {
	slug: string;
	title: string;
	cover: string | null;
	year: string;
	added: number;
};

export type UserList = {
	id: string;
	kind: 'played' | 'play_later' | 'custom';
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

const defaultLists = (): UserList[] => [
	{
		id: 'played',
		kind: 'played',
		name: 'Played',
		description: 'The games you have played.',
		visibility: 'public',
		image: '',
		backdrop: '',
		items: []
	},
	{
		id: 'play-later',
		kind: 'play_later',
		name: 'Play later',
		description: 'The games waiting for their turn.',
		visibility: 'private',
		image: '',
		backdrop: '',
		items: []
	}
];

// Shared by the game page menu, the list pages and the profile so a change shows up everywhere.
// The server sends the member's lists once per visit (`hydrateLists`); changes are applied here
// right away and sent to the API, then undone if the API refuses them. This lives in the browser
// only: it is never filled during server rendering, so one visitor's lists can't reach another.
export const userLists = $state<UserList[]>(defaultLists());
const session = $state({ signedIn: false });

export function hydrateLists(lists: UserList[] | null) {
	session.signedIn = lists !== null;
	// The cover and image are not stored yet; keep what was picked during this visit.
	const local = (id: string) => userLists.find((list) => list.id === id);
	userLists.splice(
		0,
		userLists.length,
		...(lists ?? defaultLists()).map((list) => ({
			...list,
			image: local(list.id)?.image ?? '',
			backdrop: local(list.id)?.backdrop ?? ''
		}))
	);
}

export const listHref = (list: Pick<UserList, 'id' | 'kind'>) =>
	list.kind === 'played'
		? '/played'
		: list.kind === 'play_later'
			? '/play-later'
			: `/lists/${list.id}`;

export const findList = (id: string) => userLists.find((list) => list.id === id);
export const listByKind = (kind: 'played' | 'play_later') =>
	userLists.find((list) => list.kind === kind)!;
export const customLists = () => userLists.filter((list) => list.kind === 'custom' && !list.owner);
export const ownLists = () => userLists.filter((list) => !list.owner);
export const listsWithGame = (slug: string) =>
	userLists.filter(
		(list) => (list.owner ? list.visibility === 'public' : true) && hasGame(list, slug)
	);

export const hasGame = (list: UserList, slug: string) =>
	list.items.some((item) => item.slug === slug);

const withGame = (items: ListGame[], game: GameSummary) =>
	items.some((item) => item.slug === game.slug)
		? items
		: [{ ...game, added: Date.now() }, ...items];
const withoutGame = (items: ListGame[], slug: string) => items.filter((item) => item.slug !== slug);

/** Adds or removes a game. Resolves to false when it was refused and has been undone. */
export async function setGame(list: UserList, game: GameSummary, on: boolean): Promise<boolean> {
	if (on === hasGame(list, game.slug)) return true;
	if (!session.signedIn) {
		promptSignIn('save games to your library');
		return false;
	}
	list.items = on ? withGame(list.items, game) : withoutGame(list.items, game.slug);
	const result = await socialRequest(
		on ? 'PUT' : 'DELETE',
		`me/lists/${list.id}/games/${game.slug}`
	);
	if (!result.ok) {
		list.items = on ? withoutGame(list.items, game.slug) : withGame(list.items, game);
		failToast(result.error);
		return false;
	}
	void invalidate('app:social');
	return true;
}

export async function createList(name: string, game?: GameSummary): Promise<UserList | null> {
	if (!session.signedIn) {
		promptSignIn('create lists');
		return null;
	}
	const result = await socialRequest<{ id: string }>('POST', 'me/lists', {
		name,
		game_slug: game?.slug
	});
	if (!result.ok) {
		failToast(result.error);
		return null;
	}
	userLists.push({
		id: result.data.id,
		kind: 'custom',
		name,
		description: '',
		visibility: 'private',
		image: '',
		backdrop: '',
		items: game ? [{ ...game, added: Date.now() }] : []
	});
	if (game) void invalidate('app:social');
	return userLists[userLists.length - 1];
}

/** Saves a list's name, description and visibility. System lists keep their name. */
export async function saveListDetails(
	list: UserList,
	details: Pick<UserList, 'name' | 'description' | 'visibility'>
): Promise<boolean> {
	const before = { name: list.name, description: list.description, visibility: list.visibility };
	const name = list.kind === 'custom' ? details.name : list.name;
	Object.assign(list, { ...details, name });
	const result = await socialRequest('PATCH', `me/lists/${list.id}`, {
		name: list.kind === 'custom' ? name : undefined,
		description: details.description,
		visibility: details.visibility
	});
	if (!result.ok) {
		Object.assign(list, before);
		failToast(result.error);
		return false;
	}
	return true;
}
