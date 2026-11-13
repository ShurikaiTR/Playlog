import type { GameSummary } from '#lib/features/games/games.js';

/** One slide of the home page showcase. */
export type HeroSlide = {
	slug: string;
	title: string;
	image: string;
	logo?: string;
	rating?: string;
	genres: string[];
	description: string;
	href: string;
};

/** Games with wide artwork make slides; the rest are skipped. */
export const toHeroSlides = (games: GameSummary[], count = 5): HeroSlide[] =>
	games
		.filter((game) => game.backdrop)
		.slice(0, count)
		.map((game) => ({
			slug: game.slug,
			title: game.title,
			image: game.backdrop!,
			logo: game.logo ?? undefined,
			genres: game.genres.slice(0, 3),
			description: game.summary ?? '',
			href: `/games/${game.slug}`
		}));
