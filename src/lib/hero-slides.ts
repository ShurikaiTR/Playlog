export type HeroSlide = {
	slug: string;
	title: string;
	image: string;
	logo?: string;
	rating?: string;
	genres: string[];
	description: string;
	href: string;
};

const unsplash = (id: string) => `https://images.unsplash.com/${id}?w=1600&q=80&auto=format&fit=crop`;

// Placeholder slides until games come from real data.
export const heroSlides: HeroSlide[] = [
	{
		slug: "neon-surgun",
		title: "Neon Sürgün",
		image: unsplash("photo-1542751371-adc38448a05e"),
		rating: "16+",
		genres: ["Aksiyon", "Bilim kurgu", "Açık dünya"],
		description:
			"Işıkları hiç sönmeyen bir şehirde, kimsenin hatırlamadığı bir geçmişin izini sür. Oynadıklarını kaydet, puanla ve listene ekle.",
		href: "/oyun/neon-surgun",
	},
	{
		slug: "piksel-gecesi",
		title: "Piksel Gecesi",
		image: unsplash("photo-1550745165-9bc0b252726f"),
		rating: "7+",
		genres: ["Retro", "Platform"],
		description:
			"Atari salonlarının ruhunu taşıyan, kısa ama tekrar tekrar dönülen bir platform oyunu. Rekorlarını arkadaşlarınla karşılaştır.",
		href: "/oyun/piksel-gecesi",
	},
	{
		slug: "derin-yorunge",
		title: "Derin Yörünge",
		image: unsplash("photo-1538481199705-c710c4e965fc"),
		rating: "12+",
		genres: ["Strateji", "Uzay", "Simülasyon"],
		description:
			"Filonu kur, rotanı çiz ve galaksinin kenarına kadar ilerle. Topluluğun en çok listeye eklediği strateji oyunlarından biri.",
		href: "/oyun/derin-yorunge",
	},
];
