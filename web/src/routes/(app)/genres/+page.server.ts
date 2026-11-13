import { listTags } from '#lib/server/catalog.js';
import type { PageServerLoad } from './$types';

// Every genre with how many games carry it, most common first.
export const load: PageServerLoad = async (event) => ({ genres: await listTags(event, 'genre') });
