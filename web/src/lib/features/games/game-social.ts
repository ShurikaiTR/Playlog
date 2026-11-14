// What people did with a game, as the game page and the library menu read it. The Rust API
// computes the totals (`GET /v1/games/{slug}/social`); `lib/server/social.ts` translates them.
export type SocialPerson = { username: string; name: string; image: string | null };

export type SocialReview = {
	key: string;
	username: string;
	name: string;
	image: string | null;
	text: string;
	spoiler: boolean;
	score: number | null;
	date: string;
	mine: boolean;
};

export type GameSocial = {
	played: number;
	playLater: number;
	// Average out of 10, null while nobody has rated.
	score: number | null;
	ratings: number;
	reviewCount: number;
	// A few people who played it, newest first, for the avatar stack.
	people: SocialPerson[];
	reviews: SocialReview[];
	// The signed-in viewer's own score, review and lists; null when signed out.
	mine: {
		rating: number | null;
		review: { text: string; spoiler: boolean } | null;
		lists: string[];
	} | null;
};

export const compactCount = (value: number) =>
	new Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 1 }).format(value);

export const initials = (name: string) =>
	name
		.split(/\s+/)
		.filter(Boolean)
		.slice(0, 2)
		.map((part) => part[0]?.toUpperCase())
		.join('') || '?';
