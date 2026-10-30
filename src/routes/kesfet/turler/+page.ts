import { allGameDetails } from "#lib/games.js";
import type { PageLoad } from "./$types";

// Every genre with how many games carry it, most common first.
export const load: PageLoad = () => {
	const counts = new Map<string, number>();
	for (const game of allGameDetails) for (const genre of game.genres) counts.set(genre, (counts.get(genre) ?? 0) + 1);
	const genres = [...counts].map(([name, count]) => ({ name, count })).sort((a, b) => b.count - a.count || a.name.localeCompare(b.name, "tr"));
	return { genres };
};
