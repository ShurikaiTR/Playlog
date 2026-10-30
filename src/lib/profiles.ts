import { allGameDetails } from "./games.js";

export type Person = { name: string; username: string; image?: string };

export type ProfileList = {
	id: string;
	kind: "favorites" | "play_later" | "custom";
	name: string;
	description?: string;
	cover: string;
	gameCount: number;
};

export type Profile = Person & {
	bio?: string;
	cover: string;
	following: Person[];
	followers: Person[];
	lists: ProfileList[];
};

const cover = (i: number) => allGameDetails[i % allGameDetails.length].backdrop;

const people: Person[] = [
	{ name: "Deniz Aksoy", username: "deniz" },
	{ name: "Elif Karaca", username: "elif" },
	{ name: "Mert Yıldız", username: "mert" },
	{ name: "Zeynep Arslan", username: "zeynep" },
];

// Placeholder profiles until users come from real data.
const profiles: Record<string, Profile> = {
	gokhan: {
		name: "Gökhan Güler",
		username: "gokhan",
		bio: "Hikâyesi güçlü oyunların peşinde. Hafta sonları RPG, gece yarısı korku.",
		cover: cover(0),
		following: people.slice(0, 3),
		followers: people,
		lists: [
			{ id: "favoriler", kind: "favorites", name: "Favoriler", description: "Tekrar tekrar oynadıklarım", cover: cover(1), gameCount: 12 },
			{ id: "oynayacaklarim", kind: "play_later", name: "Planlananlar", description: "Sırada bekleyenler", cover: cover(2), gameCount: 27 },
			{ id: "gece-yarisi", kind: "custom", name: "Gece yarısı korkuları", description: "Işıklar kapalıyken", cover: cover(3), gameCount: 8 },
			{ id: "kisa-ve-oz", kind: "custom", name: "Kısa ve öz", description: "Bir akşamda biten oyunlar", cover: cover(4), gameCount: 14 },
		],
	},
	deniz: {
		name: "Deniz Aksoy",
		username: "deniz",
		bio: "Strateji ve simülasyon tutkunu.",
		cover: cover(2),
		following: [people[1], people[3]],
		followers: [people[1]],
		lists: [{ id: "favoriler", kind: "favorites", name: "Favoriler", description: "Şimdilik en iyiler", cover: cover(5), gameCount: 6 }],
	},
};

// The signed-in user until there is real auth.
export const viewerUsername = "gokhan";

export const getProfile = (username: string): Profile | undefined => profiles[username.toLowerCase()];
export const hasProfile = (username: string) => username in profiles;
