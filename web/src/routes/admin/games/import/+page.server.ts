import { fail, redirect } from '@sveltejs/kit';
import { adminErrorMessage } from '#lib/features/admin/game-form.js';
import type { ImportHit } from '#lib/features/admin/import.js';
import {
	adminApi,
	buildGameInput,
	detailFromImport,
	requireStaff,
	valuesFromForm,
	valuesFromImport,
	type ImportedGame,
	type PlatformOption
} from '#lib/server/admin.js';
import type { Actions, PageServerLoad } from './$types';

// Importing reads Wikidata and Steam through the API and writes the catalog: admin only.
export const load: PageServerLoad = async (event) => {
	requireStaff(event.locals, 'admin');
	const platforms = await adminApi<PlatformOption[]>(event, '/platforms');
	return { platforms: platforms.ok ? platforms.data : [] };
};

const text = (form: FormData, key: string) => {
	const value = form.get(key);
	return typeof value === 'string' ? value.trim() : '';
};

export const actions: Actions = {
	// Search by name, or by a pasted Steam link or app id.
	search: async (event) => {
		requireStaff(event.locals, 'admin');
		const q = text(await event.request.formData(), 'q').slice(0, 200);
		const result = await adminApi<{ hits: ImportHit[] }>(
			event,
			`/import/search?q=${encodeURIComponent(q)}`
		);
		if (!result.ok) return fail(result.status, { q, error: adminErrorMessage(result.error) });
		return { q, hits: result.data.hits };
	},

	// Fetch one game from the sources and show it in the form; nothing is saved yet.
	preview: async (event) => {
		requireStaff(event.locals, 'admin');
		const form = await event.request.formData();
		const source = text(form, 'source');
		const id = text(form, 'id');
		const key = source === 'wikidata' ? 'wikidata' : 'steam';
		const result = await adminApi<{
			game: ImportedGame;
			review: string | null;
			wikidata: boolean;
			steam: boolean;
			existing_slug: string | null;
		}>(event, `/import/preview?${key}=${encodeURIComponent(id)}`);
		if (!result.ok) return fail(result.status, { error: adminErrorMessage(result.error) });
		const { game, review, wikidata, steam, existing_slug } = result.data;
		return {
			preview: {
				title: game.title,
				review,
				wikidata,
				steam,
				existingSlug: existing_slug,
				values: valuesFromImport(game),
				// Carried through the form so what it does not show is saved too.
				imported: JSON.stringify(game)
			}
		};
	},

	// Save the previewed game, with the admin's edits, as a new game.
	save: async (event) => {
		requireStaff(event.locals, 'admin');
		const form = await event.request.formData();
		let imported: ImportedGame;
		try {
			imported = JSON.parse(text(form, 'imported'));
		} catch {
			return fail(400, { error: adminErrorMessage('invalid_format') });
		}
		const values = valuesFromForm(form);
		const input = {
			...buildGameInput(values, detailFromImport(imported)),
			external_ids: imported.external_ids ?? {}
		};
		const result = await adminApi<{ id: string; slug: string }>(event, '/games', {
			method: 'POST',
			body: input
		});
		if (!result.ok) return fail(result.status, { error: adminErrorMessage(result.error) });
		redirect(303, `/admin/games/${result.data.slug}`);
	}
};
