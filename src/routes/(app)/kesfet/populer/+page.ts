import { redirect } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

// Popular is the default order of the browse page.
export const load: PageLoad = () => redirect(307, "/oyunlar");
