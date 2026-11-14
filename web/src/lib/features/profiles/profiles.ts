export type Person = { name: string; username: string; image?: string };

export type ProfileList = {
	id: string;
	kind: 'played' | 'play_later' | 'custom';
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
