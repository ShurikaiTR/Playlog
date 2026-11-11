import { allGameDetails } from "./games.js";

export type ListGame = {
	slug: string;
	title: string;
	cover: string;
	score: number;
	year: string;
	added: number;
};

export type UserList = {
	id: string;
	kind: "favorites" | "play_later" | "custom";
	name: string;
	description: string;
	visibility: "private" | "public";
	// Set on lists made by other people; the viewer's own lists have no owner.
	owner?: string;
	image: string;
	backdrop: string;
	items: ListGame[];
};

type GameSummary = Pick<ListGame, "slug" | "title" | "cover" | "score" | "year">;

export const toListGame = (game: { slug: string; title: string; cover: string; score: number; releaseDate: string }): GameSummary => ({
	slug: game.slug,
	title: game.title,
	cover: game.cover,
	score: game.score,
	year: game.releaseDate.slice(0, 4),
});

const seed = (indexes: number[], daysAgo = 0): ListGame[] =>
	indexes.map((i, n) => ({ ...toListGame(allGameDetails[i % allGameDetails.length]), added: Date.now() - (daysAgo + n) * 86_400_000 }));

const backdrop = (i: number) => allGameDetails[i % allGameDetails.length].backdrop;

// Shared by the game page menu, the list pages and the profile so a change shows up everywhere during the session.
// Demo only: resets on a full reload.
export const userLists = $state<UserList[]>([
	{
		id: "favoriler",
		kind: "favorites",
		name: "Favoriler",
		description: "Bazı oyunlar seninle kalır. Onları yakında tut.",
		visibility: "public",
		image: "",
		backdrop: backdrop(1),
		items: seed([0, 3, 1], 2),
	},
	{
		id: "planlananlar",
		kind: "play_later",
		name: "Planlananlar",
		description: "Sırasını bekleyen birkaç macera.",
		visibility: "public",
		image: "",
		backdrop: backdrop(2),
		items: seed([2, 4, 5, 1], 1),
	},
	{
		id: "kis-gecesi",
		kind: "custom",
		name: "Kış gecesi oyunları",
		description: "Battaniye, sıcak çay ve uzun bir oturum.",
		visibility: "public",
		owner: "Deniz Aksoy",
		image: "",
		backdrop: backdrop(3),
		items: seed([0, 1, 2, 3], 5),
	},
	{
		id: "hikayesi-guclu-olanlar",
		kind: "custom",
		name: "Hikâyesi güçlü olanlar",
		description: "Bitirince bir süre düşündüren oyunlar.",
		visibility: "public",
		owner: "Elif Karaca",
		image: "",
		backdrop: backdrop(4),
		items: seed([2, 3, 4, 5], 8),
	},
	{
		id: "bir-aksamda-biten",
		kind: "custom",
		name: "Bir akşamda biten oyunlar",
		description: "Kısa ama akılda kalıcı.",
		visibility: "public",
		owner: "Mert Yıldız",
		image: "",
		backdrop: backdrop(5),
		items: seed([4, 5, 0, 1], 12),
	},
]);

export const listHref = (list: Pick<UserList, "id" | "kind">) =>
	list.kind === "favorites" ? "/favoriler" : list.kind === "play_later" ? "/planlananlar" : `/liste/${list.id}`;

export const findList = (id: string) => userLists.find((list) => list.id === id);
export const listByKind = (kind: "favorites" | "play_later") => userLists.find((list) => list.kind === kind)!;
export const customLists = () => userLists.filter((list) => list.kind === "custom" && !list.owner);
export const ownLists = () => userLists.filter((list) => !list.owner);
export const listsWithGame = (slug: string) => userLists.filter((list) => (list.owner ? list.visibility === "public" : true) && hasGame(list, slug));

export const hasGame = (list: UserList, slug: string) => list.items.some((item) => item.slug === slug);

export function setGame(list: UserList, game: GameSummary, on: boolean) {
	if (on === hasGame(list, game.slug)) return;
	list.items = on ? [{ ...game, added: Date.now() }, ...list.items] : list.items.filter((item) => item.slug !== game.slug);
}

export function createList(name: string, game?: GameSummary): UserList {
	const list: UserList = {
		id: `${Date.now().toString(36)}-${name.toLocaleLowerCase("tr").replace(/[^a-z0-9ğüşıöç]+/g, "-")}`,
		kind: "custom",
		name,
		description: "",
		visibility: "private",
		image: "",
		backdrop: backdrop(userLists.length),
		items: [],
	};
	userLists.push(list);
	const created = userLists[userLists.length - 1];
	if (game) setGame(created, game, true);
	return created;
}

// Library status of a game. "Oynamayı planlıyorum" is not stored here: it is membership of the play-later list, so the
// profile shelf and the menu always agree.
export const playStatuses = ["Oynuyorum", "Bitirdim", "Bıraktım", "Ara verdim"] as const;
export type PlayStatus = (typeof playStatuses)[number];
export const gameStatus = $state<Record<string, PlayStatus | undefined>>({});

export const statusOf = (slug: string): PlayStatus | "Oynamayı planlıyorum" | "" =>
	hasGame(listByKind("play_later"), slug) ? "Oynamayı planlıyorum" : (gameStatus[slug] ?? "");

export function setStatus(game: GameSummary, next: string) {
	gameStatus[game.slug] = (playStatuses as readonly string[]).includes(next) ? (next as PlayStatus) : undefined;
	setGame(listByKind("play_later"), game, next === "Oynamayı planlıyorum");
}

// The viewer's own score (1-10) and review per game. Demo only, like the lists.
export const gameRating = $state<Record<string, number | undefined>>({});
export type MyReview = { text: string; spoiler: boolean };
export const myReviews = $state<Record<string, MyReview | undefined>>({});
