// What the admin game form edits, as plain strings so it can round-trip through a form post.
// Fields the form does not show (screenshots, trailers, age ratings, system requirements, external
// ids, per-region releases) are carried over from the stored game on save: see `server/admin.ts`.

export const GAME_KINDS = [
	'game',
	'dlc',
	'expansion',
	'bundle',
	'remake',
	'remaster',
	'port'
] as const;
export const GAME_STATUSES = ['draft', 'published', 'hidden'] as const;
export const DATE_PRECISIONS = ['day', 'month', 'quarter', 'year'] as const;

export type GameKind = (typeof GAME_KINDS)[number];
export type GameStatus = (typeof GAME_STATUSES)[number];

export type GameFormValues = {
	title: string;
	slug: string;
	kind: string;
	status: string;
	summary: string;
	description: string;
	releaseDate: string;
	releasePrecision: string;
	coverUrl: string;
	backdropUrl: string;
	logoUrl: string;
	/** Minutes to finish the main story. */
	timeToBeat: string;
	series: string;
	parentSlug: string;
	/** Comma-separated names. */
	genres: string;
	themes: string;
	modes: string;
	perspectives: string;
	developers: string;
	publishers: string;
	altTitles: string;
	/** Comma-separated BCP 47 codes: en, tr, pt-BR. */
	languages: string;
	platforms: string[];
};

export const emptyGameForm = (): GameFormValues => ({
	title: '',
	slug: '',
	kind: 'game',
	status: 'draft',
	summary: '',
	description: '',
	releaseDate: '',
	releasePrecision: 'day',
	coverUrl: '',
	backdropUrl: '',
	logoUrl: '',
	timeToBeat: '',
	series: '',
	parentSlug: '',
	genres: '',
	themes: '',
	modes: '',
	perspectives: '',
	developers: '',
	publishers: '',
	altTitles: '',
	languages: '',
	platforms: []
});

/** The messages for the codes the API's validation answers with. */
export const adminErrors: Record<string, string> = {
	invalid_title: 'The title must be 1–300 characters.',
	invalid_slug: 'The slug can only have lowercase letters, numbers and single hyphens.',
	slug_taken: 'That slug is already used by another game.',
	invalid_kind: 'Pick a valid type.',
	invalid_status: 'Pick a valid status.',
	invalid_summary: 'The summary can be at most 500 characters.',
	invalid_time_to_beat: 'Time to beat must be a positive number of minutes.',
	invalid_url: 'Image addresses must start with https:// and have no spaces.',
	invalid_precision: 'Pick a valid date precision.',
	invalid_region: 'A release has an invalid region.',
	invalid_language: 'Languages must be codes such as en, tr or pt-BR.',
	invalid_tag: 'A genre, theme, mode or perspective name is empty or too long.',
	invalid_company: 'A developer or publisher name is empty or too long.',
	invalid_series: 'The series name is empty or too long.',
	unknown_parent: 'There is no game with that parent slug.',
	unknown_platform: 'One of the platforms does not exist.',
	too_many_items: 'A list has too many entries.',
	game_not_found: 'That game no longer exists.',
	rate_limited: 'Too many changes in a short time. Try again in a few minutes.',
	forbidden: 'You are not allowed to do that.',
	unavailable: 'The API could not be reached. Try again.'
};

export const adminErrorMessage = (code: string): string =>
	adminErrors[code] ?? 'Something went wrong. Try again.';
