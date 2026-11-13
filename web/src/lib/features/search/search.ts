/** A game as the search dialog lists and previews it. */
export type GameResult = {
	slug: string;
	title: string;
	cover: string | null;
	backdrop: string | null;
	genres: string[];
	summary: string | null;
	developer: string;
	year: string;
};

/**
 * Games matching the query, or suggestions when it is empty. The catalog is searched on the
 * server (typos included); a failed request simply gives no results.
 */
export async function searchGames(query: string, signal?: AbortSignal): Promise<GameResult[]> {
	try {
		const response = await fetch(`/api/search?q=${encodeURIComponent(query.trim())}`, { signal });
		return response.ok ? await response.json() : [];
	} catch {
		// Aborted by a newer query, or offline.
		return [];
	}
}
