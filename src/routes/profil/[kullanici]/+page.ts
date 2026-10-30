import { error } from "@sveltejs/kit";
import { getProfile, hasProfile, viewerUsername } from "#lib/profiles.js";
import type { PageLoad } from "./$types";

export const load: PageLoad = ({ params }) => {
	const profile = getProfile(params.kullanici);
	if (!profile) error(404, "Profil bulunamadı");
	return {
		profile,
		isOwner: profile.username === viewerUsername,
		// Only people with a profile page get a link.
		linkable: [...profile.following, ...profile.followers].filter((p) => hasProfile(p.username)).map((p) => p.username),
	};
};
