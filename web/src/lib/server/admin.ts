import { error, type RequestEvent } from '@sveltejs/kit';
import { hasRole } from '#lib/features/auth/roles.js';
import type { ApiUser } from './api.js';
import { api, type ApiResult } from './api.js';
import type { Role } from '#lib/features/auth/roles.js';
import type { GameFormValues } from '#lib/features/admin/game-form.js';

type Event = Pick<RequestEvent, 'request' | 'getClientAddress' | 'locals'>;

/**
 * Staff gate for every /admin page and action. Anyone else gets a plain 404, so the area does not
 * reveal that it exists. The API checks the role again on each call; this keeps the pages away.
 */
export function requireStaff(
	locals: App.Locals,
	min: 'moderator' | 'admin' = 'moderator'
): { user: ApiUser; token: string } {
	const { user, token } = locals;
	if (!user || !token || !hasRole(user.role, min)) error(404, 'Not found');
	return { user, token };
}

/** A call to `/v1/admin/...` with the staff member's own session. */
export function adminApi<T>(
	event: Event,
	path: string,
	options: { method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'; body?: unknown } = {}
): Promise<ApiResult<T>> {
	return api<T>(event, `/v1/admin${path}`, {
		method: options.method ?? 'GET',
		body: options.body,
		token: event.locals.token
	});
}

export type AdminGameRow = {
	id: string;
	slug: string;
	title: string;
	kind: string;
	status: 'draft' | 'published' | 'hidden';
	release_date: string | null;
	release_precision: string;
	cover_url: string | null;
	updated_at: string;
};

export type AdminUserRow = {
	id: string;
	username: string;
	display_name: string | null;
	email: string | null;
	role: Role;
	created_at: string;
};

export type AdminStats = {
	games: number;
	games_published: number;
	games_draft: number;
	users: number;
	reviews: number;
	lists: number;
};

export type PlatformOption = { slug: string; name: string; family: string };

type Named = { slug: string; name: string };

/** The part of the API's game detail the editor needs (preview shows drafts too). */
export type AdminGameDetail = {
	id: string;
	slug: string;
	title: string;
	kind: string;
	status: string;
	summary: string | null;
	description: string | null;
	release_date: string | null;
	release_precision: string;
	cover_url: string | null;
	backdrop_url: string | null;
	logo_url: string | null;
	time_to_beat_minutes: number | null;
	pc_requirements: unknown;
	parent: { slug: string } | null;
	series: Named | null;
	releases: {
		platform: { slug: string };
		region: string;
		release_date: string | null;
		release_precision: string;
	}[];
	genres: Named[];
	themes: Named[];
	modes: Named[];
	perspectives: Named[];
	developers: Named[];
	publishers: Named[];
	screenshots: string[];
	artworks: string[];
	trailers: { provider: string; video_id: string }[];
	languages: { language: string; interface: boolean; audio: boolean; subtitles: boolean }[];
	age_ratings: Record<string, string>;
	alt_titles: string[];
};

const names = (items: Named[]) => items.map((item) => item.name).join(', ');

export function valuesFromDetail(game: AdminGameDetail): GameFormValues {
	return {
		title: game.title,
		slug: game.slug,
		kind: game.kind,
		status: game.status,
		summary: game.summary ?? '',
		description: game.description ?? '',
		releaseDate: game.release_date ?? '',
		releasePrecision: game.release_precision === 'tba' ? 'day' : game.release_precision,
		coverUrl: game.cover_url ?? '',
		backdropUrl: game.backdrop_url ?? '',
		logoUrl: game.logo_url ?? '',
		timeToBeat: game.time_to_beat_minutes ? String(game.time_to_beat_minutes) : '',
		series: game.series?.name ?? '',
		parentSlug: game.parent?.slug ?? '',
		genres: names(game.genres),
		themes: names(game.themes),
		modes: names(game.modes),
		perspectives: names(game.perspectives),
		developers: names(game.developers),
		publishers: names(game.publishers),
		altTitles: game.alt_titles.join(', '),
		languages: game.languages.map((language) => language.language).join(', '),
		platforms: [...new Set(game.releases.map((release) => release.platform.slug))]
	};
}

const text = (form: FormData, key: string) => {
	const value = form.get(key);
	return typeof value === 'string' ? value.trim() : '';
};

export function valuesFromForm(form: FormData): GameFormValues {
	return {
		title: text(form, 'title'),
		slug: text(form, 'slug'),
		kind: text(form, 'kind'),
		status: text(form, 'status'),
		summary: text(form, 'summary'),
		description: text(form, 'description'),
		releaseDate: text(form, 'releaseDate'),
		releasePrecision: text(form, 'releasePrecision'),
		coverUrl: text(form, 'coverUrl'),
		backdropUrl: text(form, 'backdropUrl'),
		logoUrl: text(form, 'logoUrl'),
		timeToBeat: text(form, 'timeToBeat'),
		series: text(form, 'series'),
		parentSlug: text(form, 'parentSlug'),
		genres: text(form, 'genres'),
		themes: text(form, 'themes'),
		modes: text(form, 'modes'),
		perspectives: text(form, 'perspectives'),
		developers: text(form, 'developers'),
		publishers: text(form, 'publishers'),
		altTitles: text(form, 'altTitles'),
		languages: text(form, 'languages'),
		platforms: form.getAll('platforms').filter((value) => typeof value === 'string')
	};
}

const list = (value: string): string[] => [
	...new Set(
		value
			.split(',')
			.map((item) => item.trim())
			.filter(Boolean)
	)
];
const orNull = (value: string) => value || null;

/**
 * The API's game input for a form post. The edit replaces everything the API stores for the game,
 * so what the form does not show is taken from the stored game: media, trailers, age ratings,
 * system requirements, and the regional releases and language flags of what stays selected.
 */
export function buildGameInput(values: GameFormValues, stored: AdminGameDetail | null) {
	const date = orNull(values.releaseDate);
	const precision = date ? values.releasePrecision || 'day' : null;

	const releases = (stored?.releases ?? [])
		.filter((release) => values.platforms.includes(release.platform.slug))
		.map((release) => ({
			platform: release.platform.slug,
			region: release.region,
			date: release.release_date,
			precision: release.release_date ? release.release_precision : null
		}));
	const present = new Set(releases.map((release) => release.platform));
	for (const platform of values.platforms) {
		if (!present.has(platform)) {
			present.add(platform);
			releases.push({ platform, region: 'worldwide', date, precision });
		}
	}

	const storedLanguages = new Map(
		(stored?.languages ?? []).map((language) => [language.language, language])
	);
	const languages = list(values.languages).map(
		(code) =>
			storedLanguages.get(code) ?? {
				language: code,
				interface: true,
				audio: false,
				subtitles: false
			}
	);

	const minutes = values.timeToBeat ? Number(values.timeToBeat) : null;

	return {
		slug: orNull(values.slug),
		title: values.title,
		kind: values.kind,
		status: values.status,
		parent_slug: orNull(values.parentSlug),
		series: orNull(values.series),
		summary: orNull(values.summary),
		description: orNull(values.description),
		release_date: date,
		release_precision: precision,
		cover_url: orNull(values.coverUrl),
		backdrop_url: orNull(values.backdropUrl),
		logo_url: orNull(values.logoUrl),
		// NaN or a fraction goes to the API as 0, which it rejects with a clear code.
		time_to_beat_minutes: minutes === null ? null : Number.isInteger(minutes) ? minutes : 0,
		pc_requirements: stored?.pc_requirements ?? null,
		releases,
		genres: list(values.genres),
		themes: list(values.themes),
		modes: list(values.modes),
		perspectives: list(values.perspectives),
		developers: list(values.developers),
		publishers: list(values.publishers),
		screenshots: stored?.screenshots ?? [],
		artworks: stored?.artworks ?? [],
		trailers: stored?.trailers ?? [],
		languages,
		age_ratings: stored?.age_ratings ?? {},
		alt_titles: list(values.altTitles)
	};
}
