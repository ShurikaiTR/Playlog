import { allGameDetails, sorts } from "#lib/games.js";
import type { PageLoad } from "./$types";

const platformOptions = ["PC", "PlayStation", "Xbox", "Switch", "Mobil"];
const genreOptions = ["Aksiyon", "Macera", "RPG", "Strateji", "Nişancı", "Simülasyon", "Spor", "Yarış", "Bulmaca", "Korku", "Platform", "Dövüş"];
const modeOptions = ["Tek oyunculu", "Çok oyunculu", "Co-op"];
// The genres the catalog really has also work in the address (the Türler page links to them), even when the drawer does not list them.
const knownGenres = [...new Set([...genreOptions, ...allGameDetails.flatMap((g) => g.genres)])];
const minYear = 1990;
const maxYear = new Date().getFullYear();
const today = new Date().toISOString().slice(0, 10);

// The filters live in the address (?platform=…&tur=…&mod=…&yilmin=…&yilmax=…&yakinda=1&puan=…&sirala=…) so a filtered list can be shared
// and the back button works. Platforms and modes widen the list (a game needs any of them), genres narrow it (a game needs all of them).
export const load: PageLoad = ({ url }) => {
	const params = url.searchParams;
	const pick = (key: string, options: readonly string[]) => params.getAll(key).filter((value) => options.includes(value));
	const number = (key: string, min: number, max: number, fallback: number) => {
		const value = Number(params.get(key));
		return params.get(key) !== null && Number.isFinite(value) ? Math.min(max, Math.max(min, value)) : fallback;
	};
	const filters = {
		platform: pick("platform", platformOptions),
		tur: pick("tur", knownGenres),
		mod: pick("mod", modeOptions),
		yilmin: number("yilmin", minYear, maxYear, minYear),
		yilmax: number("yilmax", minYear, maxYear, maxYear),
		yakinda: params.get("yakinda") === "1",
		puan: number("puan", 0, 10, 0),
	};
	if (filters.yilmin > filters.yilmax) [filters.yilmin, filters.yilmax] = [filters.yilmax, filters.yilmin];
	const sort = sorts.find((s) => s.key === params.get("sirala"))?.key ?? "populer";

	const onPlatform = (g: (typeof allGameDetails)[number], p: string) => (p === "Mobil" ? g.platforms.some((x) => x === "iOS" || x === "Android") : g.platforms.some((x) => x === p));

	const games = allGameDetails
		.filter((g) => filters.platform.length === 0 || filters.platform.some((p) => onPlatform(g, p)))
		.filter((g) => filters.tur.every((tur) => g.genres.includes(tur)))
		.filter((g) => filters.mod.length === 0 || filters.mod.some((m) => g.modes.some((x) => x === m)))
		.filter((g) => {
			const year = Number(g.releaseDate.slice(0, 4));
			return year >= filters.yilmin && year <= filters.yilmax;
		})
		.filter((g) => !filters.yakinda || g.releaseDate > today)
		.filter((g) => g.score >= filters.puan)
		.sort((a, b) => {
			if (sort === "yeni") return b.releaseDate.localeCompare(a.releaseDate);
			if (sort === "puan") return b.score - a.score;
			if (sort === "ad") return a.title.localeCompare(b.title, "tr");
			return b.votes - a.votes;
		});

	return { games, filters, sort, total: allGameDetails.length, minYear, maxYear, options: { platform: platformOptions, tur: genreOptions, mod: modeOptions } };
};
