import { json } from '@sveltejs/kit';
import { adminErrorMessage } from '#lib/features/admin/game-form.js';
import { adminApi, requireStaff } from '#lib/server/admin.js';
import type { ImportJob } from '#lib/features/admin/import.js';
import type { RequestHandler } from './$types';

// The bulk import runs inside the API; these three calls start it, stop it and read how it is doing.
const reply = (result: Awaited<ReturnType<typeof adminApi<ImportJob | null>>>) =>
	result.ok
		? json({ job: result.data })
		: json({ error: adminErrorMessage(result.error) }, { status: result.status });

export const GET: RequestHandler = async (event) => {
	requireStaff(event.locals, 'admin');
	return reply(await adminApi<ImportJob | null>(event, '/import/jobs/current'));
};

export const POST: RequestHandler = async (event) => {
	requireStaff(event.locals, 'admin');
	const body = (await event.request.json().catch(() => null)) as Record<string, unknown> | null;
	if (!body) return json({ error: adminErrorMessage('invalid_batch_size') }, { status: 400 });
	return reply(
		await adminApi<ImportJob>(event, '/import/jobs/current', {
			method: 'POST',
			body: {
				limit: Number(body.limit),
				platform: typeof body.platform === 'string' ? body.platform : null,
				year_from: body.yearFrom ? Number(body.yearFrom) : null,
				year_to: body.yearTo ? Number(body.yearTo) : null,
				status: body.status === 'published' ? 'published' : 'draft',
				update_existing: body.updateExisting === true
			}
		})
	);
};

export const DELETE: RequestHandler = async (event) => {
	requireStaff(event.locals, 'admin');
	const result = await adminApi(event, '/import/jobs/current', { method: 'DELETE' });
	return result.ok
		? json({ job: null })
		: json({ error: adminErrorMessage(result.error) }, { status: result.status });
};
