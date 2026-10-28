import { moodGames, moods } from "#lib/games.js";
import type { PageServerLoad } from "./$types";

// Without a chosen mood the page opens on a random one, picked once per request on the server so
// hydration shows the same mood.
export const load: PageServerLoad = ({ url }) => {
	const mood = moods.find((m) => m.key === url.searchParams.get("ruh")) ?? moods[Math.floor(Math.random() * moods.length)];
	const others = moods.filter((m) => m.key !== mood.key);
	return {
		mood: { key: mood.key, title: mood.title, subtitle: mood.subtitle, items: moodGames(mood) },
		// "Rastgele bir oyun" opens one of the other moods.
		surprise: others[Math.floor(Math.random() * others.length)].key,
	};
};
