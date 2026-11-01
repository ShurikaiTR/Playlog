import { error } from "@sveltejs/kit";
import { getGameDetail } from "#lib/games.js";
import type { PageLoad } from "./$types";

export const load: PageLoad = ({ params }) => {
	const game = getGameDetail(params.slug);
	if (!game) error(404, "Oyun bulunamadı");
	return { game };
};
