// Shapes of the import endpoints, shared by the import page and its job route.

export type ImportItem = {
	title: string;
	outcome: 'created' | 'updated' | 'skipped' | 'failed';
	slug?: string;
	/** An error or skip code. */
	reason?: string;
	/** Why the game needs a look (no release date, sources disagree). */
	review?: string;
};

export type ImportJob = {
	state: 'running' | 'done' | 'cancelled' | 'failed';
	limit: number;
	processed: number;
	created: number;
	updated: number;
	skipped: number;
	failed: number;
	review: number;
	error?: string;
	started_at: string;
	items: ImportItem[];
};

export type ImportHit = {
	source: 'steam' | 'wikidata';
	id: string;
	name: string;
	description: string | null;
	image: string | null;
	existing_slug: string | null;
};

export const skipReasons: Record<string, string> = {
	already_in_catalog: 'Already in the catalog',
	steam_app_not_a_game: 'Not a game on Steam'
};
