import { redirect } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

// The old address of the play-later list.
export const load: PageLoad = () => redirect(308, "/planlananlar");
