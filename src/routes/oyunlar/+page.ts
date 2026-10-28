import { allGameDetails, gameModes, platforms, playtimeOf, playtimes, sorts } from "#lib/games.js";
import type { PageLoad } from "./$types";

const unique = (values: string[]) => [...new Set(values)];
const genres = unique(allGameDetails.flatMap((g) => g.genres)).sort((a, b) => a.localeCompare(b, "tr"));
const years = unique(allGameDetails.map((g) => g.releaseDate.slice(0, 4))).sort().reverse();
const ages = unique(allGameDetails.map((g) => g.ageRating)).sort((a, b) => parseInt(a) - parseInt(b));
const themes = unique(allGameDetails.flatMap((g) => g.themes)).sort((a, b) => a.localeCompare(b, "tr"));
const scores = ["7", "8", "9"];

// The filters live in the address (?q=…&platform=…&mod=…&tur=…&tema=…&sure=…&yil=…&yas=…&puan=…&sirala=…) so a filtered list can be shared
// and the back button works. Genres and themes narrow the list down (a game needs all of them), the other groups widen it
// (a game needs any of them) and the score is a minimum.
export const load: PageLoad = ({ url }) => {
	const pick = (key: string, options: readonly string[]) => url.searchParams.getAll(key).filter((value) => options.includes(value));
	const q = (url.searchParams.get("q") ?? "").trim();
	const filters = {
		platform: pick("platform", platforms),
		mod: pick("mod", gameModes),
		tur: pick("tur", genres),
		tema: pick("tema", themes),
		sure: pick("sure", playtimes),
		yil: pick("yil", years),
		yas: pick("yas", ages),
		puan: pick("puan", scores).slice(0, 1),
	};
	const sort = sorts.find((s) => s.key === url.searchParams.get("sirala"))?.key ?? "populer";

	const needle = q.toLocaleLowerCase("tr");
	const games = allGameDetails
		.filter((g) => !needle || g.title.toLocaleLowerCase("tr").includes(needle) || g.developer.toLocaleLowerCase("tr").includes(needle))
		.filter((g) => filters.platform.length === 0 || g.platforms.some((p) => filters.platform.includes(p)))
		.filter((g) => filters.mod.length === 0 || g.modes.some((m) => filters.mod.includes(m)))
		.filter((g) => filters.tur.every((tur) => g.genres.includes(tur)))
		.filter((g) => filters.tema.every((tema) => g.themes.includes(tema)))
		.filter((g) => filters.sure.length === 0 || filters.sure.includes(playtimeOf(g.playtime)))
		.filter((g) => filters.yil.length === 0 || filters.yil.includes(g.releaseDate.slice(0, 4)))
		.filter((g) => filters.yas.length === 0 || filters.yas.includes(g.ageRating))
		.filter((g) => filters.puan.length === 0 || g.score >= Number(filters.puan[0]))
		.sort((a, b) => {
			if (sort === "yeni") return b.releaseDate.localeCompare(a.releaseDate);
			if (sort === "puan") return b.score - a.score;
			if (sort === "ad") return a.title.localeCompare(b.title, "tr");
			return b.votes - a.votes;
		});

	return { games, q, filters, sort, total: allGameDetails.length, options: { platform: [...platforms], mod: [...gameModes], tur: genres, tema: themes, sure: [...playtimes], yil: years, yas: ages, puan: scores } };
};
