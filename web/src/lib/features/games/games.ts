// Shapes and display helpers for catalog data. The data itself comes from the Rust API through
// `$lib/server/catalog`; nothing here talks to the network, so components can import it freely.

export type DatePrecision = 'day' | 'month' | 'quarter' | 'year' | 'tba';

/** The least a card needs. `meta` is the small line under the title. */
export type Game = {
	slug: string;
	title: string;
	cover: string | null;
	meta?: string;
};

/** A game in lists, rows, the home showcase and search results. */
export type GameSummary = Game & {
	summary: string | null;
	backdrop: string | null;
	logo: string | null;
	genres: string[];
	developers: string[];
	releaseDate: string | null;
	releasePrecision: DatePrecision;
};

export type Named = { slug: string; name: string };

export type GameLanguage = {
	code: string;
	name: string;
	interface: boolean;
	audio: boolean;
	subtitles: boolean;
};

export type SystemRequirement = Partial<Record<'os' | 'cpu' | 'gpu' | 'ram' | 'storage', string>>;
export type SystemRequirements = {
	minimum: SystemRequirement | null;
	recommended: SystemRequirement | null;
};

export type GameDetail = {
	slug: string;
	title: string;
	kind: string;
	cover: string | null;
	backdrop: string | null;
	logo: string | null;
	summary: string | null;
	description: string | null;
	genres: Named[];
	themes: Named[];
	modes: Named[];
	developers: Named[];
	publishers: Named[];
	releaseDate: string | null;
	releasePrecision: DatePrecision;
	/** Every platform the game came out on, in release order. */
	platforms: { slug: string; name: string; family: string }[];
	languages: GameLanguage[];
	/** The rating to show, e.g. "PEGI 18", with the board's own wording for it. */
	ageRating: { label: string; title: string; text: string } | null;
	/** Hours to finish the main story. */
	playtime: number | null;
	requirements: SystemRequirements | null;
	screenshots: string[];
	/** YouTube video id. */
	trailer: string | null;
	series: Named | null;
	/** The base game, when this is an expansion or a remake. */
	parent: Game | null;
	/** Expansions and other editions of this game. */
	related: Game[];
	seriesGames: Game[];
	similar: Game[];
};

const DAY = new Intl.DateTimeFormat('en-US', {
	day: 'numeric',
	month: 'long',
	year: 'numeric',
	timeZone: 'UTC'
});
const MONTH = new Intl.DateTimeFormat('en-US', { month: 'long', year: 'numeric', timeZone: 'UTC' });
const SHORT_DAY = new Intl.DateTimeFormat('en-US', {
	day: 'numeric',
	month: 'short',
	timeZone: 'UTC'
});

/**
 * A release date as precisely as it is known: "May 19, 2015", "March 2027", "Q1 2027", "2027"
 * or "TBA". Dates are calendar days, so they are read as UTC and never shift with the time zone.
 */
export function formatRelease(date: string | null, precision: DatePrecision): string {
	if (!date || precision === 'tba') return 'TBA';
	const day = new Date(`${date}T00:00:00Z`);
	if (precision === 'year') return String(day.getUTCFullYear());
	if (precision === 'quarter')
		return `Q${Math.floor(day.getUTCMonth() / 3) + 1} ${day.getUTCFullYear()}`;
	return precision === 'month' ? MONTH.format(day) : DAY.format(day);
}

/** The day and month alone ("May 19"), for where the year is shown separately. */
export const formatDayMonth = (date: string) => SHORT_DAY.format(new Date(`${date}T00:00:00Z`));

export const releaseYear = (date: string | null) => (date ? date.slice(0, 4) : '');

/** How the games page can order its list. */
export const sorts = [
	{ key: 'newest', label: 'Newest' },
	{ key: 'title', label: 'Title' },
	{ key: 'added', label: 'Recently added' }
] as const;
export type SortKey = (typeof sorts)[number]['key'];

/** Platform families, as the API groups them. */
export const platformFamilies = [
	{ key: 'pc', label: 'PC' },
	{ key: 'playstation', label: 'PlayStation' },
	{ key: 'xbox', label: 'Xbox' },
	{ key: 'nintendo', label: 'Nintendo' },
	{ key: 'mobile', label: 'Mobile' }
] as const;

/** Moods for the discover page. Each one is a genre under a friendlier name. */
export const moods = [
	{
		key: 'adventure',
		title: 'Grand adventures',
		subtitle: 'Journeys and new worlds',
		genre: 'adventure'
	},
	{
		key: 'action',
		title: 'Pure adrenaline',
		subtitle: 'Fast hands, faster choices',
		genre: 'action'
	},
	{
		key: 'rpg',
		title: 'Stories to live in',
		subtitle: 'Feel the weight of your choices',
		genre: 'rpg'
	},
	{
		key: 'strategy',
		title: 'Think it through',
		subtitle: 'Plans, patience and payoff',
		genre: 'strategy'
	},
	{
		key: 'indie',
		title: 'Small and brilliant',
		subtitle: 'Big ideas from small teams',
		genre: 'indie'
	},
	{
		key: 'simulation',
		title: 'Another life',
		subtitle: 'Build it, run it, live it',
		genre: 'simulation'
	}
] as const;
export type Mood = (typeof moods)[number];

// What the rating boards mean by each label.
const PEGI: Record<string, string> = {
	'3': 'Suitable for all ages.',
	'7': 'May contain scenes or sounds that can frighten younger children.',
	'12': 'May contain mild violence, mild bad language or suggestive themes.',
	'16': 'May contain realistic violence, strong language or the use of tobacco and alcohol.',
	'18': 'Adults only: may contain gross violence, glamorised drug use or explicit content.'
};
const ESRB: Record<string, [title: string, text: string]> = {
	E: ['Everyone', 'Content is generally suitable for all ages.'],
	E10: ['Everyone 10+', 'Content is generally suitable for ages 10 and up.'],
	'E10+': ['Everyone 10+', 'Content is generally suitable for ages 10 and up.'],
	T: ['Teen', 'Content is generally suitable for ages 13 and up.'],
	M: ['Mature 17+', 'Content is generally suitable for ages 17 and up.'],
	AO: ['Adults Only 18+', 'Content suitable only for adults ages 18 and up.']
};

/** The rating to show for a game: PEGI when there is one, otherwise ESRB. */
export function ageRating(ratings: Record<string, string>): GameDetail['ageRating'] {
	const pegi = ratings.pegi;
	if (pegi && PEGI[pegi])
		return { label: `PEGI ${pegi}`, title: `Ages ${pegi} and up`, text: PEGI[pegi] };
	const esrb = ratings.esrb?.toUpperCase();
	if (esrb && ESRB[esrb])
		return { label: `ESRB ${esrb}`, title: ESRB[esrb][0], text: ESRB[esrb][1] };
	return null;
}
