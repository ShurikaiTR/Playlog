import type { RequestEvent } from '@sveltejs/kit';
import type { GameSocial } from '#lib/features/games/game-social.js';
import type { UserList } from '#lib/features/lists/user-lists.svelte.js';
import { api } from './api.js';

type Event = Pick<RequestEvent, 'request' | 'getClientAddress'>;

type ApiPerson = { username: string; name: string; avatar_url: string | null };
type ApiSocial = {
	played: number;
	play_later: number;
	rating_avg: number | null;
	rating_count: number;
	review_count: number;
	players: ApiPerson[];
	reviews: (ApiPerson & {
		text: string;
		spoiler: boolean;
		score: number | null;
		created_at: string;
		mine: boolean;
	})[];
	mine: {
		rating: number | null;
		review: { text: string; spoiler: boolean } | null;
		lists: string[];
	} | null;
};

const emptySocial: GameSocial = {
	played: 0,
	playLater: 0,
	score: null,
	ratings: 0,
	reviewCount: 0,
	people: [],
	reviews: [],
	mine: null
};

const dateFormat = new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeZone: 'UTC' });

/** Totals, reviews and the viewer's own state for a game. A failure reads as "nothing yet". */
export async function getGameSocial(
	event: Event,
	slug: string,
	token: string | null
): Promise<GameSocial> {
	const result = await api<ApiSocial>(event, `/v1/games/${encodeURIComponent(slug)}/social`, {
		method: 'GET',
		token
	});
	if (!result.ok) return emptySocial;
	const data = result.data;
	return {
		played: data.played,
		playLater: data.play_later,
		score: data.rating_avg,
		ratings: data.rating_count,
		reviewCount: data.review_count,
		people: data.players.map((p) => ({ username: p.username, name: p.name, image: p.avatar_url })),
		reviews: data.reviews.map((r) => ({
			key: `${r.username}`,
			username: r.username,
			name: r.name,
			image: r.avatar_url,
			text: r.text,
			spoiler: r.spoiler,
			score: r.score,
			date: dateFormat.format(new Date(r.created_at)),
			mine: r.mine
		})),
		mine: data.mine
	};
}

type ApiList = {
	id: string;
	kind: UserList['kind'];
	name: string;
	description: string;
	visibility: UserList['visibility'];
	cover_slug: string | null;
	items: {
		slug: string;
		title: string;
		cover_url: string | null;
		backdrop_url: string | null;
		release_date: string | null;
		added_at: string;
	}[];
};

/** The signed-in member's lists with their games; null when the API cannot be reached. */
export async function getMyLists(event: Event, token: string): Promise<UserList[] | null> {
	const result = await api<ApiList[]>(event, '/v1/me/lists', { method: 'GET', token });
	if (!result.ok) return null;
	return result.data.map((list) => ({
		id: list.id,
		kind: list.kind,
		name: list.name,
		description: list.description,
		visibility: list.visibility,
		coverSlug: list.cover_slug,
		items: list.items.map((item) => ({
			slug: item.slug,
			title: item.title,
			cover: item.cover_url,
			backdrop: item.backdrop_url,
			year: item.release_date?.slice(0, 4) ?? '',
			added: Date.parse(item.added_at)
		}))
	}));
}
