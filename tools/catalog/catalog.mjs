#!/usr/bin/env node
// Builds catalog data from Wikidata and Steam and pushes it to the Playthru API.
//
//   node tools/catalog/catalog.mjs fetch --limit 200 --out catalog.json
//   node tools/catalog/catalog.mjs push catalog.json
//
// `fetch` writes a file in the format of POST /v1/admin/games/import (see docs/plan.md).
// `push` sends it in batches. It needs API_URL (default http://localhost:8080) and
// CATALOG_IMPORT_TOKEN, the same value the API was started with.
//
// Facts come from structured sources only: Wikidata gives the title, the first release date,
// the platforms and the series; Steam gives descriptions, images, genres, languages, age
// ratings and PC requirements. Nothing is generated. Needs Node 22+, no dependencies.

import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';

const USER_AGENT = 'playthru-catalog-importer/0.1 (https://playthru.app)';
const CACHE_DIR = path.join(import.meta.dirname, '.cache');
const STEAM_ASSETS = 'https://shared.akamai.steamstatic.com/store_item_assets/';
// Steam's store API allows roughly 200 requests per 5 minutes.
const STEAM_DELAY_MS = 1600;
const RETRY_DELAY_MS = 65_000;

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/** Fetches JSON, keeping a copy on disk so a rerun does not ask the source again. */
async function cachedJson(key, load) {
	const file = path.join(CACHE_DIR, `${key}.json`);
	if (existsSync(file)) return JSON.parse(await readFile(file, 'utf8'));
	const data = await load();
	await mkdir(CACHE_DIR, { recursive: true });
	await writeFile(file, JSON.stringify(data));
	return data;
}

async function getJson(url, init = {}, attempt = 1) {
	const response = await fetch(url, {
		...init,
		headers: { 'user-agent': USER_AGENT, accept: 'application/json', ...init.headers }
	});
	// Both sources answer 429 (or 5xx) under load. Wikidata can drop to one request a minute for
	// busy addresses, so the wait is a little over that, and a few tries are allowed.
	if ((response.status === 429 || response.status >= 500) && attempt <= 6) {
		const retryAfter = Number(response.headers.get('retry-after'));
		await sleep(Number.isFinite(retryAfter) && retryAfter > 0 ? retryAfter * 1000 : RETRY_DELAY_MS);
		return getJson(url, init, attempt + 1);
	}
	if (!response.ok) throw new Error(`${response.status} from ${new URL(url).host}`);
	return response.json();
}

// ---------- Wikidata ----------

async function sparql(query) {
	const url = `https://query.wikidata.org/sparql?query=${encodeURIComponent(query)}`;
	const data = await getJson(url, { headers: { accept: 'application/sparql-results+json' } });
	return data.results.bindings.map((row) =>
		Object.fromEntries(Object.entries(row).map(([key, cell]) => [key, cell.value]))
	);
}

/** The video games on Steam that Wikipedia covers in the most languages: a popularity proxy. */
function popularGames(limit) {
	return cachedJson(`wikidata-top-${limit}`, () =>
		sparql(`
			SELECT ?item ?itemLabel ?appid ?links WHERE {
				?item wdt:P31 wd:Q7889; wdt:P1733 ?appid; wikibase:sitelinks ?links.
				SERVICE wikibase:label { bd:serviceParam wikibase:language "en". }
			} ORDER BY DESC(?links) LIMIT ${limit}`)
	);
}

/** First release date, platforms and series for a batch of items. */
function wikidataDetails(ids) {
	return sparql(`
		SELECT ?item (MIN(?date) AS ?released)
			(GROUP_CONCAT(DISTINCT STR(?platform); separator="|") AS ?platformIds)
			(GROUP_CONCAT(DISTINCT ?platformLabel; separator="|") AS ?platforms)
			(GROUP_CONCAT(DISTINCT ?seriesLabel; separator="|") AS ?series) WHERE {
			VALUES ?item { ${ids.map((id) => `wd:${id}`).join(' ')} }
			OPTIONAL { ?item wdt:P577 ?date. }
			OPTIONAL {
				?item wdt:P400 ?platform.
				OPTIONAL { ?platform rdfs:label ?platformLabel. FILTER(LANG(?platformLabel) = "en") }
			}
			OPTIONAL { ?item wdt:P179 ?s. ?s rdfs:label ?seriesLabel. FILTER(LANG(?seriesLabel) = "en") }
		} GROUP BY ?item`);
}

// Wikidata platforms -> the slugs in the `platforms` table, by item id. Anything else is left out.
const PLATFORM_IDS = {
	Q1406: 'pc',
	Q14116: 'mac',
	Q388: 'linux',
	Q63184502: 'ps5',
	Q5014725: 'ps4',
	Q10683: 'ps3',
	Q98973368: 'xbox-series',
	Q13361286: 'xbox-one',
	Q48263: 'xbox-360',
	Q19610114: 'switch',
	Q56942: 'wii-u',
	Q48493: 'ios',
	Q94: 'android'
};

// The same by English label, for platforms not listed above. The query service sometimes
// returns no label at all, which is why the ids come first.
const PLATFORM_LABELS = {
	'Microsoft Windows': 'pc',
	Windows: 'pc',
	macOS: 'mac',
	'Classic Mac OS': 'mac',
	Linux: 'linux',
	'PlayStation 5': 'ps5',
	'PlayStation 4': 'ps4',
	'PlayStation 3': 'ps3',
	'Xbox Series X and Series S': 'xbox-series',
	'Xbox Series X': 'xbox-series',
	'Xbox One': 'xbox-one',
	'Xbox 360': 'xbox-360',
	'Nintendo Switch 2': 'switch-2',
	'Nintendo Switch': 'switch',
	'Wii U': 'wii-u',
	iOS: 'ios',
	Android: 'android'
};

// ---------- Steam ----------

/** Descriptions, genres, languages, ratings and requirements for one app. Null when delisted. */
async function steamDetails(appid) {
	const data = await cachedJson(`steam-app-${appid}`, async () => {
		await sleep(STEAM_DELAY_MS);
		return getJson(
			`https://store.steampowered.com/api/appdetails?appids=${appid}&l=english&cc=us`
		);
	});
	const entry = data?.[appid];
	return entry?.success ? entry.data : null;
}

/** Image file names for a batch of apps, from Steam's store browse service. */
async function steamAssets(appids) {
	const assets = new Map();
	for (let i = 0; i < appids.length; i += 50) {
		const batch = appids.slice(i, i + 50);
		const input = {
			ids: batch.map((appid) => ({ appid: Number(appid) })),
			context: { language: 'english', country_code: 'US' },
			data_request: { include_assets: true }
		};
		const data = await cachedJson(`steam-assets-${batch[0]}-${batch.length}`, () =>
			getJson(
				`https://api.steampowered.com/IStoreBrowseService/GetItems/v1?input_json=${encodeURIComponent(JSON.stringify(input))}`
			)
		);
		for (const item of data.response?.store_items ?? []) {
			if (item.assets) assets.set(String(item.appid), item.assets);
		}
	}
	return assets;
}

/** The full address of one of an app's images, or null when Steam has none. */
function assetUrl(assets, name) {
	const file = assets?.[name];
	if (!file || !assets.asset_url_format) return null;
	return STEAM_ASSETS + assets.asset_url_format.replace('${FILENAME}', file);
}

// Steam writes text as HTML; the catalog stores plain text with blank lines between paragraphs.
function plainText(html) {
	return (html ?? '')
		.replace(/<(br|\/p|\/h\d|\/li|\/ul)\s*\/?>/gi, '\n')
		.replace(/<li[^>]*>/gi, '• ')
		.replace(/<[^>]+>/g, '')
		.replace(/&amp;/g, '&')
		.replace(/&quot;/g, '"')
		.replace(/&#0?39;|&apos;/g, "'")
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&nbsp;/g, ' ')
		.replace(/[ \t]+/g, ' ')
		.replace(/ ?\n ?/g, '\n')
		.replace(/\n{3,}/g, '\n\n')
		.trim();
}

// Steam language names -> BCP 47 codes.
const LANGUAGES = {
	English: 'en',
	French: 'fr',
	Italian: 'it',
	German: 'de',
	'Spanish - Spain': 'es',
	'Spanish - Latin America': 'es-419',
	Arabic: 'ar',
	Bulgarian: 'bg',
	Czech: 'cs',
	Danish: 'da',
	Dutch: 'nl',
	Finnish: 'fi',
	Greek: 'el',
	Hungarian: 'hu',
	Indonesian: 'id',
	Japanese: 'ja',
	Korean: 'ko',
	Norwegian: 'no',
	Polish: 'pl',
	Portuguese: 'pt',
	'Portuguese - Portugal': 'pt',
	'Portuguese - Brazil': 'pt-BR',
	Romanian: 'ro',
	Russian: 'ru',
	'Simplified Chinese': 'zh-Hans',
	'Traditional Chinese': 'zh-Hant',
	Swedish: 'sv',
	Thai: 'th',
	Turkish: 'tr',
	Ukrainian: 'uk',
	Vietnamese: 'vi'
};

/**
 * "English<strong>*</strong>, French, …<br><strong>*</strong>languages with full audio support".
 * Steam only tells interface and audio apart here, so subtitles stay unknown (false).
 */
function languages(html) {
	const list = (html ?? '').split(/<br\s*\/?>/i)[0];
	const seen = new Map();
	for (const part of list.split(',')) {
		const audio = part.includes('*');
		const code = LANGUAGES[plainText(part).replace('*', '').trim()];
		if (code && !seen.has(code))
			seen.set(code, { language: code, interface: true, audio, subtitles: false });
	}
	return [...seen.values()];
}

/** "<strong>OS:</strong> Windows 10<br>" lines -> { os, cpu, gpu, ram, storage }. */
function requirements(html) {
	const fields = { os: 'os', processor: 'cpu', memory: 'ram', graphics: 'gpu', storage: 'storage' };
	const result = {};
	for (const match of (html ?? '').matchAll(/<strong>([^<:]+):?\s*<\/strong>:?\s*([^<]+)/gi)) {
		const key = fields[match[1].trim().toLowerCase().replace(/ \*$/, '')];
		const value = plainText(match[2]);
		if (key && value && !result[key]) result[key] = value;
	}
	return Object.keys(result).length ? result : null;
}

// Steam files these under genres, but they describe how a game is sold, not what it is.
const NOT_GENRES = new Set(['Free To Play', 'Early Access', 'Game Demo']);

const MODES = {
	'Single-player': 'Single player',
	'Multi-player': 'Multiplayer',
	'Co-op': 'Co-op',
	PvP: 'PvP',
	'Online PvP': 'PvP',
	'Online Co-op': 'Co-op',
	'Shared/Split Screen': 'Split screen'
};

// ---------- Building the import file ----------

const summaryOf = (text) => {
	const clean = plainText(text);
	return clean.length <= 500 ? clean : `${clean.slice(0, 497).trimEnd()}…`;
};

const unique = (values) => [...new Set(values.filter(Boolean))];

function toGame(wiki, details, steam, assets, status) {
	const released = details?.released?.slice(0, 10) || null;
	const platforms = unique([
		...(details?.platformIds ?? '').split('|').map((uri) => PLATFORM_IDS[uri.split('/').pop()]),
		...(details?.platforms ?? '').split('|').map((label) => PLATFORM_LABELS[label.trim()])
	]);
	// Steam only sells for these three, and it is the source we are sure about.
	if (steam.platforms?.windows) platforms.push('pc');
	if (steam.platforms?.mac) platforms.push('mac');
	if (steam.platforms?.linux) platforms.push('linux');

	const screenshots = (steam.screenshots ?? []).slice(0, 12).map((shot) => shot.path_full);
	const minimum = requirements(steam.pc_requirements?.minimum);
	const recommended = requirements(steam.pc_requirements?.recommended);
	const ratings = Object.fromEntries(
		['pegi', 'esrb']
			.map((board) => [board, steam.ratings?.[board]?.rating])
			.filter(([, rating]) => rating)
			.map(([board, rating]) => [board, String(rating).toUpperCase()])
	);
	// A Wikidata item with no English label comes back as its own id; Steam's name is better then.
	const title = /^Q\d+$/.test(wiki.itemLabel) ? steam.name : wiki.itemLabel;

	return {
		title,
		status,
		series: details?.series?.split('|')[0] || undefined,
		summary: summaryOf(steam.short_description) || undefined,
		description: plainText(steam.about_the_game || steam.detailed_description) || undefined,
		release_date: released ?? undefined,
		cover_url: assetUrl(assets, 'library_capsule') ?? steam.header_image,
		backdrop_url: assetUrl(assets, 'library_hero') ?? screenshots[0] ?? steam.header_image,
		pc_requirements: minimum || recommended ? { minimum, recommended } : undefined,
		// Only the first release date is known; per-platform dates are left open.
		releases: unique(platforms).map((platform) => ({ platform })),
		genres: unique((steam.genres ?? []).map((genre) => genre.description)).filter(
			(genre) => !NOT_GENRES.has(genre)
		),
		modes: unique((steam.categories ?? []).map((category) => MODES[category.description])),
		developers: unique(steam.developers ?? []),
		publishers: unique(steam.publishers ?? []),
		external_ids: { steam: String(wiki.appid), wikidata: wiki.item.split('/').pop() },
		screenshots,
		languages: languages(steam.supported_languages),
		age_ratings: ratings
	};
}

async function fetchCatalog({ limit, out, status }) {
	console.log(`Asking Wikidata for the ${limit} most covered games on Steam…`);
	const top = await popularGames(limit);
	const ids = top.map((row) => row.item.split('/').pop());

	const details = new Map();
	for (let i = 0; i < ids.length; i += 50) {
		const batch = ids.slice(i, i + 50);
		const rows = await cachedJson(`wikidata-details-v2-${batch[0]}-${batch.length}`, () =>
			wikidataDetails(batch)
		);
		for (const row of rows) details.set(row.item, row);
	}

	const assets = await steamAssets(top.map((row) => row.appid));

	const games = [];
	const skipped = [];
	for (const [index, wiki] of top.entries()) {
		process.stdout.write(`\rSteam ${index + 1}/${top.length}`);
		const steam = await steamDetails(wiki.appid);
		// Delisted apps answer with no data; DLC and tools are not games.
		if (!steam || steam.type !== 'game') {
			skipped.push(wiki.itemLabel);
			continue;
		}
		games.push(toGame(wiki, details.get(wiki.item), steam, assets.get(String(wiki.appid)), status));
	}
	process.stdout.write('\n');

	await writeFile(out, JSON.stringify({ games }, null, '\t'));
	console.log(`Wrote ${games.length} games to ${out}.`);
	if (skipped.length) console.log(`Skipped ${skipped.length} (not on Steam any more, or not a game).`);
}

async function pushCatalog(file) {
	const apiUrl = (process.env.API_URL ?? 'http://localhost:8080').replace(/\/+$/, '');
	const token = process.env.CATALOG_IMPORT_TOKEN;
	if (!token) throw new Error('Set CATALOG_IMPORT_TOKEN to the value the API was started with.');

	const { games } = JSON.parse(await readFile(file, 'utf8'));
	const totals = { created: 0, updated: 0, failed: 0 };
	for (let i = 0; i < games.length; i += 100) {
		const response = await fetch(`${apiUrl}/v1/admin/games/import`, {
			method: 'POST',
			headers: { 'content-type': 'application/json', authorization: `Bearer ${token}` },
			body: JSON.stringify({ games: games.slice(i, i + 100) })
		});
		if (!response.ok) throw new Error(`Import failed: ${response.status} ${await response.text()}`);
		const report = await response.json();
		for (const key of Object.keys(totals)) totals[key] += report[key];
		for (const result of report.results) {
			if (result.outcome === 'failed') console.log(`  failed: ${result.title} (${result.error})`);
		}
	}
	console.log(`Created ${totals.created}, updated ${totals.updated}, failed ${totals.failed}.`);
	if (totals.failed) process.exitCode = 1;
}

const { values, positionals } = parseArgs({
	allowPositionals: true,
	options: {
		limit: { type: 'string', default: '200' },
		out: { type: 'string', default: 'catalog.json' },
		draft: { type: 'boolean', default: false }
	}
});

const [command, file] = positionals;
if (command === 'fetch') {
	const limit = Number(values.limit);
	if (!Number.isInteger(limit) || limit < 1 || limit > 5000)
		throw new Error('--limit must be between 1 and 5000');
	await fetchCatalog({ limit, out: values.out, status: values.draft ? 'draft' : 'published' });
} else if (command === 'push' && file) {
	await pushCatalog(file);
} else {
	console.log(`Usage:
  node tools/catalog/catalog.mjs fetch [--limit 200] [--out catalog.json] [--draft]
  node tools/catalog/catalog.mjs push <file>`);
	process.exitCode = 1;
}
