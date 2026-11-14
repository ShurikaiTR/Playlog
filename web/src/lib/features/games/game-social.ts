// Placeholder social proof for the game page (rating box, "1.2K played · 340 want to play").
// Nothing stores who played a game yet, so the numbers are derived from the slug. Replace the body of
// `gameSocial` with an API call when lists live on the server; the page only reads this shape.
export type GameSocial = {
	// Average score out of 10 and how many people rated.
	score: number;
	votes: number;
	played: number;
	wantToPlay: number;
	// Initials of a few people who played it, for the avatar stack.
	people: string[];
};

const PEOPLE = ['DA', 'EK', 'MY', 'AS', 'CB', 'ZT'];

export function gameSocial(slug: string): GameSocial {
	let hash = 0;
	for (const char of slug) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
	return {
		score: 6.5 + ((hash >>> 4) % 30) / 10,
		votes: 120 + ((hash >>> 12) % 9000),
		played: 200 + (hash % 4800),
		wantToPlay: 40 + ((hash >>> 8) % 1200),
		people: PEOPLE.slice(hash % 3, (hash % 3) + 4)
	};
}

export const compactCount = (value: number) =>
	new Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 1 }).format(value);
