import { allGameDetails, trendingGames, upcomingGames, type GameDetail } from '#lib/games.js';
import { allProfiles, type Profile } from '#lib/profiles.js';

// Search over the placeholder catalog. Swap these two functions for API calls once real data exists;
// the dialog only depends on the result shapes below.

export type GameResult = Pick<
	GameDetail,
	'slug' | 'title' | 'cover' | 'backdrop' | 'genres' | 'description' | 'developer'
> & { year: string };

export type UserResult = Pick<Profile, 'name' | 'username' | 'image' | 'bio' | 'cover'>;

const limit = 20;

// Lowercases the Turkish way and drops accents, so "surgun" finds "Sürgün" and "kirik kilic" finds "Kırık Kılıç".
export const fold = (text: string) =>
	text.toLocaleLowerCase('tr').replaceAll('ı', 'i').normalize('NFD').replace(/\p{M}/gu, '');

const words = (text: string) =>
	fold(text)
		.split(/[^\p{L}\p{N}]+/u)
		.filter(Boolean);

// 0: the title starts with the query, 1: a title word starts with it, 2: every query word is found somewhere.
// Returns null when some query word is missing.
function rank(query: string, title: string, extra: string[]): number | null {
	const q = fold(query).trim();
	const terms = words(query);
	if (!terms.length) return null;
	const titleWords = words(title);
	const haystack = [...titleWords, ...extra.flatMap(words)];
	if (!terms.every((term) => haystack.some((word) => word.startsWith(term)))) return null;
	if (fold(title).startsWith(q)) return 0;
	return titleWords.some((word) => word.startsWith(terms[0])) ? 1 : 2;
}

function search<T>(items: T[], query: string, fields: (item: T) => [string, string[]]): T[] {
	return items
		.flatMap((item) => {
			const [title, extra] = fields(item);
			const score = rank(query, title, extra);
			return score === null ? [] : [{ item, score, title }];
		})
		.sort((a, b) => a.score - b.score || a.title.localeCompare(b.title, 'tr'))
		.slice(0, limit)
		.map((entry) => entry.item);
}

const toGame = (game: GameDetail): GameResult => ({
	slug: game.slug,
	title: game.title,
	cover: game.cover,
	backdrop: game.backdrop,
	genres: game.genres,
	description: game.description,
	developer: game.developer,
	year: game.releaseDate.slice(0, 4)
});

const toUser = ({ name, username, image, bio, cover }: Profile): UserResult => ({
	name,
	username,
	image,
	bio,
	cover
});

// Every game once: the detail list, plus any trending or upcoming game that has no detail yet.
const games: GameResult[] = (() => {
	const bySlug = new Map(allGameDetails.map((game) => [game.slug, toGame(game)]));
	for (const game of [...trendingGames, ...upcomingGames]) {
		if (!bySlug.has(game.slug)) {
			bySlug.set(game.slug, {
				slug: game.slug,
				title: game.title,
				cover: game.cover,
				backdrop: game.cover,
				genres: game.meta ? [game.meta] : [],
				description: '',
				developer: '',
				year: ''
			});
		}
	}
	return [...bySlug.values()];
})();

const users: UserResult[] = allProfiles.map(toUser);

export const searchGames = (query: string) =>
	search(games, query, (game) => [game.title, [...game.genres, game.developer]]);

export const searchUsers = (query: string) =>
	search(users, query, (user) => [user.name, [user.username]]);

// Shown before anything is typed.
export const suggestedGames = (): GameResult[] =>
	trendingGames
		.slice(0, 5)
		.flatMap((trending) => games.find((game) => game.slug === trending.slug) ?? []);

export const suggestedUsers = (): UserResult[] => users.slice(0, 5);
