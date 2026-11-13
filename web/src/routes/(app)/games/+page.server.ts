import { platformFamilies, sorts, type SortKey } from '#lib/features/games/games.js';
import { cardMeta, listGames, listTags } from '#lib/server/catalog.js';
import type { PageServerLoad } from './$types';

const PAGE_SIZE = 60;
const MIN_YEAR = 1980;
/** The filter panel offers the most common genres; the rest still work from a link. */
const GENRE_OPTIONS = 18;

// The filters live in the address (?platform=…&genre=…&mode=…&from=…&to=…&upcoming=1&sort=…&page=…)
// so a filtered list can be shared and the back button works. Platforms and modes widen the list
// (a game needs any of them), genres narrow it (a game needs all of them).
export const load: PageServerLoad = async (event) => {
	const params = event.url.searchParams;
	// Announced games can be a few years out.
	const maxYear = new Date().getUTCFullYear() + 3;

	const [genreTags, modeTags] = await Promise.all([
		listTags(event, 'genre'),
		listTags(event, 'mode')
	]);
	const option = (tag: { slug: string; name: string }) => ({ value: tag.slug, label: tag.name });
	const known = {
		platform: platformFamilies.map((family) => ({
			value: family.key as string,
			label: family.label as string
		})),
		genre: genreTags.map(option),
		mode: modeTags.map(option)
	};

	const pick = (key: keyof typeof known) => {
		const allowed = new Set(known[key].map((item) => item.value));
		return [...new Set(params.getAll(key))].filter((value) => allowed.has(value));
	};
	const year = (key: string, fallback: number) => {
		const value = Number(params.get(key));
		return params.get(key) !== null && Number.isInteger(value)
			? Math.min(maxYear, Math.max(MIN_YEAR, value))
			: fallback;
	};
	const filters = {
		platform: pick('platform'),
		genre: pick('genre'),
		mode: pick('mode'),
		from: year('from', MIN_YEAR),
		to: year('to', maxYear),
		upcoming: params.get('upcoming') === '1'
	};
	if (filters.from > filters.to) [filters.from, filters.to] = [filters.to, filters.from];

	const sort: SortKey = sorts.find((item) => item.key === params.get('sort'))?.key ?? 'newest';
	const requested = Number(params.get('page'));
	const page = Number.isInteger(requested) && requested > 0 ? Math.min(requested, 1000) : 1;

	const result = await listGames(event, {
		genres: filters.genre,
		platforms: filters.platform,
		modes: filters.mode,
		yearMin: filters.from !== MIN_YEAR ? filters.from : undefined,
		yearMax: filters.to !== maxYear ? filters.to : undefined,
		window: filters.upcoming ? 'upcoming' : undefined,
		sort,
		limit: PAGE_SIZE,
		offset: (page - 1) * PAGE_SIZE
	});

	// A chosen genre stays listed even when it is not among the most common ones.
	const genreOptions = known.genre.filter(
		(item, index) => index < GENRE_OPTIONS || filters.genre.includes(item.value)
	);

	return {
		games: result.games.map((game) => ({ ...game, meta: cardMeta(game) })),
		total: result.total,
		unavailable: !result.ok,
		filters,
		sort,
		page,
		pages: Math.max(1, Math.ceil(result.total / PAGE_SIZE)),
		minYear: MIN_YEAR,
		maxYear,
		options: { platform: known.platform, genre: genreOptions, mode: known.mode }
	};
};
