import type { RequestEvent } from '@sveltejs/kit';
import {
	ageRating,
	formatRelease,
	type DatePrecision,
	type Game,
	type GameDetail,
	type GameSummary,
	type Named,
	type SystemRequirements
} from '#lib/features/games/games.js';
import { api } from './api.js';

// The Rust API's catalog shapes, and their translation to what the pages render.

type Event = Pick<RequestEvent, 'request' | 'getClientAddress'>;

type ApiCard = {
	slug: string;
	title: string;
	kind: string;
	summary: string | null;
	cover_url: string | null;
	backdrop_url: string | null;
	logo_url: string | null;
	release_date: string | null;
	release_precision: DatePrecision;
	genres: string[];
	developers: string[];
};

type ApiRef = { slug: string; title: string; kind: string; cover_url: string | null };

type ApiDetail = Omit<ApiCard, 'genres' | 'developers'> & {
	description: string | null;
	time_to_beat_minutes: number | null;
	pc_requirements: SystemRequirements | null;
	parent: ApiRef | null;
	series: Named | null;
	releases: { platform: { slug: string; name: string; family: string } }[];
	genres: Named[];
	themes: Named[];
	modes: Named[];
	developers: Named[];
	publishers: Named[];
	screenshots: string[];
	trailers: { provider: string; video_id: string }[];
	languages: { language: string; interface: boolean; audio: boolean; subtitles: boolean }[];
	age_ratings: Record<string, string>;
	related: ApiRef[];
	series_games: ApiRef[];
	similar: ApiRef[];
};

const toSummary = (card: ApiCard): GameSummary => ({
	slug: card.slug,
	title: card.title,
	cover: card.cover_url,
	backdrop: card.backdrop_url,
	logo: card.logo_url,
	summary: card.summary,
	genres: card.genres,
	developers: card.developers,
	releaseDate: card.release_date,
	releasePrecision: card.release_precision
});

const toGame = (ref: ApiRef): Game => ({ slug: ref.slug, title: ref.title, cover: ref.cover_url });

const languageNames = new Intl.DisplayNames(['en'], { type: 'language' });

function languageName(code: string): string {
	try {
		return languageNames.of(code) ?? code;
	} catch {
		// Not a code Intl knows; show it as it is.
		return code;
	}
}

export type GameQuery = {
	q?: string;
	genres?: string[];
	platforms?: string[];
	modes?: string[];
	yearMin?: number;
	yearMax?: number;
	window?: 'released' | 'upcoming';
	sort?: 'newest' | 'soonest' | 'title' | 'added';
	limit?: number;
	offset?: number;
};

/**
 * A page of published games. When the API cannot be reached the list is empty rather than an
 * error, so a page still renders its frame; `ok` tells the two apart.
 */
export async function listGames(
	event: Event,
	query: GameQuery = {}
): Promise<{ ok: boolean; total: number; games: GameSummary[] }> {
	const params = new URLSearchParams();
	if (query.q) params.set('q', query.q);
	if (query.genres?.length) params.set('genres', query.genres.join(','));
	if (query.platforms?.length) params.set('platforms', query.platforms.join(','));
	if (query.modes?.length) params.set('modes', query.modes.join(','));
	if (query.yearMin !== undefined) params.set('year_min', String(query.yearMin));
	if (query.yearMax !== undefined) params.set('year_max', String(query.yearMax));
	if (query.window) params.set('window', query.window);
	if (query.sort) params.set('sort', query.sort);
	if (query.limit !== undefined) params.set('limit', String(query.limit));
	if (query.offset !== undefined) params.set('offset', String(query.offset));

	const search = params.toString();
	const result = await api<{ total: number; items: ApiCard[] }>(
		event,
		`/v1/games${search ? `?${search}` : ''}`,
		{ method: 'GET' }
	);
	if (!result.ok) return { ok: false, total: 0, games: [] };
	return { ok: true, total: result.data.total, games: result.data.items.map(toSummary) };
}

/**
 * One game with everything its page shows. `null` when there is no such published game. The
 * returned slug is the current one: an old slug still finds the game, and the page redirects.
 */
export async function getGame(
	event: Event,
	slug: string
): Promise<{ status: 'found'; game: GameDetail } | { status: 'missing' | 'unavailable' }> {
	const result = await api<ApiDetail>(event, `/v1/games/${encodeURIComponent(slug)}`, {
		method: 'GET'
	});
	if (!result.ok) return { status: result.status === 404 ? 'missing' : 'unavailable' };
	const data = result.data;

	// A game is listed once per platform even when it has several regional releases.
	const platforms = [
		...new Map(data.releases.map((release) => [release.platform.slug, release.platform])).values()
	];
	const trailer = data.trailers.find((trailer) => trailer.provider === 'youtube');

	return {
		status: 'found',
		game: {
			slug: data.slug,
			title: data.title,
			kind: data.kind,
			cover: data.cover_url,
			backdrop: data.backdrop_url,
			logo: data.logo_url,
			summary: data.summary,
			description: data.description,
			genres: data.genres,
			themes: data.themes,
			modes: data.modes,
			developers: data.developers,
			publishers: data.publishers,
			releaseDate: data.release_date,
			releasePrecision: data.release_precision,
			platforms,
			languages: data.languages
				.map((language) => ({
					code: language.language,
					name: languageName(language.language),
					interface: language.interface,
					audio: language.audio,
					subtitles: language.subtitles
				}))
				.sort((a, b) => a.name.localeCompare(b.name, 'en')),
			ageRating: ageRating(data.age_ratings),
			playtime: data.time_to_beat_minutes ? Math.round(data.time_to_beat_minutes / 60) : null,
			requirements: data.pc_requirements,
			screenshots: data.screenshots,
			trailer: trailer?.video_id ?? null,
			series: data.series,
			parent: data.parent ? toGame(data.parent) : null,
			related: data.related.map(toGame),
			seriesGames: data.series_games.map(toGame),
			similar: data.similar.map(toGame)
		}
	};
}

export type TagCount = Named & { count: number };

/** Genres (or another tag kind) with how many published games carry each, most common first. */
export async function listTags(
	event: Event,
	kind: 'genre' | 'theme' | 'mode' = 'genre'
): Promise<TagCount[]> {
	const result = await api<TagCount[]>(event, `/v1/tags?kind=${kind}`, { method: 'GET' });
	return result.ok ? result.data : [];
}

/** The small line under a card: the first genre and the year, whichever are known. */
export const cardMeta = (game: GameSummary): string =>
	[game.genres[0], game.releaseDate?.slice(0, 4)].filter(Boolean).join(' · ');

/** For upcoming games the date is the point. */
export const releaseMeta = (game: GameSummary): string =>
	formatRelease(game.releaseDate, game.releasePrecision);
