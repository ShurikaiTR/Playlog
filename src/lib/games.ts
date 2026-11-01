export const platforms = ["PC", "macOS", "Linux", "PlayStation", "Xbox", "Switch", "iOS", "Android"] as const;
export const gameModes = ["Tek oyunculu", "Çok oyunculu", "Co-op", "Rekabetçi", "Çevrimiçi"] as const;
export type GameMode = (typeof gameModes)[number];
export type Platform = (typeof platforms)[number];

export type Game = {
	slug: string;
	title: string;
	cover: string;
	meta?: string;
};

const unsplash = (id: string) => `https://images.unsplash.com/${id}?w=600&q=80&auto=format&fit=crop`;

// Only photo ids already used by the hero are known to resolve; reused as cover placeholders.
const covers = [
	unsplash("photo-1542751371-adc38448a05e"),
	unsplash("photo-1550745165-9bc0b252726f"),
	unsplash("photo-1538481199705-c710c4e965fc"),
];

const titles: [string, string, string][] = [
	["neon-surgun", "Neon Sürgün", "Aksiyon"],
	["piksel-gecesi", "Piksel Gecesi", "Platform"],
	["derin-yorunge", "Derin Yörünge", "Strateji"],
	["golgeler-vadisi", "Gölgeler Vadisi", "RPG"],
	["kirik-kilic", "Kırık Kılıç Efsanesi", "Macera"],
	["son-istasyon", "Son İstasyon", "Korku"],
];

// Placeholder data until games come from real data.
export const trendingGames: Game[] = titles.map(([slug, title, meta], i) => ({
	slug,
	title,
	meta,
	cover: covers[i % covers.length],
}));

const upcomingDates = ["12 Kasım 2026", "3 Aralık 2026", "21 Ocak 2027", "Şubat 2027", "Mart 2027", "2027"];

export const upcomingGames: Game[] = [...trendingGames].reverse().map((game, i) => ({
	...game,
	meta: upcomingDates[i],
	cover: covers[(i + 1) % covers.length],
}));

export type SystemRequirement = { os: string; cpu: string; gpu: string; ram: string; storage: string };
export type SystemRequirements = { minimum: SystemRequirement; recommended: SystemRequirement };

export type GameDetail = Game & {
	backdrop: string;
	genres: string[];
	description: string;
	developer: string;
	publisher: string;
	releaseDate: string;
	ageRating: string;
	score: number;
	votes: number;
	// Hours it takes to finish the main story.
	playtime: number;
	platforms: Platform[];
	modes: GameMode[];
	// Free-form tags describing the setting, mood and feel of the game.
	themes: string[];
	// YouTube video id of the trailer.
	trailer?: string;
	// Screenshot urls; the detail page falls back to placeholders when empty.
	screenshots?: string[];
	// Transparent title logo, shown instead of the title text over the trailer.
	logo?: string;
	// Slugs of other games from the same series (sequels, spin-offs, DLC).
	series?: string[];
	// PC system requirements; left out for games that are not on PC.
	requirements?: SystemRequirements;
};

const extras: Record<string, Omit<GameDetail, keyof Game>> = {
	"neon-surgun": {
		platforms: ["PC", "macOS", "PlayStation", "Xbox"],
		modes: ["Tek oyunculu", "Co-op"],
		themes: ["Distopya", "Cyberpunk", "Atmosferik", "Hikâye odaklı"],
		series: ["golgeler-vadisi", "kirik-kilic"],
		backdrop: covers[0],
		genres: ["Aksiyon", "Bilim kurgu", "Açık dünya"],
		description:
			"Işıkları hiç sönmeyen bir şehirde, kimsenin hatırlamadığı bir geçmişin izini sür. Seçimlerin şehrin kaderini değiştirir; her bölge kendi yasalarıyla yaşayan çetelere ev sahipliği yapar.",
		developer: "Gece Yarısı Stüdyo",
		publisher: "Kuzey Yayıncılık",
		releaseDate: "2025-03-14",
		ageRating: "16+",
		score: 8.7,
		votes: 12840,
		playtime: 25,
	},
	"piksel-gecesi": {
		platforms: ["Switch", "PC", "iOS", "Android"],
		modes: ["Tek oyunculu", "Çok oyunculu"],
		themes: ["Retro", "Zorlayıcı", "Rahatlatıcı", "Bağımsız"],
		backdrop: covers[1],
		genres: ["Retro", "Platform"],
		description:
			"Atari salonlarının ruhunu taşıyan, kısa ama tekrar tekrar dönülen bir platform oyunu. Rekorlarını arkadaşlarınla karşılaştır.",
		developer: "Bit Atölyesi",
		publisher: "Bit Atölyesi",
		releaseDate: "2024-10-02",
		ageRating: "7+",
		score: 7.9,
		votes: 4210,
		playtime: 4,
	},
	"derin-yorunge": {
		platforms: ["PC", "macOS", "Linux"],
		modes: ["Tek oyunculu", "Çevrimiçi"],
		themes: ["Uzay", "Taktiksel", "Derin sistemler", "Bağımsız"],
		backdrop: covers[2],
		genres: ["Strateji", "Uzay", "Simülasyon"],
		description:
			"Filonu kur, rotanı çiz ve galaksinin kenarına kadar ilerle. Topluluğun en çok listeye eklediği strateji oyunlarından biri.",
		developer: "Yörünge Oyun",
		publisher: "Kuzey Yayıncılık",
		releaseDate: "2025-08-21",
		ageRating: "12+",
		score: 8.4,
		votes: 7355,
		playtime: 60,
	},
	"golgeler-vadisi": {
		platforms: ["PC", "PlayStation", "Xbox"],
		modes: ["Tek oyunculu", "Co-op"],
		themes: ["Karanlık fantezi", "Atmosferik", "Hikâye odaklı", "Zorlayıcı"],
		series: ["neon-surgun"],
		backdrop: covers[0],
		genres: ["RPG", "Fantastik"],
		description:
			"Sisle kaplı bir vadide kaybolan köyleri ara. Her karar, yanındaki yoldaşların sana olan güvenini değiştirir.",
		developer: "Kül Atölyesi",
		publisher: "Kül Atölyesi",
		releaseDate: "2026-01-30",
		ageRating: "16+",
		score: 9.1,
		votes: 18920,
		playtime: 45,
	},
	"kirik-kilic": {
		platforms: ["Xbox", "PC", "Switch", "Android"],
		modes: ["Tek oyunculu", "Çok oyunculu", "Rekabetçi"],
		themes: ["Efsane", "Macera", "Açık uçlu"],
		backdrop: covers[1],
		genres: ["Macera", "Aksiyon"],
		description: "Efsanelerde anlatılan kılıcın kırık parçalarını topla ve krallığı karanlıktan kurtar.",
		developer: "Demir Kapı",
		publisher: "Anka Yayınları",
		releaseDate: "2025-11-07",
		ageRating: "12+",
		score: 8.2,
		votes: 6034,
		playtime: 18,
	},
	"son-istasyon": {
		platforms: ["PC", "PlayStation", "Linux"],
		modes: ["Tek oyunculu"],
		themes: ["Uzay", "Gerilim", "Atmosferik", "Hayatta kalma"],
		backdrop: covers[2],
		genres: ["Korku", "Hayatta kalma"],
		description: "Terk edilmiş bir uzay istasyonunda tek başınasın. Sessizlik, duyduğun en tehlikeli ses.",
		developer: "Sessiz Frekans",
		publisher: "Sessiz Frekans",
		releaseDate: "2026-04-18",
		ageRating: "18+",
		score: 7.6,
		votes: 2988,
		playtime: 8,
	},
};

// Placeholder trailer shared by every game until real trailer ids exist.
// Placeholder PC requirements until they come from real data.
const req = (os: string, cpu: string, gpu: string, ram: string, storage: string): SystemRequirement => ({ os, cpu, gpu, ram, storage });
const requirements: Record<string, SystemRequirements> = {
	"neon-surgun": {
		minimum: req("Windows 10 64-bit", "Intel Core i5-8400", "GTX 1060 6 GB", "12 GB", "70 GB SSD"),
		recommended: req("Windows 11 64-bit", "Intel Core i7-10700", "RTX 3070 8 GB", "16 GB", "70 GB SSD"),
	},
	"piksel-gecesi": {
		minimum: req("Windows 10 64-bit", "Intel Core i3-6100", "Intel HD 530", "4 GB", "2 GB"),
		recommended: req("Windows 10 64-bit", "Intel Core i5-8400", "GTX 960", "8 GB", "2 GB"),
	},
	"derin-yorunge": {
		minimum: req("Windows 10 64-bit", "Intel Core i5-7500", "GTX 970", "8 GB", "40 GB"),
		recommended: req("Windows 11 64-bit", "Intel Core i7-9700", "RTX 2060", "16 GB", "40 GB SSD"),
	},
	"golgeler-vadisi": {
		minimum: req("Windows 10 64-bit", "Intel Core i5-9400F", "GTX 1660 6 GB", "16 GB", "90 GB SSD"),
		recommended: req("Windows 11 64-bit", "Intel Core i7-11700", "RTX 3070 8 GB", "32 GB", "90 GB SSD"),
	},
	"kirik-kilic": {
		minimum: req("Windows 10 64-bit", "Intel Core i5-8400", "GTX 1050 Ti", "8 GB", "45 GB"),
		recommended: req("Windows 10 64-bit", "Intel Core i7-8700", "GTX 1070", "16 GB", "45 GB SSD"),
	},
	"son-istasyon": {
		minimum: req("Windows 10 64-bit", "Intel Core i5-6600", "GTX 960 4 GB", "8 GB", "30 GB"),
		recommended: req("Windows 10 64-bit", "Intel Core i7-7700", "GTX 1070", "16 GB", "30 GB SSD"),
	},
};

const placeholderTrailer = "aqz-KE-bpKQ";
const placeholderLogo = "/logos/placeholder.png";

// Placeholder lookup until games come from real data.
export const getGameDetail = (slug: string): GameDetail | undefined => {
	const game = trendingGames.find((g) => g.slug === slug);
	const extra = extras[slug];
	return game && extra ? { ...game, ...extra, requirements: extra.platforms.includes("PC") ? requirements[slug] : undefined, trailer: placeholderTrailer, logo: placeholderLogo } : undefined;
};

// Moods for the discover page. Each one is a hand-picked group of genres.
export const moods = [
	{ key: "maceralar", title: "Büyük maceralar", subtitle: "Yolculuklar ve yeni dünyalar", genres: ["Macera", "Fantastik", "Açık dünya"] },
	{ key: "gerilim", title: "Nefes kesen gerilim", subtitle: "Hayatta kalma ve büyük riskler", genres: ["Korku", "Hayatta kalma"] },
	{ key: "hikaye", title: "Hikâyesi güçlü olanlar", subtitle: "Seçimlerin ağırlığını hisset", genres: ["RPG", "Bilim kurgu"] },
	{ key: "adrenalin", title: "Adrenalin", subtitle: "Çarpışmalar ve hızlı tempo", genres: ["Aksiyon", "Platform"] },
	{ key: "dusunenler", title: "Düşündürenler", subtitle: "Strateji, planlama ve simülasyon", genres: ["Strateji", "Simülasyon", "Uzay"] },
] as const;

export type Mood = (typeof moods)[number];

const moodSize = 10;

export const allGameDetails: GameDetail[] = trendingGames.flatMap((g) => getGameDetail(g.slug) ?? []);

// A small catalog has few games per mood, so the rest of the catalog follows after them.
export const moodGames = (mood: Mood): GameDetail[] => {
	const matches = allGameDetails.filter((g) => g.genres.some((genre) => (mood.genres as readonly string[]).includes(genre)));
	const rest = allGameDetails.filter((g) => !matches.includes(g));
	return [...matches, ...rest].slice(0, moodSize);
};

// How the browse page can order the games.
export const sorts = [
	{ key: "populer", label: "Popüler" },
	{ key: "yeni", label: "En yeni" },
	{ key: "puan", label: "En iyi" },
	{ key: "ad", label: "A–Z" },
] as const;

// How long a game takes, as the browse page buckets it.
export const playtimes = ["Kısa", "Orta", "Uzun"] as const;
export const playtimeOf = (hours: number) => (hours < 10 ? "Kısa" : hours <= 30 ? "Orta" : "Uzun");

const ageDescriptions: Record<string, { title: string; text: string }> = {
	"3+": { title: "Genel oyuncu", text: "Her yaştan oyuncu için uygundur. Rahatsız edici içerik barındırmaz." },
	"7+": { title: "7 yaş ve üzeri", text: "7 yaş ve üzeri oyuncu için uygundur. Hafif fantastik şiddet içerir. Oyuncu 7 yaşından küçükse ebeveyn gözetimi önerilir." },
	"12+": { title: "12 yaş ve üzeri", text: "12 yaş ve üzeri oyuncu için uygundur. Şiddet ve çatışma içerir. Oyuncu 12 yaşından küçükse ebeveyn gözetimi gereklidir." },
	"16+": { title: "16 yaş ve üzeri", text: "16 yaş ve üzeri oyuncu için uygundur. Yoğun şiddet ve olgun temalar içerir. Oyuncu 16 yaşından küçükse ebeveyn gözetimi gereklidir." },
	"18+": { title: "18 yaş ve üzeri", text: "Yalnızca 18 yaş ve üzeri oyuncu için uygundur. Şiddet ve yetişkin içerik barındırır." },
};

/** A short Turkish title and sentence for the age tooltip. */
export const ageDescription = (label: string) => ageDescriptions[label] ?? null;
